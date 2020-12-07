#![allow(dead_code)]
use tokio;
use warp::{http::StatusCode, Filter};
use std::collections::HashMap;
use hive::game_state::GameType;
use tokio::sync::{RwLock};
use std::sync::{Arc};
use crate::matchmaker::Matchmaker;
use crate::player::Player;
use crate::db::{DBPool};
use crate::client::WebsocketClient;
use crate::err_handler::handle_rejection;
use crate::hive_match::HiveMatch;
#[macro_use] extern crate diesel;
use dotenv::dotenv;
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

pub type Clients = Arc<RwLock<HashMap<i32, WebsocketClient>>>;
pub type PendingGames = Arc<RwLock<Vec<(HiveMatch, Option<WebsocketClient>)>>>;

#[tokio::main]
async fn main() {
    let matchmaker = Arc::new(RwLock::new(Matchmaker::new(GameType::Base)));
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_pool = db::create_db_pool(&db_url);

    let health_route = warp::path("health")
        .map(|| StatusCode::OK);

    let players_route = warp::path("players")
        .and(filters::with(db_pool.clone()))
        .and_then(handlers::list_players);

    let player = warp::path("player");
    let player_route = player
        .and(warp::get())
        .and(filters::with(db_pool.clone()))
        .and(warp::path::param())
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
            .and_then(handlers::delete_player));

    let matchmaking = warp::path("matchmaking");
    let matchmaking_route = matchmaking
        .and(warp::post())
        .and(filters::with(db_pool.clone()))
        .and(filters::with_player_auth(db_pool.clone()))
        .and(filters::with(matchmaker.clone()))
        .and_then(handlers::enter_matchmaking)
        .or(matchmaking
            .and(warp::get())
            .and(filters::with(db_pool.clone()))
            .and(filters::with_player_auth(db_pool.clone()))
            .and(filters::with(matchmaker.clone()))
            .and_then(handlers::check_matchmaking));

    let game_route = warp::path!("game" / i32)
        .and(warp::get())
        .and(filters::with(db_pool.clone()))
        .and_then(handlers::get_game);

    let play_route = warp::path!("play")
        .and(warp::ws())
        .and(filters::with(db_pool.clone()))
        .and(filters::with_player_auth(db_pool.clone()))
        .and(filters::with(matchmaker.clone()))
        .and_then(handlers::play_game);

    let routes = health_route
        .or(players_route)
        .or(player_route)
        .or(matchmaking_route)
        .or(game_route)
        .or(play_route)
        .recover(handle_rejection)
        .with(warp::cors().allow_any_origin());

    let server_handle = warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
}
