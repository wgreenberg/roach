use std::io::stdin;
use clap::{Arg, App};
use hive::engine::Engine;
use hive::ai::AIOptions;
use ai::mcts::MCTSOptions;

fn main() {
    let opts = App::new("cli-engine")
        .about("UHP compliant hive engine w/ AI")
        .arg(Arg::with_name("num iterations")
            .short("n")
            .long("num-iterations")
            .takes_value(true)
            .help("Number of iterations for the Monte Carlo tree search"))
        .arg(Arg::with_name("max depth")
            .short("d")
            .long("max-depth")
            .takes_value(true)
            .help("Maximum depth that MCTS should explore a game tree"))
        .get_matches();

    let mut mcts_opts: MCTSOptions = Default::default();
    if let Some(depth) = opts.value_of("max depth") {
        mcts_opts.max_depth = depth.parse().unwrap();
    }
    if let Some(iter) = opts.value_of("num iterations") {
        mcts_opts.n_iterations = iter.parse().unwrap();
    }
    dbg!(&mcts_opts);
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
