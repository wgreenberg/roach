use crate::piece::Piece;
use crate::piece::Bug::*;
use crate::hex::{Hex, ORIGIN};
use self::Player::*;

pub struct GameState {
    pub pieces: Vec<Piece>,
    pub turns: Vec<(Player, Turn)>,
    pub current_player: Player,
    pub status: GameStatus,
}

impl GameState {
    pub fn new() -> GameState {
        GameState {
            pieces: get_initial_pieces(),
            turns: Vec::new(),
            current_player: White,
            status: GameStatus::NotStarted,
        }
    }

    pub fn get_valid_moves(&self) -> Vec<Turn> {
        // The first move is always placing a non-queen White piece on the origin
        if self.status == GameStatus::NotStarted {
            return self.pieces.iter()
                .filter(|p| p.id == 1 && p.owner == White && p.bug != Queen)
                .map(|&piece| Turn::Place(piece, ORIGIN))
                .collect();
        }
        todo!();
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

#[derive(PartialEq)]
pub enum GameStatus {
    NotStarted,
    InProgress,
    Draw,
    Win(Player),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Turn {
    Place(Piece, Hex),
    Move(Piece, Hex),
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
        let all_but_queen = [Ant, Beetle, Grasshopper, Spider].iter()
            .map(|&bug| Piece::new(bug, White))
            .map(|p| Turn::Place(p, ORIGIN))
            .collect();
        assert_set_equality(new_game.get_valid_moves(), all_but_queen);
    }
}