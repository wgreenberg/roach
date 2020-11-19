use crate::piece::{Piece, Bug};
use crate::piece::Bug::*;
use crate::hex::{Hex, ORIGIN};
use self::Player::*;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct GameState {
    pub unplayed_pieces: Vec<Piece>,
    pub board: HashMap<Hex, Piece>,
    pub stacks: HashMap<Hex, Vec<Piece>>,
    pub turns: Vec<Turn>,
    pub current_player: Player,
    pub status: GameStatus,
    pub game_type: GameType,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum GameType {
    Base,
    PLM(bool, bool, bool),
}

#[derive(PartialEq, Debug)]
pub enum TurnError {
    WrongPlayer,
    InvalidMove,
    GameOver,
}

impl GameState {
    pub fn new_with_type(first_player: Player, game_type: GameType) -> GameState {
        GameState {
            unplayed_pieces: get_initial_pieces(game_type),
            board: HashMap::new(),
            stacks: HashMap::new(),
            turns: Vec::new(),
            current_player: first_player,
            status: GameStatus::NotStarted,
            game_type,
        }
    }
    pub fn new(first_player: Player) -> GameState {
        GameState::new_with_type(first_player, GameType::Base)
    }

    pub fn turn_no(&self) -> usize { self.turns.len() + 1 }

    pub fn get_valid_moves(&self) -> Vec<Turn> {
        let mut moves = Vec::new();
        let open_hexes = match self.status {
            GameStatus::NotStarted => vec![ORIGIN],
            _ => Hex::get_empty_neighbors(&self.board.keys().cloned().collect()),
        };

        // start with the set of piece placements
        moves.extend(self.get_placeable_pieces().iter()
            .flat_map(|piece| open_hexes.iter()
                .filter(|&hex| {
                    // If past turn 2, filter out any hexes adjacent to enemy pieces
                    if self.turn_no() > 2 {
                        self.board.iter()
                            .filter(|(_, board_piece)| board_piece.owner != self.current_player)
                            .all(|(enemy_hex, _)| !enemy_hex.is_adj(hex))
                    } else { true }
                })
                .map(move |hex| Turn::Place(piece.clone(), hex.clone()))));

        // if this player's queen is in play, add in the set of possible piece moves
        if !self.unplayed_pieces.contains(&Piece::new(Queen, self.current_player)) {
            // TODO filter out moves that don't change board state
            moves.extend(self.board.iter()
                .filter(|(_, piece)| piece.owner == self.current_player)
                .filter(|(_, &piece)| match self.turns.last() {
                    // pieces that have been pillbugged can't move for a turn, and the only time
                    // the current player's piece would've been moved a turn ago is during a
                    // pillbug ability
                    Some(Turn::Move(moved_piece, _)) => piece != *moved_piece,
                    _ => true,
                })
                .flat_map(|(start, piece)| self.get_piece_moves(piece, start)));
        }

        if moves.len() == 0 {
            vec![Turn::Pass]
        } else {
            moves
        }
    }

    fn check_one_hive_rule(&self, board: &Vec<Hex>, piece: &Hex) -> bool {
        // before we do an expensive call to Hex::all_contiguous, check if this hex has only one
        // group of contiguous neighbors -- if so, we can easily say it doesn't violate the rule
        let mut group = false;
        let mut n_flips = 0;
        for (i, neighbor) in piece.neighbors().iter().enumerate() {
            if self.board.contains_key(neighbor) {
                if !group {
                    group = true;
                    if i > 0 {
                        n_flips += 1;
                    }
                }
            } else if group {
                group = false;
                n_flips += 1;
            }
        }
        if n_flips <= 2 {
            return true;
        }
        Hex::all_contiguous(&board)
    }

    fn get_piece_moves(&self, piece: &Piece, start: &Hex) -> Vec<Turn> {
        // setup a version of the board where this piece is gone (i.e. picked up)
        let mut board_without_piece = self.board.clone();
        board_without_piece.remove(&start);
        // if moving this piece uncovers something in a stack, move that piece to the board
        let mut on_hive = false; // remember if we're currently on a stack
        if let Some(stack) = self.stacks.get(&start) {
            if let Some(&under) = stack.last() {
                on_hive = true;
                board_without_piece.insert(*start, under);
            }
        }

        // check if removing this piece breaks the One Hive Rule
        let pieces_after_pickup = board_without_piece.keys().cloned().collect();
        if !on_hive && !self.check_one_hive_rule(&pieces_after_pickup, start) {
            // but if this is a pillbug (or a mosquito imitating a pillbug), just return the pieces
            // it can toss
            // TODO this doesn't cover e.g. Black tosses their Queen and White tries to toss the
            // same Queen
            match piece.bug {
                Pillbug => return self.get_pillbug_tosses(start),
                Mosquito => return start.neighbors().iter()
                    .flat_map(|neighbor| self.board.get(neighbor))
                    .find(|neighbor_piece| neighbor_piece.bug == Pillbug)
                    .map_or(vec![], |_| self.get_pillbug_tosses(start)),
                _ => return vec![],
            }
        }

        // all open hexes to move to
        let spaces_after_pickup = Hex::get_empty_neighbors(&pieces_after_pickup);

        match piece.bug {
            Ant => start.pathfind(&spaces_after_pickup, &pieces_after_pickup, None).iter()
                .map(|end| Turn::Move(*piece, *end))
                .collect(),
            // TODO: add exception for stacked pincers
            Beetle => {
                // if a beetle's on the hive, it's not restricted by anything except its move
                // speed; if it's not, consider pieces to be barriers like normal
                let empty = vec![];
                let barriers = if on_hive { &empty } else { &pieces_after_pickup };
                start.pathfind(&spaces_after_pickup, barriers, Some(1)).iter()
                    .chain(start.pathfind(&pieces_after_pickup, &vec![], Some(1)).iter())
                    .map(|end| Turn::Move(*piece, *end))
                    .collect()
            },
            Queen => start.pathfind(&spaces_after_pickup, &pieces_after_pickup, Some(1)).iter()
                .map(|end| Turn::Move(*piece, *end))
                .collect(),
            Spider => start.pathfind(&spaces_after_pickup, &pieces_after_pickup, Some(3)).iter()
                .map(|end| Turn::Move(*piece, *end))
                .collect(),
            Grasshopper => start.neighbors().iter()
                .filter(|neighbor| self.board.contains_key(neighbor)) // only hop over adjacent pieces
                .map(|neighbor| {
                    // given a direction to hop, keep looking in that direction until we find
                    // an open hex
                    let direction = neighbor.sub(&start);
                    let mut travel = direction;
                    while self.board.contains_key(&neighbor.add(&travel)) {
                        travel = travel.add(&direction);
                    }
                    Turn::Move(*piece, neighbor.add(&travel))
                })
                .collect(),
            Pillbug => start.pathfind(&spaces_after_pickup, &pieces_after_pickup, Some(1)).iter()
                .map(|end| Turn::Move(*piece, *end))
                .chain(self.get_pillbug_tosses(start))
                .collect(),
            // TODO: add exception for stacked pincers
            Ladybug => start.pathfind(&pieces_after_pickup, &vec![], Some(2)).iter()
                .flat_map(|on_hive| on_hive.neighbors().iter()
                    .filter(|neighbor| !self.board.contains_key(neighbor))
                    .map(|end| Turn::Move(*piece, *end)).collect::<Vec<Turn>>())
                .collect(),
            Mosquito => {
                if on_hive {
                    self.get_piece_moves(&Piece::new(Beetle, piece.owner), start).iter()
                        .map(|&turn| match turn {
                            Turn::Move(_, dest) => Turn::Move(*piece, dest),
                            _ => unreachable!(),
                        }).collect::<Vec<Turn>>()
                } else {
                    start.neighbors().iter()
                        .flat_map(|neighbor| self.board.get(neighbor))
                        .filter(|neighbor_piece| neighbor_piece.bug != Mosquito)
                        .flat_map(|&neighbor_piece| {
                            // if we're imitating a pillbug, we unfortunately have to manually calculate
                            // the moves here since it's impossible to distinguish moves (which we want
                            // to overwrite the piece value of) from tosses (which we don't) from the
                            // results of self.get_piece_moves(...)
                            if neighbor_piece.bug == Pillbug {
                                start.pathfind(&spaces_after_pickup, &pieces_after_pickup, Some(1)).iter()
                                    .map(|end| Turn::Move(*piece, *end))
                                    .chain(self.get_pillbug_tosses(start))
                                    .collect()
                            } else {
                                // for normal moves, overwrite the piece value with our mosquito
                                self.get_piece_moves(&neighbor_piece, start).iter()
                                    .map(|&turn| match turn {
                                        Turn::Move(_, dest) => Turn::Move(*piece, dest),
                                        _ => unreachable!(),
                                    }).collect::<Vec<Turn>>()
                            }
                        })
                        .collect()
                }
            },
        }
    }

    fn get_pillbug_tosses(&self, hex: &Hex) -> Vec<Turn> {
        let (neighbors, empty): (Vec<Hex>, Vec<Hex>) = hex.neighbors().iter()
            .partition(|hex| self.board.contains_key(hex));
        neighbors.iter()
            .filter(|neighbor| match self.turns.last() {
                // we can't move neighbors that've just been moved
                Some(Turn::Move(_, hex)) => hex != *neighbor,
                _ => true,
            })
            // can't toss pices on a stack
            .filter(|neighbor| self.stacks.get(neighbor).map_or(true, |stack| stack.len() == 0))
            .filter(|neighbor| {
                // check if this neighbor can be moved w/o violating the One Hive Rule
                let mut board_without_neighbor = self.board.clone();
                board_without_neighbor.remove(neighbor);
                let pieces_without_neighbor = board_without_neighbor.keys().cloned().collect();
                // TODO: add exception for stacked pincers
                self.check_one_hive_rule(&pieces_without_neighbor, neighbor)
            })
            .flat_map(|neighbor| {
                let neighbor_piece = self.board.get(neighbor).unwrap();
                empty.iter().map(move |dest| Turn::Move(neighbor_piece.clone(), dest.clone()))
            })
            .collect()
    }

    fn get_placeable_pieces(&self) -> Vec<Piece> {
        // if it's a player's 4th turn (i.e. game turn 7 or 8) and their queen isn't out, force it
        if self.turn_no() == 7 || self.turn_no() == 8 {
            let player_queen = Piece::new(Queen, self.current_player);
            if self.unplayed_pieces.contains(&player_queen) {
                return vec![player_queen];
            }
        }

        let mut lowest_ids: HashMap<Bug, u8> = HashMap::new();
        self.unplayed_pieces.iter()
            .filter(|piece| piece.owner == self.current_player)
            .for_each(|piece| {
                let id = lowest_ids.entry(piece.bug).or_insert(piece.id);
                if piece.id < *id {
                    *id = piece.id;
                }
            });

        self.unplayed_pieces.iter()
            .filter(|piece| self.turn_no() > 2 || piece.bug != Queen) // disallow queen plays on turn 1
            .filter(|piece| Some(&piece.id) == lowest_ids.get(&piece.bug))
            .filter(|piece| piece.owner == self.current_player)
            .cloned()
            .collect()
    }

    pub fn get_hex_for_piece(&self, piece: &Piece) -> Option<Hex> {
        // first check the board, then check underneath any stacks
        self.board.iter()
            .find_map(|(&hex, board_piece)| if board_piece == piece { Some(hex) } else { None })
            .or_else(|| self.stacks.iter()
                .find_map(|(&hex, stack)| if stack.contains(&piece) { Some(hex) } else { None }))
    }

    pub fn submit_turn_unchecked(&mut self, turn: Turn) {
        if self.status == GameStatus::NotStarted {
            self.status = GameStatus::InProgress;
        }
        self.current_player = self.current_player.other();
        match turn {
            Turn::Place(piece, hex) => {
                assert!(self.board.insert(hex, piece).is_none());
                self.unplayed_pieces.retain(|&p| p != piece);
            },
            Turn::Move(piece, dest) => {
                let from = self.get_hex_for_piece(&piece).unwrap();
                assert!(self.board.remove(&from).is_some());
                // if this piece is uncovering something in a stack, move it onto the board
                if let Some(stack) = self.stacks.get_mut(&from) {
                    if let Some(under) = stack.pop() {
                        self.board.insert(from, under);
                    }
                }
                // if this piece moving somewhere that covers a piece, move that piece into a new
                // stack
                if let Some(existing) = self.board.insert(dest, piece) {
                    self.stacks.entry(dest).or_insert(Vec::new()).push(existing);
                }
            },
            Turn::Pass => {},
        }
        self.turns.push(turn);

        // check for win condition
        let mut num_wins = 0;
        for color in [White, Black].iter() {
            if let Some(queen) = self.get_hex_for_piece(&Piece::new(Queen, *color)) {
                let n_neighbors = queen.neighbors().iter()
                    .filter(|hex| self.board.contains_key(hex)).count();
                if n_neighbors == 6 {
                    self.status = GameStatus::Win(color.other());
                    num_wins += 1;
                }
            }
        }
        if num_wins == 2 {
            self.status = GameStatus::Draw;
        }
    }

    pub fn submit_turn(&mut self, turn: Turn) -> Result<(), TurnError> {
        match self.status {
            GameStatus::Win(_) | GameStatus::Draw => return Err(TurnError::GameOver),
            _ => {},
        };

        if turn != Turn::Pass && !self.get_valid_moves().contains(&turn) {
            return Err(TurnError::InvalidMove)
        }

        self.submit_turn_unchecked(turn);
        Ok(())
    }
}

fn get_initial_pieces(game_type: GameType) -> Vec<Piece> {
    let mut pieces = Vec::new();
    for &player in [White, Black].iter() {
        pieces.extend(Piece::new_set(Ant, player, 3));
        pieces.extend(Piece::new_set(Grasshopper, player, 3));
        pieces.extend(Piece::new_set(Beetle, player, 2));
        pieces.extend(Piece::new_set(Spider, player, 2));
        pieces.push(Piece::new(Queen, player));
        if let GameType::PLM(pillbug, ladybug, mosquito) = game_type {
            if pillbug { pieces.push(Piece::new(Pillbug, player)); }
            if ladybug { pieces.push(Piece::new(Ladybug, player)); }
            if mosquito { pieces.push(Piece::new(Mosquito, player)); }
        }
    }
    pieces
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Player {
    White,
    Black,
}

impl Player {
    pub fn other(&self) -> Player {
        match self {
            White => Black,
            Black => White,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum GameStatus {
    NotStarted,
    InProgress,
    Draw,
    Win(Player),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Turn  {
    Place(Piece, Hex),
    Move(Piece, Hex),
    Pass,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::{assert_set_equality, check_move, play_and_verify,
                            assert_valid_movements, assert_piece_movements};

    #[test]
    fn test_first_valid_moves() {
        let new_game = GameState::new(Black);
        let all_but_queen = vec![
            Turn::Place(Piece::new(Ant, Black), ORIGIN),
            Turn::Place(Piece::new(Beetle, Black), ORIGIN),
            Turn::Place(Piece::new(Grasshopper, Black), ORIGIN),
            Turn::Place(Piece::new(Spider, Black), ORIGIN),
        ];
        assert_set_equality(new_game.get_valid_moves(), all_but_queen);
    }

    #[test]
    fn test_make_first_move() {
        let mut new_game = GameState::new(Black);
        let black_ant_1 = Piece::new(Ant, Black);
        let turn = Turn::Place(black_ant_1, ORIGIN);
        check_move(&mut new_game, turn);
        assert_eq!(new_game.current_player, White);
        assert_eq!(new_game.board.get(&ORIGIN), Some(&black_ant_1));
        assert_eq!(new_game.unplayed_pieces.len(), get_initial_pieces(GameType::Base).len() - 1);
        assert_eq!(new_game.status, GameStatus::InProgress);
        assert_eq!(new_game.turns, vec![turn]);
    }

    #[test]
    fn test_make_second_move() {
        let mut game = GameState::new(Black);
        let black_ant_1 = Piece::new(Ant, Black);
        let turn_1 = Turn::Place(black_ant_1, ORIGIN);
        check_move(&mut game, turn_1);

        // 6 possible hexes * 4 possible pieces = 24 possible moves for White
        assert_eq!(game.get_valid_moves().len(), 24);
        let white_spider_1 = Piece::new(Spider, White);
        let west_of_origin = ORIGIN.w();
        let turn_2 = Turn::Place(white_spider_1, west_of_origin);
        check_move(&mut game, turn_2);
        assert_eq!(game.board.get(&ORIGIN), Some(&black_ant_1));
        assert_eq!(game.board.get(&west_of_origin), Some(&white_spider_1));
        assert_eq!(game.unplayed_pieces.len(), get_initial_pieces(GameType::Base).len() - 2);
    }

    #[test]
    fn test_make_third_move() {
        let mut game = GameState::new(Black);
        let black_ant_1 = Piece::new(Ant, Black);
        let turn_1 = Turn::Place(black_ant_1, ORIGIN);
        check_move(&mut game, turn_1);
        let white_spider_1 = Piece::new(Spider, White);
        let west_of_origin = ORIGIN.w();
        let turn_2 = Turn::Place(white_spider_1, west_of_origin);
        check_move(&mut game, turn_2);

        let mut pieces = Vec::new();
        let mut hexes = Vec::new();
        game.get_valid_moves().iter().for_each(|m| match m {
            &Turn::Place(piece, hex) => {
                pieces.push(piece);
                hexes.push(hex);
            },
            _ => panic!("moves are invalid here!"),
        });
        // Only 3 valid hexes remain for placement, and 5 pieces = 15 moves
        assert_set_equality(pieces, vec![
            Piece { bug: Ant, owner: Black, id: 2 },
            Piece::new(Beetle, Black),
            Piece::new(Grasshopper, Black),
            Piece::new(Queen, Black),
            Piece::new(Spider, Black),
        ]);
        assert_set_equality(hexes, vec![ORIGIN.ne(), ORIGIN.e(), ORIGIN.se()]);
        assert_eq!(game.get_valid_moves().len(), 15);

        let black_ant_2 = Piece { bug: Ant, owner: Black, id: 2 };
        let east_of_origin = ORIGIN.e();
        let turn_3 = Turn::Place(black_ant_2, east_of_origin);
        check_move(&mut game, turn_3);
    }

    #[test]
    fn test_queen_placement_rule() {
        let mut game = GameState::new(Black);
        play_and_verify(&mut game, vec![
            "bA1",
            "wA1 -bA1",
            "bS1 bA1-",
            "wS1 -wA1",
            "bB1 bS1-",
            "wB1 -wS1",
        ]);
        let mut pieces = Vec::new();
        game.get_valid_moves().iter().for_each(|m| match m {
            &Turn::Place(piece, _) => pieces.push(piece),
            _ => panic!("moves are invalid here!"),
        });
        assert_set_equality(pieces, vec![Piece::new(Queen, Black)]);
        play_and_verify(&mut game, vec!["bQ1 \\bS1"]);
        let mut pieces = Vec::new();
        game.get_valid_moves().iter().for_each(|m| match m {
            &Turn::Place(piece, _) => pieces.push(piece),
            _ => panic!("moves are invalid here!"),
        });
        assert_set_equality(pieces, vec![Piece::new(Queen, White)]);
        play_and_verify(&mut game, vec!["wQ1 \\wA1"]);
    }

    #[test]
    fn test_simple_movement() {
        let mut game = GameState::new(Black);
        play_and_verify(&mut game, vec![
            "bA1",
            "wA1 -bA1",
            "bQ1 bA1-",
            "wS1 -wA1",
        ]);
        assert_valid_movements(&game, vec![
            "bQ1 bA1/",
            "bQ1 bA1\\",
        ]);
        play_and_verify(&mut game, vec![
            "bQ1 \\bQ1",
            "wQ1 \\wA1",
        ]);
        assert_valid_movements(&game, vec![
            "bQ1 bA1-",
            "bQ1 \\bA1",
        ]);
        play_and_verify(&mut game, vec!["bS1 bQ1\\"]);
        assert_valid_movements(&game, vec![
            "wS1 wQ1/",
            "wS1 bA1\\",
            "wQ1 wA1/",
            "wQ1 \\wS1",
        ]);
        play_and_verify(&mut game, vec![
            "wS1 /bS1",
            "bQ1 \\bA1",
        ]);
        assert_valid_movements(&game, vec![
            "wS1 bS1/",
            "wS1 -wA1",
            "wQ1 \\bQ1",
            "wQ1 -wA1",
            "wA1 /bA1",
            "wA1 -wS1",
            "wA1 /wS1",
            "wA1 wS1\\",
            "wA1 wS1-",
            "wA1 bS1\\",
            "wA1 bS1-",
            "wA1 bS1/",
            "wA1 \\bS1",
            "wA1 bQ1-",
            "wA1 bQ1/",
            "wA1 \\bQ1",
            "wA1 \\wQ1",
            "wA1 -wQ1",
            "wA1 /wQ1",
        ]);
    }

    #[test]
    fn test_grasshoppers() {
        let mut game = GameState::new(Black);
        play_and_verify(&mut game, vec![
            "bG1",
            "wS1 -bG1",
            "bQ1 bG1/",
            "wA1 \\wS1",
            "bQ1 \\bG1",
            "wQ1 -wA1",
        ]);
        assert_valid_movements(&game, vec![
            "bQ1 bG1/",
            "bQ1 wA1/",
            "bG1 \\bQ1",
            "bG1 -wS1",
        ]);
        play_and_verify(&mut game, vec![
            "bG1 /wA1",
            "wG1 \\wQ1",
        ]);
        assert_valid_movements(&game, vec![
            "bQ1 wS1-",
            "bQ1 wA1/",
            "bG1 wA1/",
            "bG1 \\wG1",
            "bG1 wS1-",
        ]);
    }

    #[test]
    fn test_beetles() {
        let mut game = GameState::new(Black);
        play_and_verify(&mut game, vec![
            "bB1",
            "wS1 -bB1",
            "bQ1 bB1/",
            "wB1 \\wS1",
            "bQ1 \\bB1",
            "wQ1 /wB1",
        ]);
        assert_valid_movements(&game, vec![
            "bQ1 bB1/",
            "bQ1 wB1/",
            "bB1 bQ1-",
            "bB1 wS1\\",
            "bB1 /bQ1",
            "bB1 wS1/",
        ]);
        play_and_verify(&mut game, vec!["bB1 /bQ1"]);
        assert_eq!(game.stacks.get(&ORIGIN.w()), Some(&vec![Piece::new(Spider, White)]));
        assert_eq!(game.board.get(&ORIGIN.w()), Some(&Piece::new(Beetle, Black)));
        play_and_verify(&mut game, vec!["wB1 /bQ1"]);
        assert_eq!(game.stacks.get(&ORIGIN.w()), Some(&vec![Piece::new(Spider, White), Piece::new(Beetle, Black)]));
        assert_eq!(game.board.get(&ORIGIN.w()), Some(&Piece::new(Beetle, White)));
        play_and_verify(&mut game, vec![
            "bQ1 bQ1\\",
            "wB1 wB1-",
        ]);
        assert_eq!(game.stacks.get(&ORIGIN.w()), Some(&vec![Piece::new(Spider, White)]));
        assert_eq!(game.board.get(&ORIGIN.w()), Some(&Piece::new(Beetle, Black)));
        assert_valid_movements(&game, vec![
            "bB1 /wB1",
            "bB1 \\wB1",
            "bB1 wQ1/",
            "bB1 wQ1\\",
            "bB1 wS1-",
            "bB1 -wS1",
        ]);

        // complete a circle to test placing beetles in holes
        play_and_verify(&mut game, vec![
            "bB1 /wB1",
            "wB1 \\bB1",
            "bA1 bQ1-",
            "wA1 \\wB1",
            "bA1 /bB1",
            "wA1 \\bQ1",
            "bS1 -bA1",
            "wA1 /wQ1",
            "bG1 bQ1/",
        ]);

        // finally, move the beetle into the center of the hole
        play_and_verify(&mut game, vec![
            "wB1 /wB1",
            "bA2 bG1/",
        ]);
        // and move it out
        play_and_verify(&mut game, vec!["wB1 /bQ1"]);
    }

    #[test]
    fn test_gap_jumps() {
        /* in a case where there's a curve of pieces w/ a wide gap, hex neighbors that aren't
         * adjacent may appear that way. e.g.
         *
         *     / \ / \ / \
         *    |wG1|wQ1| 4 |
         *   / \ / \ / \ /
         *  |bB1| 2 | 3 |
         *   \ / \ / \ /
         *    |bQ1| 1 |
         *     \ / \ /
         *      |bS1|
         *       \ /
         *
         * here, although hexes 1 and 3 are "adjacent" on the board, wS1 must cross through
         * 2 before hitting 3.
         */
        let mut game = GameState::new(Black);
        play_and_verify(&mut game, vec![
            "bB1",
            "wG1 bB1/",
            "bQ1 bB1\\",
            "wQ1 wG1-",
            "bS1 bQ1\\",
            "wA1 \\wG1",
        ]);
        assert_valid_movements(&game, vec![
            "bS1 wQ1\\",
            "bS1 -bB1",
        ]);
    }

    #[test]
    fn test_win_condition() {
        let mut game = GameState::new(Black);
        play_and_verify(&mut game, vec![
            "bB1",
            "wS1 -bB1",
            "bQ1 bB1/",
            "wQ1 -wS1",
            "bG1 bQ1\\",
            "wA1 \\wS1",
            "bS1 bG1/",
            "wA1 \\bB1",
            "bA1 \\bS1",
            "wA2 \\wS1",
            "bA2 bS1\\",
            "wA2 \\bQ1",
        ]);
        assert_eq!(game.status, GameStatus::Win(White));
        assert_eq!(game.submit_turn(Turn::Move(Piece::new(Beetle, Black), ORIGIN.ne())).err(),
                   Some(TurnError::GameOver));
    }

    fn count_pieces(game: &GameState, player: Player) -> Vec<(Bug, usize)> {
        let mut counts = HashMap::new();
        game.unplayed_pieces.iter()
            .for_each(|piece| { if piece.owner == player { *counts.entry(piece.bug).or_insert(0) += 1 }});
        counts.iter().map(|(&a, &b)| (a, b)).collect()
    }

    #[test]
    fn test_initial_pieces() {
        assert_set_equality(count_pieces(&GameState::new(Black), Black), vec![
            (Queen, 1), (Beetle, 2), (Spider, 2), (Grasshopper, 3), (Ant, 3),
        ]);
        let p = GameState::new_with_type(Black, GameType::PLM(true, false, false));
        assert_set_equality(count_pieces(&p, Black), vec![
            (Queen, 1), (Beetle, 2), (Spider, 2), (Grasshopper, 3), (Ant, 3), (Pillbug, 1),
        ]);
        let l = GameState::new_with_type(Black, GameType::PLM(false, true, false));
        assert_set_equality(count_pieces(&l, Black), vec![
            (Queen, 1), (Beetle, 2), (Spider, 2), (Grasshopper, 3), (Ant, 3), (Ladybug, 1),
        ]);
        let m = GameState::new_with_type(Black, GameType::PLM(false, false, true));
        assert_set_equality(count_pieces(&m, Black), vec![
            (Queen, 1), (Beetle, 2), (Spider, 2), (Grasshopper, 3), (Ant, 3), (Mosquito, 1),
        ]);
        let plm = GameState::new_with_type(Black, GameType::PLM(true, true, true));
        assert_set_equality(count_pieces(&plm, Black), vec![
            (Queen, 1), (Beetle, 2), (Spider, 2), (Grasshopper, 3), (Ant, 3),
            (Pillbug, 1), (Ladybug, 1), (Mosquito, 1),
        ]);
    }

    #[test]
    fn test_pillbug() {
        let mut game = GameState::new_with_type(Black, GameType::PLM(true, false, false));
        play_and_verify(&mut game, vec![
            "bP1",
            "wS1 -bP1",
            "bQ1 bP1/",
            "wQ1 \\wS1",
            "bQ1 \\bP1",
            "wQ1 \\bQ1",
        ]);
        assert_valid_movements(&game, vec![
            "bP1 wS1\\",
            "bP1 bQ1-",
            "wS1 /bP1",
            "wS1 bP1\\",
            "wS1 bP1-",
            "wS1 bP1/",
        ]);
        play_and_verify(&mut game, vec!["wS1 bP1-"]);
        // make sure white can't move the white spider, since it was just pillbug'd
        assert_valid_movements(&game, vec![
            "wQ1 bQ1/",
            "wQ1 -bQ1",
        ]);
        play_and_verify(&mut game, vec![
            "wQ1 -bQ1",
            "bS1 bQ1/",
            "wS1 -bP1",
        ]);
        // make sure the pillbug can only move normally, since the white Spider just moved and
        // thus cannot be pillbug'd
        assert_piece_movements(&game, "bP1", vec![
            "bP1 wS1\\",
            "bP1 bQ1-",
        ]);
        play_and_verify(&mut game, vec![
            "bB1 bP1/",
            "wB1 /wS1",
            "bB1 bQ1",
            "wB1 wS1",
        ]);
        // again, the pillbug can only move normally because the two adjacent pieces are stacks
        assert_piece_movements(&game, "bP1", vec![
            "bP1 wB1\\",
            "bP1 bB1-",
        ]);
    }

    #[test]
    fn test_ladybug() {
        let mut game = GameState::new_with_type(Black, GameType::PLM(false, true, false));
        play_and_verify(&mut game, vec![
            "bL1",
            "wS1 -bL1",
            "bQ1 bL1/",
            "wQ1 \\wS1",
            "bQ1 \\bL1",
            "wA1 /wS1",
            "bQ1 wQ1/",
            "wA2 /wA1",
        ]);
        assert_valid_movements(&game, vec![
            "bQ1 wQ1-",
            "bQ1 \\wQ1",
            "bL1 wQ1-",
            "bL1 \\wQ1",
            "bL1 -wQ1",
            "bL1 /wQ1",
            "bL1 wA1-",
            "bL1 wA1\\",
            "bL1 -wA1",
            "bL1 \\wA1",
        ]);

        // from ./test_data/HV-omiomio-andyy-2020-03-28-0355.sgf
        let mut game = GameState::new_with_type(White, GameType::PLM(true, true, true));
        play_and_verify(&mut game, vec![
            "wS1",
            "bG1 /wS1",
            "wS2 wS1-",
            "bL1 bG1\\",
            "wG1 \\wS1",
            "bQ -bL",
            "wQ -wG1",
            "bS1 \\bQ",
            "wS2 bL\\",
            "bS1 -wQ",
            "wA1 wG1-",
            "bB1 \\bS1",
            "wA1 \\bB1",
            "bB2 /bS1",
            "wB1 wQ/",
            "bB2 /wQ",
            "wS2 -bQ",
        ]);
        assert_piece_movements(&game, "bL1", vec![
            "bL1 -wS2",
            "bL1 \\wS2",
            "bL1 /wS2",
            "bL1 wS2\\",
            "bL1 wS2/",
            "bL1 -bG1",
            "bL1 bG1-",
            "bL1 \\bG1",
            "bL1 -wS1",
            "bL1 wS1-",
            "bL1 wS1/",
            "bL1 wS1\\",
            "bL1 bQ1\\",
        ]);
    }

    #[test]
    fn test_mosquito() {
        let mut game = GameState::new_with_type(Black, GameType::PLM(false, false, true));
        play_and_verify(&mut game, vec![
            "bM1",
            "wS1 -bM1",
            "bQ1 bM1/",
            "wQ1 \\wS1",
            "bQ1 \\bM1",
            "wA1 /wS1",
            "bG1 bM1/",
            "wA1 -wS1",
        ]);
        assert_valid_movements(&game, vec![
            "bQ1 \\bG1",
            "bQ1 wQ1/",
            "bG1 -wQ1",
            "bG1 /bM1",
            "bM1 /wA1", // mimic spider
            "bM1 bG1/", // mimic spider
            "bM1 wS1\\", // mimic queen
            "bM1 bG1\\", // mimic queen
            "bM1 bG1/", // mimic grasshopper
            "bM1 \\bQ1", // mimic grasshopper
            "bM1 -wA1", // mimic grasshopper
        ]);

        // test a case where we imitate a pillbug
        let mut game2 = GameState::new_with_type(White, GameType::PLM(true, true, true));
        play_and_verify(&mut game2, vec![
            "wL1",
            "bL1 \\wL1",
            "wP1 wL-",
            "bM1 bL/",
            "wS1 wP-",
            "bQ \\bL",
            "wQ wP\\",
            "bM1 -wQ",
            "wS2 wQ\\",
            "wP1 -bM",
        ]);

        // make sure when a mosquito's on the hive, it can only move like a beetle until it drops
        // back down
        let mut game3 = GameState::new_with_type(White, GameType::PLM(true, true, true));
        play_and_verify(&mut game3, vec![
            "wM1",
            "bB1 -wM1",
            "wQ1 wM1/",
            "bQ1 \\bB1",
            "wQ1 \\wM1",
            "bA1 -bB1",
            "wM1 /wQ1", // beetle movement onto the hive
            "bG1 -bA1",
        ]);
        assert_piece_movements(&game3, "wM1", vec![
            "wM1 -wQ1",
            "wM1 bQ1-",
            "wM1 /bQ1",
            "wM1 /bB1",
            "wM1 bB1\\",
            "wM1 bB1-",
        ]);
        play_and_verify(&mut game3, vec![
            "wM1 -wQ1",
            "bG1 bB1-",
        ]);
        assert_piece_movements(&game3, "wM1", vec![
            "wM1 /wQ1",
            "wM1 bB1/",
            "wM1 -bB1",
            "wM1 \\bA1",
            "wM1 \\bQ1",
            "wM1 bQ1/",
        ]);
    }

    #[test]
    fn test_make_invalid_first_move() {
        let mut new_game = GameState::new(Black);
        let black_queen = Piece::new(Queen, Black);
        let turn = Turn::Place(black_queen, ORIGIN);
        let result = new_game.submit_turn(turn);
        assert_eq!(result.err(), Some(TurnError::InvalidMove));
    }
}
