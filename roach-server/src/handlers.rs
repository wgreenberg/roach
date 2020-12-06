use warp::{http::StatusCode, reply::json, Reply, Rejection};
use warp::ws::Message;
use warp::reject;
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
use crate::err_handler::{db_query_err, matchmaking_err};
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
        .map_err(db_query_err)?
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
        .map_err(db_query_err)?
        .into();
    Ok(json(&player))
}

pub async fn create_player(db: DBPool, body: CreatePlayerBody) -> Result<impl Reply, Rejection> {
    let (new_player, token) = Player::new(body.name);
    let row: PlayerRowInsertable = (&new_player).into();
    let db_player = row.insert_into(players::table)
        .get_result_async::<PlayerRow>(&db)
        .await
        .map_err(db_query_err)?;
    Ok(json(&NewPlayerResponse { player: db_player.into(), token }))
}

pub async fn delete_player(db: DBPool, id: i32) -> Result<impl Reply, Rejection> {
    diesel::delete(players::table.filter(players::id.eq(id)))
        .execute_async(&db)
        .await
        .map_err(db_query_err)?;
    Ok(StatusCode::OK)
}

pub async fn enter_matchmaking(db: DBPool, player: Player, matchmaker: Arc<RwLock<Matchmaker>>) -> Result<impl Reply, Rejection> {
    matchmaker.write().await
        .add_to_pool(player.into())
        .map_err(matchmaking_err)?;
    Ok(StatusCode::OK)
}

pub async fn check_matchmaking(db: DBPool, player: Player, matchmaker: Arc<RwLock<Matchmaker>>) -> Result<impl Reply, Rejection> {
    let existing_match = find_notstarted_match_for_player(&db, &player).await
        .map_err(db_query_err)?;
    if existing_match.is_some() {
        let response = MatchmakingResponse { match_info: existing_match };
        return Ok(warp::reply::with_status(json(&response), StatusCode::OK));
    }
    let matchmaking_result = matchmaker.write().await.find_match(player.clone())
        .map_err(matchmaking_err)?;
    let response = match matchmaking_result {
        Some(hive_match) => {
            insert_match(&db, &hive_match).await.map_err(db_query_err)?;
            let match_info = find_notstarted_match_for_player(&db, &player).await
                .map_err(db_query_err)?;
            MatchmakingResponse { match_info }
        },
        None => MatchmakingResponse { match_info: None },
    };
    Ok(warp::reply::with_status(json(&response), StatusCode::OK))
}

pub async fn get_game(id: i32, db: DBPool) -> Result<impl Reply, Rejection> {
    let match_row = matches::table
        .filter(matches::id.eq(id))
        .get_result_async::<MatchRow>(&db)
        .await
        .map_err(db_query_err)?;
    let game = match_row.into_match(&db).await.map_err(db_query_err)?;
    Ok(json(&game))
}

pub async fn play_game(id: i32, ws: warp::ws::Ws, db: DBPool, player: Player, clients: Clients) -> Result<impl Reply, Rejection> {
    let match_row = matches::table
        .filter(matches::id.eq(id))
        .get_result_async::<MatchRow>(&db)
        .await
        .map_err(db_query_err)?;
    let mut game = match_row.into_match(&db).await.map_err(db_query_err)?;
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
