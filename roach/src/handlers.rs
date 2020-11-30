use warp::{http::StatusCode, reply::json, Reply, Rejection};
use serde::Serialize;
use crate::db::{DBPool, insert_match, get_last_row_id, find_notstarted_match_for_player};
use crate::player::{Player, hash_string};
use crate::matchmaker::Matchmaker;
use crate::hive_match::HiveMatch;
use serde::Deserialize;
use crate::schema::players;
use tokio_diesel::*;
use diesel::prelude::*;
use tokio::sync::{RwLock};
use std::sync::{Arc};

#[derive(Deserialize)]
pub struct CreatePlayerBody {
    name: String,
}

#[derive(Serialize)]
pub struct NewPlayerResponse {
    player: Player,
    token: String,
}

#[derive(Serialize)]
pub struct MatchmakingResponse {
    match_info: Option<HiveMatch>,
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

pub async fn enter_matchmaking(db: DBPool, token: String, matchmaker: Arc<RwLock<Matchmaker>>) -> Result<impl Reply, Rejection> {
    let player = players::table
        .filter(players::token_hash.eq(hash_string(&token)))
        .get_result_async::<Player>(&db)
        .await
        .expect("couldn't get player w/ token hash");
    matchmaker.write().await.add_to_pool(player);
    Ok(StatusCode::OK)
}

pub async fn check_matchmaking(db: DBPool, token: String, matchmaker: Arc<RwLock<Matchmaker>>) -> Result<impl Reply, Rejection> {
    let player = players::table
        .filter(players::token_hash.eq(hash_string(&token)))
        .get_result_async::<Player>(&db)
        .await
        .expect("couldn't get player w/ token hash");
    let existing_match = find_notstarted_match_for_player(&db, &player).await
        .expect("couldn't check db for existing match");
    if existing_match.is_some() {
        let response = MatchmakingResponse { match_info: existing_match };
        return Ok(warp::reply::with_status(json(&response), StatusCode::OK));
    }
    if matchmaker.read().await.is_queued(player.clone()) {
        let match_info = matchmaker.write().await.find_match(player);
        if let Some(hive_match) = match_info.clone() {
            insert_match(&db, &hive_match).await.expect("couldn't insert hive match");
        }
        let response = MatchmakingResponse { match_info };
        Ok(warp::reply::with_status(json(&response), StatusCode::OK))
    } else {
        let response = MatchmakingResponse { match_info: None };
        Ok(warp::reply::with_status(json(&response), StatusCode::FORBIDDEN))
    }
}
