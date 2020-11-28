#![allow(dead_code)]
use tokio;
use warp::{http::StatusCode, Filter};
use hive::game_state::GameType;
use tokio::sync::{RwLock};
use std::sync::{Arc};
use crate::matchmaker::Matchmaker;
use crate::db::{DBPool};
use crate::player::Player;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use dotenv::dotenv;
use std::env;

mod hive_match;
mod matchmaker;
mod player;
mod client;
mod db;
mod filters;
mod handlers;

#[tokio::main]
async fn main() {
    let matchmaker = Arc::new(RwLock::new(Matchmaker::new(GameType::Base)));
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_pool = db::create_db_pool(&db_url);

    let health_route = warp::path("health")
        .map(|| StatusCode::OK);

    let players_route = warp::path("players")
        .and(filters::with_db(db_pool.clone()))
        .and_then(handlers::list_players);

    let player = warp::path("player");
    let player_route = player
        .and(warp::get())
        .and(filters::with_db(db_pool.clone()))
        .and(warp::query())
        .and_then(handlers::get_player)
        .or(player
            .and(warp::post())
            .and(filters::with_db(db_pool.clone()))
            .and(warp::body::json())
            .and_then(handlers::create_player))
        .or(player
            .and(warp::delete())
            .and(filters::with_db(db_pool.clone()))
            .and(warp::body::json())
            .and_then(handlers::delete_player))
        .or(player
            .and(warp::put())
            .and(filters::with_db(db_pool.clone()))
            .and(warp::body::json())
            .and_then(handlers::create_player));

    let routes = health_route
        .or(players_route)
        .or(player_route)
        .with(warp::cors().allow_any_origin());

    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
}
