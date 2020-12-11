use tokio;
use warp::Filter;
use hive::game_state::GameType;
use tokio::sync::{RwLock};
use std::sync::{Arc};
use handlebars::Handlebars;
use crate::matchmaker::Matchmaker;
use crate::err_handler::handle_rejection;
use crate::client::WebsocketClient;
#[macro_use] extern crate diesel;
use dotenv::dotenv;
use pretty_env_logger;
use std::env;

mod hive_match;
mod matchmaker;
mod player;
mod client;
mod db;
mod filters;
mod handlers;
mod err_handler;
mod schema;
mod model;

pub type AHandlebars<'a> = Arc<Handlebars<'a>>;
pub type AMatchmaker = Arc<RwLock<Matchmaker<WebsocketClient>>>;

fn initialize_handlebars<'a>(expected_templates: Vec<&str>) -> Handlebars<'a> {
    let mut hb = Handlebars::new();
    hb.register_templates_directory(".hbs", "./templates")
        .expect("failed to open handlebars templates");
    hb.set_strict_mode(true);
    for name in expected_templates {
        if hb.get_template(name).is_none() {
            panic!("Couldn't find template \"{}.hbs\" in handlebars registry", name);
        }
    }
    hb
}

#[tokio::main]
async fn main() {
    let matchmaker = Arc::new(RwLock::new(Matchmaker::new(GameType::Base)));
    dotenv().ok();
    pretty_env_logger::init();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_pool = db::create_db_pool(&db_url);
    let hb = Arc::new(initialize_handlebars(vec![
        "player", "players",
        "game", "games",
        "index",
    ]));

    let health_route = warp::path("health")
        .and(filters::with(db_pool.clone()))
        .and_then(handlers::health_handler);

    let players_route = warp::path("players")
        .and(filters::with(db_pool.clone()))
        .and(filters::with(hb.clone()))
        .and_then(handlers::get_players);

    let player = warp::path("player");
    let player_route = player
        .and(warp::get())
        .and(filters::with(db_pool.clone()))
        .and(warp::path::param())
        .and(filters::with(hb.clone()))
        .and_then(handlers::get_player)
        .or(player
            .and(warp::post())
            .and(filters::with(db_pool.clone()))
            .and(warp::body::json())
            .and_then(handlers::create_player))
        .or(player
            .and(warp::delete())
            .and(filters::with(db_pool.clone()))
            .and(warp::path::param())
            .and(filters::with_player_auth(db_pool.clone()))
            .and_then(handlers::delete_player));

    let matchmaking = warp::path!("matchmaking");
    let matchmaking_route = matchmaking
        .and(warp::post())
        .and(filters::with_player_auth(db_pool.clone()))
        .and(filters::with(matchmaker.clone()))
        .and_then(handlers::enter_matchmaking)
        .or(matchmaking
            .and(warp::get())
            .and(filters::with_player_auth(db_pool.clone()))
            .and(filters::with(matchmaker.clone()))
            .and_then(handlers::check_matchmaking));

    let game_route = warp::path!("game" / i32)
        .and(warp::get())
        .and(filters::with(db_pool.clone()))
        .and(filters::with(hb.clone()))
        .and_then(handlers::get_game);

    let games_route = warp::path!("games")
        .and(warp::get())
        .and(filters::with(db_pool.clone()))
        .and(filters::with(hb.clone()))
        .and_then(handlers::get_games);

    let play_route = warp::path!("play")
        .and(warp::ws())
        .and(filters::with(db_pool.clone()))
        .and(filters::with_player_auth(db_pool.clone()))
        .and(filters::with(matchmaker.clone()))
        .and_then(handlers::play_game);

    let index_route = warp::path::end()
        .and(filters::with(hb.clone()))
        .and_then(handlers::main_page);

    let static_route = warp::fs::dir("./static/");

    let log = warp::log("roach");

    let routes = health_route
        .or(players_route)
        .or(player_route)
        .or(matchmaking_route)
        .or(games_route)
        .or(game_route)
        .or(play_route)
        .or(index_route)
        .or(static_route)
        .recover(handle_rejection)
        .with(log)
        .with(warp::cors().allow_any_origin());

    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
}
