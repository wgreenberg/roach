use warp::{http::StatusCode, reply::json, Reply, Rejection};
use warp::ws::Message;
use futures::StreamExt;
use futures::FutureExt;
use serde::Serialize;
use crate::db::{DBPool, insert_match, get_last_row_id, find_notstarted_match_for_player};
use crate::player::{Player, hash_string};
use crate::matchmaker::Matchmaker;
use crate::hive_match::{HiveMatch, MatchRow, HiveSession};
use crate::client::WebsocketClient;
use serde::Deserialize;
use crate::schema::{players, matches};
use tokio_diesel::*;
use diesel::prelude::*;
use tokio::sync::{RwLock, mpsc::Sender};
use crate::{PlayerJoined, Clients};
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

async fn run_game(mut session: HiveSession<WebsocketClient>, db: DBPool) {
    let result = session.play().await;
    // TODO figure out how to close client websocket connections after the game
    dbg!(result);
}

async fn player_joined(clients: Clients, p: PlayerJoined, db: DBPool) {
    let (mut game, socket, player) = p;
    let match_id = game.id.unwrap();
    let (client_ws_sender, mut client_ws_rcv) = socket.split();
    let (client_sender, ws_sender) = tokio::sync::mpsc::unbounded_channel::<String>();
    let (ws_reciever, client_reciever) = tokio::sync::mpsc::unbounded_channel::<String>();

    let client = WebsocketClient {
        tx: client_sender,
        rx: client_reciever,
    };

    {
        let mut c = clients.write().await;
        if let Some(other_client) = c.remove(&match_id) {
            let session = game.create_session(client, other_client);
            tokio::task::spawn(run_game(session, db));
        } else {
            c.insert(match_id, client);
        }
    }

    tokio::task::spawn(ws_sender.map(|s| Ok(Message::text(s)))
        .forward(client_ws_sender).map(|result| {
            if let Err(e) = result {
                eprintln!("error sending websocket msg: {}", e);
            }
        }));

    while let Some(result) = client_ws_rcv.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("error receiving ws message): {}", e);
                break;
            }
        };
        ws_reciever.send(msg.to_str().unwrap().to_string());
    }
    println!("player {} disconnected", player.id);
}

pub async fn play_game(id: i32, ws: warp::ws::Ws, db: DBPool, token: String, clients: Clients) -> Result<impl Reply, Rejection> {
    let player = players::table
        .filter(players::token_hash.eq(hash_string(&token)))
        .get_result_async::<Player>(&db)
        .await
        .expect("couldn't get player w/ token hash");
    let match_row = matches::table
        .filter(matches::id.eq(id))
        .get_result_async::<MatchRow>(&db)
        .await
        .expect("couldn't get game");
    let game = match_row.into_match(&db).await.expect("couldn't marshall match");
    Ok(ws.on_upgrade(move |socket| player_joined(clients, (game, socket, player), db)))
}
