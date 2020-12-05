use warp::{http::StatusCode, reply::json, Reply, Rejection};
use warp::ws::Message;
use futures::StreamExt;
use futures::FutureExt;
use serde::Serialize;
use crate::db::{DBPool, insert_match, find_notstarted_match_for_player};
use crate::player::{Player, hash_string};
use crate::matchmaker::Matchmaker;
use crate::hive_match::{HiveMatch, HiveSession, MatchOutcome};
use crate::model::{MatchRow, PlayerRow, PlayerRowInsertable};
use crate::client::WebsocketClient;
use serde::Deserialize;
use crate::schema::{players, matches, match_outcomes};
use tokio_diesel::*;
use diesel::prelude::*;
use tokio::sync::{RwLock, mpsc::Sender};
use crate::Clients;
use std::sync::{Arc};

pub type PlayerJoined = (HiveMatch, warp::filters::ws::WebSocket, Player);

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
    let players: Vec<Player> = players::table
        .load_async::<PlayerRow>(&db)
        .await
        .expect("couldn't get list of players")
        .drain(..)
        .map(|row| row.into())
        .collect();
    Ok(json(&players))
}

pub async fn get_player(db: DBPool, id: i32) -> Result<impl Reply, Rejection> {
    let player: Player = players::table
        .filter(players::id.eq(id))
        .get_result_async::<PlayerRow>(&db)
        .await
        .expect("couldn't get player")
        .into();
    Ok(json(&player))
}

pub async fn create_player(db: DBPool, body: CreatePlayerBody) -> Result<impl Reply, Rejection> {
    let (new_player, token) = Player::new(body.name);
    let row: PlayerRowInsertable = (&new_player).into();
    row.insert_into(players::table)
        .execute_async(&db)
        .await
        .expect("couldn't insert new player");
    // sqlite doesn't let us get the result of an insert, so do another fetch (since token_hash is
    // unique)
    let db_player = players::table
        .filter(players::token_hash.eq(new_player.token_hash))
        .get_result_async::<PlayerRow>(&db)
        .await
        .expect("couldn't find newly created player");
    Ok(json(&NewPlayerResponse { player: db_player.into(), token }))
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
        .get_result_async::<PlayerRow>(&db)
        .await
        .expect("couldn't get player w/ token hash");
    matchmaker.write().await
        .add_to_pool(player.into())
        .expect("player already in queue");
    Ok(StatusCode::OK)
}

pub async fn check_matchmaking(db: DBPool, token: String, matchmaker: Arc<RwLock<Matchmaker>>) -> Result<impl Reply, Rejection> {
    let player: Player = players::table
        .filter(players::token_hash.eq(hash_string(&token)))
        .get_result_async::<PlayerRow>(&db)
        .await
        .expect("couldn't get player w/ token hash")
        .into();
    let existing_match = find_notstarted_match_for_player(&db, &player).await
        .expect("couldn't check db for existing match");
    if existing_match.is_some() {
        let response = MatchmakingResponse { match_info: existing_match };
        return Ok(warp::reply::with_status(json(&response), StatusCode::OK));
    }
    if matchmaker.read().await.is_queued(&player) {
        let response = match matchmaker.write().await.find_match(player.clone()) {
            Some(hive_match) => {
                insert_match(&db, &hive_match).await.expect("couldn't insert hive match");
                let match_info = find_notstarted_match_for_player(&db, &player).await
                    .expect("couldn't get just-inserted match");
                MatchmakingResponse { match_info }
            },
            None => MatchmakingResponse { match_info: None },
        };
        Ok(warp::reply::with_status(json(&response), StatusCode::OK))
    } else {
        let response = MatchmakingResponse { match_info: None };
        Ok(warp::reply::with_status(json(&response), StatusCode::FORBIDDEN))
    }
}

pub async fn get_game(id: i32, db: DBPool) -> Result<impl Reply, Rejection> {
    let match_row = matches::table
        .filter(matches::id.eq(id))
        .get_result_async::<MatchRow>(&db)
        .await
        .expect("couldn't get game");
    let game = match_row.into_match(&db).await.expect("couldn't marshall match");
    Ok(json(&game))
}

pub async fn play_game(id: i32, ws: warp::ws::Ws, db: DBPool, token: String, clients: Clients) -> Result<impl Reply, Rejection> {
    let player: Player = players::table
        .filter(players::token_hash.eq(hash_string(&token)))
        .get_result_async::<PlayerRow>(&db)
        .await
        .expect("couldn't get player w/ token hash")
        .into();
    let match_row = matches::table
        .filter(matches::id.eq(id))
        .get_result_async::<MatchRow>(&db)
        .await
        .expect("couldn't get game");
    let mut game = match_row.into_match(&db).await.expect("couldn't marshall match");
    Ok(ws.on_upgrade(|socket| async move {
        let match_id = game.id.unwrap();

        let client = WebsocketClient::new(socket);
        let maybe_session = {
            let mut c = clients.write().await;
            match c.remove(&match_id) {
                Some(other_client) => Some(game.create_session(client, other_client)),
                None => {
                    c.insert(match_id, client);
                    None
                }
            }
        };

        if let Some(mut session) = maybe_session {
            match session.play().await {
                Ok(outcome) => {
                    println!("game {} outcome: {}, {}, {}", match_id, outcome.status, outcome.comment, outcome.game_string);
                    outcome.insertable(&game)
                        .insert_into(match_outcomes::table)
                        .execute_async(&db)
                        .await
                        .expect("couldn't insert match outcome");
                },
                Err(err) => eprintln!("hive session failed due to error: {:?}", err),
            }
        }
    }))
}
