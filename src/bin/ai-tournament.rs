use hive::game_state::GameStatus::*;
use hive::game_state::Player::*;
use hive::game_state::GameState;
use hive::ai::{AIOptions, AIPlayer};
use hive::ai::mcts::MCTSOptions;

fn main() {
    let mut mcts_wins = 0;
    let mut random_wins = 0;
    let mcts_options = AIOptions::MonteCarloTreeSearch(MCTSOptions::default());
    let random_options = AIOptions::Random;
    for i in 0..10 {
        let mut game = GameState::new(Black);
        while game.status == NotStarted || game.status == InProgress {
            let opts = match game.current_player {
                Black => mcts_options,
                White => random_options,
            };
            game.submit_turn_unchecked(game.find_best_move(opts));
            hive::test_utils::draw_board(&game);
        }
        match game.status {
            Win(Black) => mcts_wins += 1,
            Win(White) => random_wins += 1,
            _ => {},
        }
        println!("game {}: mcts {}, random {}", i, mcts_wins, random_wins);
    }
    dbg!(mcts_wins, random_wins);
}
