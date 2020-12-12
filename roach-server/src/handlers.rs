use warp::{http::StatusCode, reply::json, Reply, Rejection};
use serde_json::json;
use crate::{AHandlebars, AMatchmaker};
use crate::db::*;
use crate::player::Player;
use crate::matchmaker::{PollStatus, ClientStatus};
use crate::client::WebsocketClient;
use serde::Deserialize;
use warp::ws::Ws;
use crate::err_handler::{db_query_err, matchmaking_err, template_err};

#[derive(Deserialize)]
pub struct CreatePlayerBody {
    name: String,
}

type Result<T> = std::result::Result<T, Rejection>;

pub async fn health_handler(db: DBPool) -> Result<impl Reply> {
    health_check(&db).await.map_err(db_query_err)?;
    Ok(StatusCode::OK)
}

pub async fn get_players(db: DBPool, hb: AHandlebars<'_>) -> Result<impl Reply> {
    let html = hb.render("players", &json!({
        "title": "Players",
        "players": find_players(&db).await.map_err(db_query_err)?,
    })).map_err(template_err)?;
    Ok(warp::reply::html(html))
}

pub async fn get_player(db: DBPool, id: i32, hb: AHandlebars<'_>) -> Result<impl Reply> {
    let player: Player = find_player(&db, id).await.map_err(db_query_err)?;
    let games = find_player_matches(&db, id).await.map_err(db_query_err)?;
    let stats = find_player_stats(&db, id).await.map_err(db_query_err)?;
    let html = hb.render("player", &json!({
        "title": format!("Player {}: {}", id, player.name),
        "player": player,
        "stats": stats,
        "games": games,
    })).map_err(template_err)?;
    Ok(warp::reply::html(html))
}

pub async fn main_page(hb: AHandlebars<'_>) -> Result<impl Reply> {
    Ok(warp::reply::html(hb.render("index", &json!({
        "title": "Ranked Online Arena for Computer Hive",
    })).map_err(template_err)?))
}

pub async fn create_player(db: DBPool, body: CreatePlayerBody) -> Result<impl Reply> {
    let (new_player, token) = Player::new(body.name);
    let db_player = insert_player(&db, new_player).await.map_err(db_query_err)?;
    Ok(json(&json!({
        "player": db_player,
        "token": token,
    })))
}

pub async fn enter_matchmaking(player: Player, matchmaker: AMatchmaker) -> Result<impl Reply> {
    matchmaker.write().await
        .add_to_pool(&player.into())
        .map_err(matchmaking_err)?;
    Ok(StatusCode::OK)
}

pub async fn check_matchmaking(player: Player, matchmaker: AMatchmaker) -> Result<impl Reply> {
    let ready = match matchmaker.write().await.poll(&player).map_err(matchmaking_err)? {
        PollStatus::Ready => true,
        PollStatus::NotReady => false,
    };
    Ok(json(&json!({ "ready": ready })))
}

pub async fn get_game(id: i32, db: DBPool, hb: AHandlebars<'_>) -> Result<impl Reply> {
    let game = find_match(&db, id).await.map_err(db_query_err)?;
    let html = hb.render("game", &json!({
        "title": format!("Game {}: {} vs {}", id, game.black.name, game.white.name),
        "game": game,
    })).map_err(template_err)?;
    Ok(warp::reply::html(html))
}

pub async fn get_games(db: DBPool, hb: AHandlebars<'_>) -> Result<impl Reply> {
    let html = hb.render("games", &json!({
        "title": "Games",
        "games": find_matches(&db).await.map_err(db_query_err)?,
    })).map_err(template_err)?;
    Ok(warp::reply::html(html))
}

pub async fn play_game(ws: Ws, db: DBPool, player: Player, matchmaker: AMatchmaker) -> Result<Box<dyn Reply>> {
    if !matchmaker.read().await.has_pending_match(&player) {
        return Ok(Box::new(StatusCode::FORBIDDEN));
    }
    Ok(Box::new(ws.on_upgrade(|socket| async move {
        let client = WebsocketClient::new(socket);
        let matchmaking_result = matchmaker.write().await
            .submit_client(&player, client)
            // because we already checked for a pending match, this shouldn't happen (unless client
            // is bombarding us w/ play requests
            .expect("failed to submit client!");
        match matchmaking_result {
            ClientStatus::Pending => {}, // this player's the first to show up, so we wait
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
                        insert_match(&db, hive_match)
                            .await
                            .expect("couldn't insert match outcome");
                    },
                    Err(err) => eprintln!("hive session failed due to error: {:?}", err),
                }
            },
        }
    })))
}
