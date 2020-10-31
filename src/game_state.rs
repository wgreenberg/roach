use crate::piece::{Piece, Bug};
use crate::piece::Bug::*;
use crate::hex::{Hex, ORIGIN};
use self::Player::*;
use std::collections::HashMap;

pub struct GameState {
    pub unplayed_pieces: Vec<Piece>,
    pub board: HashMap<Hex, Piece>,
    pub turns: Vec<Turn>,
    pub current_player: Player,
    pub status: GameStatus,
}

#[derive(PartialEq, Debug)]
pub enum TurnError {
    WrongPlayer,
    InvalidMove,
}

impl GameState {
    pub fn new() -> GameState {
        GameState {
            unplayed_pieces: get_initial_pieces(),
            board: HashMap::new(),
            turns: Vec::new(),
            current_player: White,
            status: GameStatus::NotStarted,
        }
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
                            .filter(|(_, bp)| bp.owner != self.current_player)
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
        // check if removing this piece breaks the One Hive Rule
        let mut board_without_piece = self.board.clone();
        board_without_piece.remove(&start);
        let hexes_without_piece = board_without_piece.keys().cloned().collect();
        if !Hex::all_contiguous(&hexes_without_piece) {
            return vec![];
        }
        match piece.bug {
            Queen => {
                let possible_moves = start.pathfind(hexes_without_piece, 1);
                vec![
                    Turn::Move(White, Piece::new(Queen, White), ORIGIN.ne()),
                    Turn::Move(White, Piece::new(Queen, White), ORIGIN.se()),
                ]
            },
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

    pub fn submit_turn(&mut self, turn: Turn) -> Result<(), TurnError> {
        if !self.get_valid_moves().contains(&turn) {
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
            Turn::Move(_, _, _) => todo!(),
        }
        self.turns.push(turn);
        Ok(())
    }
}

fn get_initial_pieces() -> Vec<Piece> {
    let mut pieces = Vec::new();
    for &player in [White, Black].iter() {
        pieces.append(&mut Piece::new_set(Ant, player, 3));
        pieces.append(&mut Piece::new_set(Grasshopper, player, 3));
        pieces.append(&mut Piece::new_set(Beetle, player, 2));
        pieces.append(&mut Piece::new_set(Spider, player, 2));
        pieces.push(Piece::new(Queen, player));
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
    Move(Player, Piece, Hex),
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashSet;
    use std::hash::Hash;
    use std::fmt::Debug;

    fn assert_set_equality<T>(got: Vec<T>, expected: Vec<T>)
        where T: Clone + Eq + Hash + Debug {
        let got_hash: HashSet<T> = got.iter().cloned().collect();
        let expected_hash: HashSet<T> = expected.iter().cloned().collect();
        if got_hash != expected_hash {
            let unwanted: HashSet<&T> = got_hash.difference(&expected_hash).collect();
            let needed: HashSet<&T> = expected_hash.difference(&got_hash).collect();
            panic!("set inequality! expected len {}, got {}\nmissing {:?}\nunwanted {:?}",
                expected_hash.len(), got_hash.len(), needed, unwanted);
        }
    }

    #[test]
    fn test_first_valid_moves() {
        let new_game = GameState::new();
        let all_but_queen = vec![
            Turn::Place(Piece::new(Ant, White), ORIGIN),
            Turn::Place(Piece::new(Beetle, White), ORIGIN),
            Turn::Place(Piece::new(Grasshopper, White), ORIGIN),
            Turn::Place(Piece::new(Spider, White), ORIGIN),
        ];
        assert_set_equality(new_game.get_valid_moves(), all_but_queen);
    }

    #[test]
    fn test_make_first_move() {
        let mut new_game = GameState::new();
        let white_ant_1 = Piece::new(Ant, White);
        let turn = Turn::Place(white_ant_1, ORIGIN);
        assert!(new_game.submit_turn(turn).is_ok());
        assert_eq!(new_game.current_player, Black);
        assert_eq!(new_game.board.get(&ORIGIN), Some(&white_ant_1));
        assert_eq!(new_game.unplayed_pieces.len(), get_initial_pieces().len() - 1);
        assert_eq!(new_game.status, GameStatus::InProgress);
        assert_eq!(new_game.turns, vec![turn]);
    }

    #[test]
    fn test_make_second_move() {
        let mut game = GameState::new();
        let white_ant_1 = Piece::new(Ant, White);
        let turn_1 = Turn::Place(white_ant_1, ORIGIN);
        assert!(game.submit_turn(turn_1).is_ok());

        // 6 possible hexes * 4 possible pieces = 24 possible moves for Black
        assert_eq!(game.get_valid_moves().len(), 24);
        let black_spider_1 = Piece::new(Spider, Black);
        let west_of_origin = ORIGIN.w();
        let turn_2 = Turn::Place(black_spider_1, west_of_origin);
        assert!(game.submit_turn(turn_2).is_ok());
        assert_eq!(game.board.get(&ORIGIN), Some(&white_ant_1));
        assert_eq!(game.board.get(&west_of_origin), Some(&black_spider_1));
        assert_eq!(game.unplayed_pieces.len(), get_initial_pieces().len() - 2);
    }

    #[test]
    fn test_make_third_move() {
        let mut game = GameState::new();
        let white_ant_1 = Piece::new(Ant, White);
        let turn_1 = Turn::Place(white_ant_1, ORIGIN);
        assert!(game.submit_turn(turn_1).is_ok());
        let black_spider_1 = Piece::new(Spider, Black);
        let west_of_origin = ORIGIN.w();
        let turn_2 = Turn::Place(black_spider_1, west_of_origin);
        assert!(game.submit_turn(turn_2).is_ok());

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
            Piece { bug: Ant, owner: White, id: 2 },
            Piece::new(Beetle, White),
            Piece::new(Grasshopper, White),
            Piece::new(Queen, White),
            Piece::new(Spider, White),
        ]);
        assert_set_equality(hexes, vec![ORIGIN.ne(), ORIGIN.e(), ORIGIN.se()]);
        assert_eq!(game.get_valid_moves().len(), 15);

        let white_ant_2 = Piece { bug: Ant, owner: White, id: 2 };
        let east_of_origin = ORIGIN.e();
        let turn_3 = Turn::Place(white_ant_2, east_of_origin);
        assert!(game.submit_turn(turn_3).is_ok());
    }

    #[test]
    fn test_queen_placement_rule() {
        let mut game = GameState::new();
        assert!(game.submit_turn(Turn::Place(Piece::new(Ant, White), ORIGIN)).is_ok());
        assert!(game.submit_turn(Turn::Place(Piece::new(Ant, Black), ORIGIN.w())).is_ok());
        assert!(game.submit_turn(Turn::Place(Piece::new(Spider, White), ORIGIN.e())).is_ok());
        assert!(game.submit_turn(Turn::Place(Piece::new(Spider, Black), ORIGIN.w().w())).is_ok());
        assert!(game.submit_turn(Turn::Place(Piece::new(Beetle, White), ORIGIN.e().e())).is_ok());
        assert!(game.submit_turn(Turn::Place(Piece::new(Beetle, Black), ORIGIN.w().w().w())).is_ok());
        let mut pieces = Vec::new();
        game.get_valid_moves().iter().for_each(|m| match m {
            &Turn::Place(piece, _) => pieces.push(piece),
            _ => panic!("moves are invalid here!"),
        });
        assert_set_equality(pieces, vec![Piece::new(Queen, White)]);
        assert!(game.submit_turn(Turn::Place(Piece::new(Queen, White), ORIGIN.ne())).is_ok());
        let mut pieces = Vec::new();
        game.get_valid_moves().iter().for_each(|m| match m {
            &Turn::Place(piece, _) => pieces.push(piece),
            _ => panic!("moves are invalid here!"),
        });
        assert_set_equality(pieces, vec![Piece::new(Queen, Black)]);
        assert!(game.submit_turn(Turn::Place(Piece::new(Queen, Black), ORIGIN.w().nw())).is_ok());
    }

    #[test]
    fn test_simple_movement() {
        let mut game = GameState::new();
        // wS - bA - wA - wQ
        assert!(game.submit_turn(Turn::Place(Piece::new(Ant, White), ORIGIN)).is_ok());
        assert!(game.submit_turn(Turn::Place(Piece::new(Ant, Black), ORIGIN.w())).is_ok());
        assert!(game.submit_turn(Turn::Place(Piece::new(Queen, White), ORIGIN.e())).is_ok());
        assert!(game.submit_turn(Turn::Place(Piece::new(Spider, Black), ORIGIN.w().w())).is_ok());
        let moves: Vec<Turn> = game.get_valid_moves().iter().filter(|turn| match turn {
            Turn::Place(_, _) => false,
            Turn::Move(_, _, _) => true,
        }).cloned().collect();
        assert!(moves.len() > 0);
        assert_eq!(moves, vec![
            Turn::Move(White, Piece::new(Queen, White), ORIGIN.ne()),
            Turn::Move(White, Piece::new(Queen, White), ORIGIN.se()),
        ]);
    }

    #[test]
    fn test_make_invalid_first_move() {
        let mut new_game = GameState::new();
        let white_queen = Piece::new(Queen, White);
        let turn = Turn::Place(white_queen, ORIGIN);
        let result = new_game.submit_turn(turn);
        assert_eq!(result.err(), Some(TurnError::InvalidMove));
    }
}
