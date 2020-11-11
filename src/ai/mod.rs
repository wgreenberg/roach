pub mod negamax;
pub mod mcts;

use rand::thread_rng;
use rand::seq::SliceRandom;
use crate::ai::negamax::{NegamaxTree, Evaluation};
use crate::ai::mcts::MonteCarloSearchable;
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

impl MonteCarloSearchable for GameState {
    type Action = Turn;

    fn simulate(&self, max_depth: usize) -> f64 {
        let mut simulation = self.clone();
        let mut rng = thread_rng();
        let mut n_turns = 0;
        let result = loop {
            if n_turns > max_depth {
                break 0.0;
            }
            match simulation.get_terminal_value() {
                Some(reward) => break reward,
                _ => {},
            }
            let choices = simulation.get_valid_moves();
            let turn = choices.choose(&mut rng);
            simulation.submit_turn_unchecked(*turn.unwrap());
            n_turns += 1;
        };
        result
    }

    fn get_terminal_value(&self) -> Option<f64> {
        match self.status {
            GameStatus::Draw => Some(0.0),
            GameStatus::Win(Player::Black) => Some(1.0),
            GameStatus::Win(Player::White) => Some(-1.0),
            _ => None
        }
    }
    fn get_possible_actions(&self) -> Vec<Self::Action> {
        self.get_valid_moves()
    }
    fn get_last_action(&self) -> Option<Self::Action> {
        self.turns.last().cloned()
    }
    fn apply_action(&self, action: Self::Action) -> Self {
        let mut clone = self.clone();
        clone.submit_turn(action).expect("failed to submit turn");
        clone
    }
}
