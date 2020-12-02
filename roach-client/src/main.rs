use clap::{Arg, App, SubCommand, AppSettings};
use std::env;
use std::io::stdin;

mod process;
mod engine;
mod matchmaking;

use crate::engine::{EngineType, UHPCompliant, get_engine};
use crate::matchmaking::MatchmakingClient;

#[tokio::main]
async fn main() {
    let opts = App::new("roach-client")
        .about("Enables your Hive-playing AI to play games on the roach server")
        .arg(Arg::with_name("bin")
            .short("b")
            .long("bin")
            .value_name("FILE")
            .required(true)
            .help("Path to your Hive AI binary")
            .takes_value(true))
        .arg(Arg::with_name("engine type")
            .short("e")
            .long("engine-type")
            .possible_values(&["uhp", "simple"])
            .value_name("ENGINE_TYPE")
            .default_value("uhp"))
        .subcommand(SubCommand::with_name("engine")
            .about("run a local UHP compliant engine"))
        .subcommand(SubCommand::with_name("matchmaking")
            .about("run a local UHP compliant engine")
                .arg(Arg::with_name("roach server")
                    .short("s")
                    .long("server")
                    .takes_value(true)
                    .help("domain of the roach server to play against")
                    .value_name("SERVER"))
                .arg(Arg::with_name("player token")
                    .short("t")
                    .long("token")
                    .takes_value(true)
                    .help("AI player API token")
                    .value_name("TOKEN")))
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .get_matches();

    let ai_path: String = opts.value_of("bin").unwrap().into();
    let engine_type = match opts.value_of("engine type").unwrap() {
        "uhp" => EngineType::UHP,
        "simple" => EngineType::Simple,
        t => panic!("unrecognized engine type {}", t),
    };
    match opts.subcommand_name() {
        Some("engine") => engine(ai_path, engine_type).await,
        Some("matchmaking") => {
            let matchmaking_opts = opts.subcommand_matches("matchmaking").unwrap();
            let player_token = matchmaking_opts.value_of("player token")
                .map(String::from)
                .or(env::var("PLAYER_TOKEN").ok())
                .expect("please provide a player token (either as an arg or PLAYER_TOKEN env var");
            let roach_server = matchmaking_opts.value_of("roach server")
                .unwrap_or("localhost:8000")
                .to_string();
            matchmaking(ai_path, engine_type, player_token, roach_server).await
        },
        _ => panic!("please specify a valid subcommand"),
    }
}

async fn engine(ai_path: String, engine_type: EngineType) {
    let mut engine = get_engine(ai_path, engine_type);
    loop {
        let mut input = String::new();
        match stdin().read_line(&mut input) {
            Ok(0) => break, // EOF
            Ok(_) => println!("{}", engine.handle_command(input.trim()).await),
            Err(e) => eprintln!("{}", e),
        }
    }
}

async fn matchmaking(ai_path: String, engine_type: EngineType, roach_server: String, player_token: String) {
    let engine = get_engine(ai_path, engine_type);
    let client = MatchmakingClient::new(roach_server, player_token);
    let res = client.enter_matchmaking().await.expect("couldn't enter matchmaking");
    let match_id = client.wait_for_match().await.expect("couldn't poll for match");
    client.play_match(match_id, engine).await;
    dbg!(res);
}
