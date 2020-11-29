use warp::{http::StatusCode, reply::json, Reply, Rejection};
use serde::Serialize;
use crate::db::DBPool;
use crate::player::Player;
use serde::Deserialize;
use crate::schema::players;
use tokio_diesel::*;
use diesel::prelude::*;

#[derive(Deserialize)]
pub struct CreatePlayerBody {
    name: String,
}

#[derive(Serialize)]
pub struct NewPlayerResponse {
    player: Player,
    token: String,
}

pub async fn list_players(db: DBPool) -> Result<impl Reply, Rejection> {
    let players = players::table
        .load_async::<Player>(&db)
        .await
        .expect("couldn't get list of players");
    Ok(json(&players))
}

pub async fn get_player(db: DBPool, id: i32) -> Result<impl Reply, Rejection> {
    let player = players::table
        .filter(players::id.eq(id))
        .get_result_async::<Player>(&db)
        .await
        .expect("couldn't get player");
    Ok(json(&player))
}

pub async fn create_player(db: DBPool, body: CreatePlayerBody) -> Result<impl Reply, Rejection> {
    let (new_player, token) = Player::new(body.name);
    new_player.insertable()
        .insert_into(players::table)
        .execute_async(&db)
        .await
        .expect("couldn't insert new player");
    // sqlite doesn't let us get the result of an insert, so do another fetch (since token_hash is
    // unique)
    let db_player = players::table
        .filter(players::token_hash.eq(new_player.token_hash))
        .get_result_async::<Player>(&db)
        .await
        .expect("couldn't find newly created player");
    Ok(json(&NewPlayerResponse { player: db_player, token }))
}

pub async fn delete_player(db: DBPool, id: i32) -> Result<impl Reply, Rejection> {
    diesel::delete(players::table.filter(players::id.eq(id)))
        .execute_async(&db)
        .await
        .expect("couldn't delete");
    Ok(StatusCode::OK)
}
