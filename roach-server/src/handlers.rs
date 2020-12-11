use warp::{http::StatusCode, reply::json, Reply, Rejection};
use serde::Serialize;
use crate::db::DBPool;
use crate::player::Player;
use crate::matchmaker::{Matchmaker, PollStatus, ClientStatus};
use crate::model::{MatchRow, PlayerRow, PlayerRowInsertable};
use crate::client::WebsocketClient;
use serde::Deserialize;
use crate::schema::{players, matches};
use tokio_diesel::*;
use diesel::prelude::*;
use crate::err_handler::{db_query_err, matchmaking_err};
use tokio::sync::RwLock;
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
    ready: bool,
}

pub async fn health_handler(db: DBPool) -> Result<impl Reply, Rejection> {
    diesel::sql_query("SELECT 1")
        .execute_async(&db)
        .await
        .map_err(db_query_err)?;
    Ok(StatusCode::OK)
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

pub async fn enter_matchmaking(player: Player, matchmaker: Arc<RwLock<Matchmaker<WebsocketClient>>>) -> Result<impl Reply, Rejection> {
    matchmaker.write().await
        .add_to_pool(&player.into())
        .map_err(matchmaking_err)?;
    Ok(StatusCode::OK)
}

pub async fn check_matchmaking(player: Player, matchmaker: Arc<RwLock<Matchmaker<WebsocketClient>>>) -> Result<impl Reply, Rejection> {
    let ready = match matchmaker.write().await.poll(&player).map_err(matchmaking_err)? {
        PollStatus::Ready => true,
        PollStatus::NotReady => false,
    };
    Ok(json(&MatchmakingResponse { ready }))
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

pub async fn get_games(db: DBPool) -> Result<impl Reply, Rejection> {
    let match_rows = matches::table
        .get_results_async::<MatchRow>(&db)
        .await
        .map_err(db_query_err)?;
    let mut games = Vec::new();
    for row in match_rows {
        games.push(row.into_match(&db).await.map_err(db_query_err)?);
    }
    Ok(json(&games))
}

pub async fn play_game(ws: warp::ws::Ws, db: DBPool, player: Player, matchmaker: Arc<RwLock<Matchmaker<WebsocketClient>>>) -> Result<impl Reply, Rejection> {
    if !matchmaker.read().await.has_pending_match(&player) {

    }
    Ok(ws.on_upgrade(|socket| async move {
        let client = WebsocketClient::new(socket);
        let matchmaking_result = matchmaker.write().await
            .submit_client(&player, client)
            // because we already checked for a pending match, this shouldn't happen (unless client
            // is bombarding us w/ play requests
            .expect("failed to submit client!");
        match matchmaking_result {
            ClientStatus::Pending => {},
            ClientStatus::Ready(mut hive_match, mut session) => {
                let match_info = format!("{}: black {}, white {}",
                    hive_match.game_type,
                    hive_match.black.id(),
                    hive_match.white.id());
                println!("match started ({})", &match_info);
                match session.play().await {
                    Ok(outcome) => {
                        println!("match finished ({}) {}, {}, {}",
                            &match_info,
                            outcome.status,
                            outcome.comment,
                            outcome.game_string);
                        hive_match.outcome = Some(outcome);
                        hive_match.insertable()
                            .insert_into(matches::table)
                            .execute_async(&db)
                            .await
                            .expect("couldn't insert match outcome");
                    },
                    Err(err) => eprintln!("hive session failed due to error: {:?}", err),
                }
            },
        }
    }))
}
