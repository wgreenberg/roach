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

    pub fn get_valid_moves(&self) -> Vec<Turn> {
        let open_hexes = match self.status {
            GameStatus::NotStarted => vec![ORIGIN],
            _ => ORIGIN.neighbors(),
        };
        self.get_unplayed_pieces().iter()
            .filter(|p| p.bug != Queen)
            .flat_map(|p| {
                open_hexes.iter().clone().map(move |hex| {
                    return Turn::Place(p.clone(), hex.clone());
                })
            })
            .collect()
    }

    fn get_unplayed_pieces(&self) -> Vec<Piece> {
        let mut lowest_ids: HashMap<Bug, u8> = HashMap::new();
        self.unplayed_pieces.iter()
            .for_each(|p| {
                let id = lowest_ids.entry(p.bug).or_insert(p.id);
                if p.id < *id {
                    *id = p.id;
                }
            });

        let placement_hexes = self.unplayed_pieces.iter()
            .filter(|p| {
                let mut valid_piece = Some(&p.id) == lowest_ids.get(&p.bug);
                valid_piece &= p.owner == self.current_player;
                if self.turns.len() < 2 {
                    valid_piece &= p.bug != Queen;
                }
                valid_piece
            });

        return placement_hexes.cloned().collect();
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

    fn assert_set_equality<T>(a: Vec<T>, b: Vec<T>)
        where T: Clone + Eq + Hash + Debug {
        let hash_a: HashSet<T> = a.iter().cloned().collect();
        let hash_b: HashSet<T> = b.iter().cloned().collect();
        assert_eq!(hash_a, hash_b);
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

    //#[test]
    fn test_make_third_move() {
        let mut game = GameState::new();
        let white_ant_1 = Piece::new(Ant, White);
        let turn_1 = Turn::Place(white_ant_1, ORIGIN);
        assert!(game.submit_turn(turn_1).is_ok());
        let black_spider_1 = Piece::new(Spider, Black);
        let west_of_origin = ORIGIN.w();
        let turn_2 = Turn::Place(black_spider_1, west_of_origin);
        assert!(game.submit_turn(turn_2).is_ok());

        // Only 3 valid hexes remain for placement, and 5 pieces = 15 moves
        dbg!(game.get_valid_moves());
        assert_eq!(game.get_valid_moves().len(), 15);
        let white_beetle_1 = Piece::new(Beetle, White);
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
