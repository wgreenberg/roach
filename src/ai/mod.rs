pub mod negamax;
pub mod mcts;

use rand::thread_rng;
use rand::seq::SliceRandom;
use crate::ai::negamax::{NegamaxTree, Evaluation};
use crate::ai::mcts::MonteCarloSearchable;
use crate::game_state::{GameState, Turn, GameStatus, Player};
use crate::hex::Hex;
use crate::piece::{Bug, Piece};

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

fn get_queen_and_liberties(game: &GameState, player: Player) -> Option<(Hex, usize)> {
    if let Some(queen) = game.get_hex_for_piece(&Piece::new(Bug::Queen, player)) {
        let n_neighbors = queen.neighbors().iter()
            .filter(|hex| game.board.contains_key(hex)).count();
        Some((queen, n_neighbors))
    } else {
        None
    }
}

fn score_turn(game: &GameState, turn: &Turn) -> f64 {
    let mut score = 0.0;
    if let Turn::Move(piece, to) = turn {
        let from = game.get_hex_for_piece(&piece).unwrap();
        if let Some((queen_hex, queen_liberties)) = get_queen_and_liberties(game, Player::Black) {
            let modifier = queen_liberties as f64;
            // if we're moving to (or on top of) the black queen, that's good for white
            if queen_hex == *to || queen_hex.neighbors().contains(to) {
                score -= modifier;
            }
            if queen_hex.neighbors().contains(&from) {
                score += modifier;
            }
        }
        if let Some((queen_hex, queen_liberties)) = get_queen_and_liberties(game, Player::White) {
            let modifier = queen_liberties as f64;
            // if we're moving to (or on top of) the white queen, that's good for black
            if queen_hex == *to || queen_hex.neighbors().contains(to) {
                score += modifier;
            }
            if queen_hex.neighbors().contains(&from) {
                score -= modifier;
            }
        }
    }
    score
}

impl MonteCarloSearchable for GameState {
    type Action = Turn;

    fn simulate(&self, max_depth: usize) -> f64 {
        let mut simulation = self.clone();
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
            let turn = simulation.select_action(&choices);
            simulation.submit_turn_unchecked(turn);
            n_turns += 1;
        };
        result
    }

    fn select_action(&self, actions: &Vec<Self::Action>) -> Self::Action {
        let (first, rest) = actions.split_first().unwrap();
        let mut best_score = score_turn(self, first);
        let mut best_turn = first;
        for turn in rest {
            let score = score_turn(self, &turn);
            let is_better = match self.current_player {
                Player::Black => score > best_score,
                Player::White => score < best_score,
            };
            if is_better {
                best_score = score;
                best_turn = turn;
            }
        }
        if best_score == 0.0 {
            let mut rng = thread_rng();
            *actions.choose(&mut rng).unwrap()
        } else {
            *best_turn
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::play_and_verify;

    #[test]
    fn test_select_move() {
        let mut game = GameState::new(Player::White);
        // setup a mate-in-one situation for black
        play_and_verify(&mut game, vec![
            "wA1",
            "bA1 -wA1",
            "wQ wA1/",
            "bQ \\bA1",
            "wS wA1\\",
            "bA2 -bA1",
            "wS1 wQ1/",
            "bQ -wQ",
            "wG1 wQ\\",
            "bS1 bA2\\",
            "wB1 wQ-",
        ]);
        let winning_move = Turn::Move(Piece {
            bug: Bug::Ant,
            owner: Player::Black,
            id: 2
        }, Hex::new(1, 1, -2));
        assert_eq!(game.select_action(&game.get_possible_actions()), winning_move);
        // do something irrelevant
        play_and_verify(&mut game, vec!["bS2 -bS1"]);
        // the best move for white is moving wS1 from wQ to bQ
        let saving_move = Turn::Move(Piece::new(Bug::Spider, Player::White), Hex::new(-1, 2, -1));
        assert_eq!(game.select_action(&game.get_possible_actions()), saving_move);
    }
}
