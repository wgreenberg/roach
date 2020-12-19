use std::io::stdin;
use clap::{Arg, App};
use hive::engine::Engine;
use hive::ai::AIOptions;
use ai::mcts::MCTSOptions;
use hive::engine::EngineOptions;

fn main() {
    let opts = App::new("cli-engine")
        .about("UHP compliant hive engine w/ AI")
        .arg(Arg::with_name("num iterations")
            .short("n")
            .long("n-iterations")
            .help("Number of iterations for the Monte Carlo tree search"))
        .arg(Arg::with_name("max depth")
            .short("d")
            .long("max-depth")
            .help("Maximum depth that MCTS should explore a game tree"))
        .get_matches();

    let max_depth: Option<usize> = opts.value_of("max depth").map(|m| m.parse().unwrap());
    let n_iterations: Option<usize> = opts.value_of("n iterations").map(|m| m.parse().unwrap());
    let mcts_opts = MCTSOptions {
        max_depth: max_depth.unwrap_or_default(),
        n_iterations: n_iterations.unwrap_or_default(),
        exploration_coefficient: Default::default(),
    };
    let mut engine = Engine::new();
    engine.options.white_ai_options = AIOptions::MonteCarloTreeSearch(mcts_opts);
    engine.options.black_ai_options = AIOptions::MonteCarloTreeSearch(mcts_opts);

    loop {
        let mut input = String::new();
        match stdin().read_line(&mut input) {
            Ok(0) => break, // EOF
            Ok(_) => println!("{}", engine.handle_command(input.trim())),
            Err(e) => eprintln!("{}", e),
        }
    }
}
