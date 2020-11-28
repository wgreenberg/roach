use warp::{http::StatusCode, reply::json, Reply, Rejection};
use crate::db::{PlayerDB, DB, Range};
use crate::player::Player;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct PlayerRequest {
    name: String,
}

pub async fn list_players(db: PlayerDB) -> Result<impl Reply, Rejection> {
    Ok(json(&db.read().await.get(Range::All).await))
}

pub async fn get_player(db: PlayerDB, id: String) -> Result<impl Reply, Rejection> {
    Ok(json(&db.read().await.get(Range::All).await))
}

pub async fn create_player(db: PlayerDB, body: PlayerRequest) -> Result<impl Reply, Rejection> {
    let new_player = Player::new(body.name);
    dbg!(&new_player);
    Ok(json(&db.write().await.create(new_player).await))
}
