use rand::thread_rng;
use rand::seq::SliceRandom;
use ai::negamax::{NegamaxTree, Evaluation};
use ai::mcts::{MonteCarloSearchable, MCTSOptions};
use crate::game_state::{GameState, Turn, GameStatus, Player};
use crate::hex::Hex;
use crate::piece::{Bug, Piece};

const PLAYER_A: Player = Player::Black; // positive eval values
const PLAYER_B: Player = Player::White; // negative eval values

#[derive(Copy, Clone, Debug)]
pub enum AIOptions {
    Negamax(usize),
    MonteCarloTreeSearch(MCTSOptions),
    Random,
}

pub trait AIPlayer {
    fn find_best_move(&self, options: AIOptions) -> Turn;
}

impl AIPlayer for GameState {
    fn find_best_move(&self, options: AIOptions) -> Turn {
        match options {
            AIOptions::Negamax(depth) => self.find_best_action_negamax(depth),
            AIOptions::MonteCarloTreeSearch(opts) => self.find_best_action_mcts(opts),
            AIOptions::Random => {
                let mut rng = thread_rng();
                *self.get_valid_moves().choose(&mut rng).unwrap()
            },
        }
    }
}

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
    type Player = Player;

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

    fn current_player(&self) -> Self::Player {
        self.current_player
    }

    fn get_terminal_value(&self, player: Player) -> Option<bool> {
        match self.status {
            GameStatus::Win(winner) => Some(winner == player),
            GameStatus::Draw => Some(false),
            _ => None
        }
    }

    fn get_possible_actions(&self) -> Vec<Self::Action> {
        self.get_valid_moves()
    }

    fn get_last_action(&self) -> Option<Self::Action> {
        self.turns.last().cloned()
    }

    // we assume all turns submitted by the AI are valid
    fn apply_action(&mut self, action: Self::Action) {
        self.submit_turn_unchecked(action);
    }

    fn describe_action(&self, action: Self::Action) -> String {
        crate::engine::get_turn_string(&action, self)
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
