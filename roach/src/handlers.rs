use warp::{http::StatusCode, reply::json, Reply, Rejection};
use crate::db::DBPool;
use crate::player::Player;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct PlayerRequest {
    name: String,
}

pub async fn list_players(db: DBPool) -> Result<impl Reply, Rejection> {
    Ok(StatusCode::OK)
}

pub async fn get_player(db: DBPool, id: i64) -> Result<impl Reply, Rejection> {
    Ok(StatusCode::OK)
}

pub async fn create_player(db: DBPool, body: PlayerRequest) -> Result<impl Reply, Rejection> {
    Ok(StatusCode::OK)
}

pub async fn delete_player(db: DBPool, id: i64) -> Result<impl Reply, Rejection> {
    Ok(StatusCode::OK)
}
