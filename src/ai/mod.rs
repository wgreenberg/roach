pub mod negamax;

use crate::ai::negamax::{NegamaxTree, Evaluation};
use crate::game_state::{GameState, Turn, GameStatus, Player};

const PLAYER_A: Player = Player::Black; // positive eval values
const PLAYER_B: Player = Player::White; // negative eval values

impl NegamaxTree for GameState {
    type Action = Turn;

    fn get_children(&self) -> Vec<Self> {
        self.get_valid_moves().iter()
            .map(|&turn| {
                let mut game = self.clone();
                game.submit_turn(turn).expect("failed to apply turn");
                game
            }).collect()
    }

    fn is_terminal(&self) -> bool {
        match self.status {
            GameStatus::Draw | GameStatus::Win(_) => true,
            _ => false,
        }
    }

    fn evaluate_node(&self) -> Evaluation<Self::Action> {
        let n_black_pieces = self.board.values().filter(|piece| piece.owner == Player::Black).count() as f64;
        let n_white_pieces = self.board.len() as f64 - n_black_pieces;
        Evaluation {
            node: self.get_node(),
            score: n_black_pieces - n_white_pieces,
            explanation: "piece difference".into(),
        }
    }

    fn get_node(&self) -> Self::Action {
        *self.turns.last().unwrap()
    }

    fn is_player_a_up(&self) -> bool {
        match self.current_player {
            Player::Black => true,
            Player::White => false,
        }
    }
}
