use crate::piece::{Piece, Bug};
use crate::piece::Bug::*;
use crate::hex::{Hex, ORIGIN};
use self::Player::*;
use std::collections::HashMap;

#[derive(Debug)]
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
            .flat_map(|p| open_hexes.iter()
                .filter(|&&hex| {
                    // If past turn 2, filter out any hexes adjacent to enemy pieces
                    if self.turn_no() > 2 {
                        let is_adjacent_to_enemies = self.board.iter()
                            .filter(|(_, board_piece)| board_piece.owner != self.current_player)
                            .fold(false, |acc, (enemy_hex, _)| acc || enemy_hex.is_adj(hex));
                        !is_adjacent_to_enemies
                    } else { true }
                })
                .map(move |hex| Turn::Place(p.clone(), hex.clone()))));

        // if this player's queen is in play, add in the set of possible piece moves
        if !self.unplayed_pieces.contains(&Piece::new(Queen, self.current_player)) {
            moves.extend(self.board.iter()
                .filter(|(_, p)| p.owner == self.current_player) // once the pillbug is implemented, this has gotta go
                .flat_map(|(&start, &p)| self.get_piece_moves(p, start)));
        }

        return moves;
    }

    fn get_piece_moves(&self, piece: Piece, start: Hex) -> Vec<Turn> {
        // setup a version of the board where this piece is gone (i.e. picked up)
        let mut board_without_piece = self.board.clone();
        board_without_piece.remove(&start);
        // if moving this piece uncovers something in a stack, move that piece to the board
        let mut on_hive = false; // remember if we're currently on a stack
        if let Some(stack) = self.stacks.get(&start) {
            if let Some(&under) = stack.last() {
                on_hive = true;
                board_without_piece.insert(start, under);
            }
        }

        // check if removing this piece breaks the One Hive Rule
        let pieces_after_pickup = board_without_piece.keys().cloned().collect();
        if !Hex::all_contiguous(&pieces_after_pickup) {
            return vec![];
        }

        // all open hexes to move to
        let spaces_after_pickup = Hex::get_empty_neighbors(&pieces_after_pickup);

        match piece.bug {
            Ant => start.pathfind(&spaces_after_pickup, &pieces_after_pickup, None).iter()
                .map(|&end| Turn::Move(piece, end))
                .collect(),
            Beetle => {
                // if a beetle's on the hive, it's not restricted by anything except its move
                // speed; if it's not, consider pieces to be barriers like normal
                let empty = vec![];
                let barriers = if on_hive { &empty } else { &pieces_after_pickup };
                start.pathfind(&spaces_after_pickup, barriers, Some(1)).iter()
                    .chain(start.pathfind(&pieces_after_pickup, &vec![], Some(1)).iter())
                    .map(|&end| Turn::Move(piece, end))
                    .collect()
            },
            Queen => start.pathfind(&spaces_after_pickup, &pieces_after_pickup, Some(1)).iter()
                .map(|&end| Turn::Move(piece, end))
                .collect(),
            Spider => start.pathfind(&spaces_after_pickup, &pieces_after_pickup, Some(3)).iter()
                .map(|&end| Turn::Move(piece, end))
                .collect(),
            Grasshopper => start.neighbors().iter()
                .filter(|neighbor| self.board.contains_key(neighbor)) // only hop over adjacent pieces
                .map(|neighbor| {
                    // given a direction to hop, keep looking in that direction until we find
                    // an open hex
                    let direction = neighbor.sub(start);
                    let mut travel = direction;
                    while self.board.contains_key(&neighbor.add(travel)) {
                        travel = travel.add(direction);
                    }
                    Turn::Move(piece, neighbor.add(travel))
                })
                .collect(),
            _ => vec![],
        }
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
            .filter(|p| p.owner == self.current_player)
            .for_each(|p| {
                let id = lowest_ids.entry(p.bug).or_insert(p.id);
                if p.id < *id {
                    *id = p.id;
                }
            });

        self.unplayed_pieces.iter()
            .filter(|p| self.turn_no() > 2 || p.bug != Queen) // disallow queen plays on turn 1
            .filter(|p| Some(&p.id) == lowest_ids.get(&p.bug))
            .filter(|p| p.owner == self.current_player)
            .cloned()
            .collect()
    }

    fn get_hex_for_piece(&self, piece: Piece) -> Option<Hex> {
        // first check the board, then check underneath any stacks
        self.board.iter()
            .find_map(|(&hex, &board_piece)| if board_piece == piece { Some(hex) } else { None })
            .or_else(|| self.stacks.iter()
                .find_map(|(&hex, stack)| if stack.contains(&piece) { Some(hex) } else { None }))
    }

    pub fn submit_turn(&mut self, turn: Turn) -> Result<(), TurnError> {
        match self.status {
            GameStatus::Win(_) | GameStatus::Draw => return Err(TurnError::GameOver),
            _ => {},
        };

        if turn != Turn::Pass && !self.get_valid_moves().contains(&turn) {
            return Err(TurnError::InvalidMove)
        }

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
                let from = self.get_hex_for_piece(piece).unwrap();
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
            if let Some(queen) = self.get_hex_for_piece(Piece::new(Queen, *color)) {
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

#[derive(PartialEq, Debug)]
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
    use crate::test_utils::{assert_set_equality, check_move, get_valid_movements};

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
        check_move(&mut game, Turn::Place(Piece::new(Ant, Black), ORIGIN));
        check_move(&mut game, Turn::Place(Piece::new(Ant, White), ORIGIN.w()));
        check_move(&mut game, Turn::Place(Piece::new(Spider, Black), ORIGIN.e()));
        check_move(&mut game, Turn::Place(Piece::new(Spider, White), ORIGIN.w().w()));
        check_move(&mut game, Turn::Place(Piece::new(Beetle, Black), ORIGIN.e().e()));
        check_move(&mut game, Turn::Place(Piece::new(Beetle, White), ORIGIN.w().w().w()));
        let mut pieces = Vec::new();
        game.get_valid_moves().iter().for_each(|m| match m {
            &Turn::Place(piece, _) => pieces.push(piece),
            _ => panic!("moves are invalid here!"),
        });
        assert_set_equality(pieces, vec![Piece::new(Queen, Black)]);
        check_move(&mut game, Turn::Place(Piece::new(Queen, Black), ORIGIN.ne()));
        let mut pieces = Vec::new();
        game.get_valid_moves().iter().for_each(|m| match m {
            &Turn::Place(piece, _) => pieces.push(piece),
            _ => panic!("moves are invalid here!"),
        });
        assert_set_equality(pieces, vec![Piece::new(Queen, White)]);
        check_move(&mut game, Turn::Place(Piece::new(Queen, White), ORIGIN.w().nw()));
    }

    #[test]
    fn test_simple_movement() {
        let mut game = GameState::new(Black);
        // bS - bA - wA - wQ
        check_move(&mut game, Turn::Place(Piece::new(Ant, Black), ORIGIN));
        check_move(&mut game, Turn::Place(Piece::new(Ant, White), ORIGIN.w()));
        check_move(&mut game, Turn::Place(Piece::new(Queen, Black), ORIGIN.e()));
        check_move(&mut game, Turn::Place(Piece::new(Spider, White), ORIGIN.w().w()));
        assert_set_equality(get_valid_movements(&game), vec![
            Turn::Move(Piece::new(Queen, Black), ORIGIN.ne()),
            Turn::Move(Piece::new(Queen, Black), ORIGIN.se()),
        ]);
        check_move(&mut game, Turn::Move(Piece::new(Queen, Black), ORIGIN.ne()));
        check_move(&mut game, Turn::Place(Piece::new(Queen, White), ORIGIN.w().nw()));
        assert_set_equality(get_valid_movements(&game), vec![
            Turn::Move(Piece::new(Queen, Black), ORIGIN.nw()),
            Turn::Move(Piece::new(Queen, Black), ORIGIN.e()),
        ]);
        check_move(&mut game, Turn::Place(Piece::new(Spider, Black), ORIGIN.e()));
        assert_set_equality(get_valid_movements(&game), vec![
            Turn::Move(Piece::new(Spider, White), ORIGIN.w().nw().ne()),
            Turn::Move(Piece::new(Spider, White), ORIGIN.se()),
            Turn::Move(Piece::new(Queen, White), ORIGIN.nw()),
            Turn::Move(Piece::new(Queen, White), ORIGIN.w().w().nw()),
        ]);
        check_move(&mut game, Turn::Move(Piece::new(Spider, White), ORIGIN.se()));
        check_move(&mut game, Turn::Move(Piece::new(Queen, Black), ORIGIN.nw()));
        assert_set_equality(get_valid_movements(&game), vec![
            Turn::Move(Piece::new(Spider, White), ORIGIN.e().ne()),
            Turn::Move(Piece::new(Spider, White), ORIGIN.w().w()),
            Turn::Move(Piece::new(Queen, White), ORIGIN.nw().nw()),
            Turn::Move(Piece::new(Queen, White), ORIGIN.w().w()),
            Turn::Move(Piece::new(Ant, White), ORIGIN.sw()),
            Turn::Move(Piece::new(Ant, White), ORIGIN.se().se()),
            Turn::Move(Piece::new(Ant, White), ORIGIN.se().sw()),
            Turn::Move(Piece::new(Ant, White), ORIGIN.e().se()),
            Turn::Move(Piece::new(Ant, White), ORIGIN.e().e()),
            Turn::Move(Piece::new(Ant, White), ORIGIN.e().ne()),
            Turn::Move(Piece::new(Ant, White), ORIGIN.ne()),
            Turn::Move(Piece::new(Ant, White), ORIGIN.nw().ne()),
            Turn::Move(Piece::new(Ant, White), ORIGIN.nw().nw()),
            Turn::Move(Piece::new(Ant, White), ORIGIN.nw().w().nw()),
            Turn::Move(Piece::new(Ant, White), ORIGIN.nw().w().w()),
            Turn::Move(Piece::new(Ant, White), ORIGIN.nw().w().sw()),
        ]);
    }

    #[test]
    fn test_grasshoppers() {
        let mut game = GameState::new(Black);
        check_move(&mut game, Turn::Place(Piece::new(Grasshopper, Black), ORIGIN));
        check_move(&mut game, Turn::Place(Piece::new(Spider, White), ORIGIN.w()));
        check_move(&mut game, Turn::Place(Piece::new(Queen, Black), ORIGIN.ne()));
        check_move(&mut game, Turn::Place(Piece::new(Ant, White), ORIGIN.w().nw()));
        check_move(&mut game, Turn::Move(Piece::new(Queen, Black), ORIGIN.nw()));
        check_move(&mut game, Turn::Place(Piece::new(Queen, White), ORIGIN.w().nw().w()));
        assert_set_equality(get_valid_movements(&game), vec![
            Turn::Move(Piece::new(Queen, Black), ORIGIN.ne()),
            Turn::Move(Piece::new(Queen, Black), ORIGIN.nw().nw()),
            Turn::Move(Piece::new(Grasshopper, Black), ORIGIN.nw().nw()),
            Turn::Move(Piece::new(Grasshopper, Black), ORIGIN.w().w()),
        ]);
        check_move(&mut game, Turn::Move(Piece::new(Grasshopper, Black), ORIGIN.w().w()));
        check_move(&mut game, Turn::Place(Piece::new(Grasshopper, White), ORIGIN.w().w().nw().nw()));
        assert_set_equality(get_valid_movements(&game), vec![
            Turn::Move(Piece::new(Queen, Black), ORIGIN),
            Turn::Move(Piece::new(Queen, Black), ORIGIN.nw().nw()),
            Turn::Move(Piece::new(Grasshopper, Black), ORIGIN),
            Turn::Move(Piece::new(Grasshopper, Black), ORIGIN.nw().nw()),
            Turn::Move(Piece::new(Grasshopper, Black), ORIGIN.w().w().nw().nw().nw()),
        ]);
    }

    #[test]
    fn test_beetles() {
        let mut game = GameState::new(Black);
        check_move(&mut game, Turn::Place(Piece::new(Beetle, Black), ORIGIN));
        check_move(&mut game, Turn::Place(Piece::new(Spider, White), ORIGIN.w()));
        check_move(&mut game, Turn::Place(Piece::new(Queen, Black), ORIGIN.ne()));
        check_move(&mut game, Turn::Place(Piece::new(Beetle, White), ORIGIN.w().nw()));
        check_move(&mut game, Turn::Move(Piece::new(Queen, Black), ORIGIN.nw()));
        check_move(&mut game, Turn::Place(Piece::new(Queen, White), ORIGIN.w().w()));
        assert_set_equality(get_valid_movements(&game), vec![
            Turn::Move(Piece::new(Queen, Black), ORIGIN.ne()),
            Turn::Move(Piece::new(Queen, Black), ORIGIN.nw().nw()),
            Turn::Move(Piece::new(Beetle, Black), ORIGIN.ne()),
            Turn::Move(Piece::new(Beetle, Black), ORIGIN.sw()),
            Turn::Move(Piece::new(Beetle, Black), ORIGIN.w()),
            Turn::Move(Piece::new(Beetle, Black), ORIGIN.nw()),
        ]);
        check_move(&mut game, Turn::Move(Piece::new(Beetle, Black), ORIGIN.w()));
        assert_eq!(game.stacks.get(&ORIGIN.w()), Some(&vec![Piece::new(Spider, White)]));
        assert_eq!(game.board.get(&ORIGIN.w()), Some(&Piece::new(Beetle, Black)));
        check_move(&mut game, Turn::Move(Piece::new(Beetle, White), ORIGIN.w()));
        assert_eq!(game.stacks.get(&ORIGIN.w()), Some(&vec![Piece::new(Spider, White), Piece::new(Beetle, Black)]));
        assert_eq!(game.board.get(&ORIGIN.w()), Some(&Piece::new(Beetle, White)));
        check_move(&mut game, Turn::Move(Piece::new(Queen, Black), ORIGIN));
        check_move(&mut game, Turn::Move(Piece::new(Beetle, White), ORIGIN));
        assert_eq!(game.stacks.get(&ORIGIN.w()), Some(&vec![Piece::new(Spider, White)]));
        assert_eq!(game.board.get(&ORIGIN.w()), Some(&Piece::new(Beetle, Black)));
        assert_set_equality(game.get_valid_moves(), vec![
            Turn::Move(Piece::new(Beetle, Black), ORIGIN),
            Turn::Move(Piece::new(Beetle, Black), ORIGIN.nw()),
            Turn::Move(Piece::new(Beetle, Black), ORIGIN.w().nw()),
            Turn::Move(Piece::new(Beetle, Black), ORIGIN.w().w()),
            Turn::Move(Piece::new(Beetle, Black), ORIGIN.w().sw()),
            Turn::Move(Piece::new(Beetle, Black), ORIGIN.sw()),
        ]);

        // complete a circle to test placing beetles in holes
        check_move(&mut game, Turn::Move(Piece::new(Beetle, Black), ORIGIN.sw()));
        check_move(&mut game, Turn::Move(Piece::new(Beetle, White), ORIGIN.w()));
        check_move(&mut game, Turn::Place(Piece::new(Ant, Black), ORIGIN.e()));
        check_move(&mut game, Turn::Place(Piece::new(Ant, White), ORIGIN.w().nw()));
        check_move(&mut game, Turn::Move(Piece::new(Ant, Black), ORIGIN.sw().sw()));
        check_move(&mut game, Turn::Move(Piece::new(Ant, White), ORIGIN.nw()));
        check_move(&mut game, Turn::Place(Piece::new(Spider, Black), ORIGIN.sw().sw().w()));
        check_move(&mut game, Turn::Move(Piece::new(Ant, White), ORIGIN.sw().w().w()));
        check_move(&mut game, Turn::Place(Piece::new(Grasshopper, Black), ORIGIN.ne()));

        // finally, move the beetle into the center of the hole
        check_move(&mut game, Turn::Move(Piece::new(Beetle, White), ORIGIN.sw().w()));
        check_move(&mut game, Turn::Place(Piece { bug: Ant, id: 2, owner: Black }, ORIGIN.ne().ne()));
        // and move it out
        check_move(&mut game, Turn::Move(Piece::new(Beetle, White), ORIGIN.sw()));
    }

    #[test]
    fn test_gap_jumps() {
        /* in a case where there's a curve of pieces w/ a wide gap, hex neighbors that aren't
         * adjacent may appear that way. e.g.
         *
         *     / \ / \ / \
         *    |bG1|bQ1| 4 |
         *   / \ / \ / \ /
         *  |wB1| 2 | 3 |
         *   \ / \ / \ /
         *    |wQ1| 1 |
         *     \ / \ /
         *      |wS1|
         *       \ /
         *
         * here, although hexes 1 and 3 are "adjacent" on the board, wS1 must cross through
         * 2 before hitting 3.
         */
        let mut game = GameState::new(Black);
        check_move(&mut game, Turn::Place(Piece::new(Beetle, Black), ORIGIN));
        check_move(&mut game, Turn::Place(Piece::new(Grasshopper, White), ORIGIN.ne()));
        check_move(&mut game, Turn::Place(Piece::new(Queen, Black), ORIGIN.se()));
        check_move(&mut game, Turn::Place(Piece::new(Queen, White), ORIGIN.ne().e()));
        check_move(&mut game, Turn::Place(Piece::new(Spider, Black), ORIGIN.se().se()));
        check_move(&mut game, Turn::Place(Piece::new(Ant, White), ORIGIN.ne().nw()));
        assert_set_equality(get_valid_movements(&game), vec![
            Turn::Move(Piece::new(Spider, Black), ORIGIN.ne().e().se()),
            Turn::Move(Piece::new(Spider, Black), ORIGIN.w()),
        ]);
    }

    #[test]
    fn test_win_condition() {
        let mut game = GameState::new(Black);
        check_move(&mut game, Turn::Place(Piece::new(Beetle, Black), ORIGIN));
        check_move(&mut game, Turn::Place(Piece::new(Spider, White), ORIGIN.w()));
        check_move(&mut game, Turn::Place(Piece::new(Queen, Black), ORIGIN.ne()));
        check_move(&mut game, Turn::Place(Piece::new(Queen, White), ORIGIN.w().w()));
        check_move(&mut game, Turn::Place(Piece::new(Grasshopper, Black), ORIGIN.e()));
        check_move(&mut game, Turn::Place(Piece::new(Ant, White), ORIGIN.w().nw()));
        check_move(&mut game, Turn::Place(Piece::new(Spider, Black), ORIGIN.e().ne()));
        check_move(&mut game, Turn::Move(Piece::new(Ant, White), ORIGIN.nw()));
        check_move(&mut game, Turn::Place(Piece::new(Ant, Black), ORIGIN.ne().ne()));
        check_move(&mut game, Turn::Place(Piece { bug: Ant, owner: White, id: 2 }, ORIGIN.w().nw()));
        check_move(&mut game, Turn::Place(Piece { bug: Ant, owner: Black, id: 2 }, ORIGIN.e().e()));
        check_move(&mut game, Turn::Move(Piece { bug: Ant, owner: White, id: 2 }, ORIGIN.ne().nw()));
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
    fn test_make_invalid_first_move() {
        let mut new_game = GameState::new(Black);
        let black_queen = Piece::new(Queen, Black);
        let turn = Turn::Place(black_queen, ORIGIN);
        let result = new_game.submit_turn(turn);
        assert_eq!(result.err(), Some(TurnError::InvalidMove));
    }
}
