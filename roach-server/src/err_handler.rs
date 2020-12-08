use warp::{http::StatusCode, Reply, Rejection, reject};
use serde::Serialize;
use crate::matchmaker::MatchmakingError;
use std::convert::Infallible;
use thiserror::Error;

pub fn db_query_err(err: tokio_diesel::AsyncError) -> Rejection {
    reject::custom(ServerError::DbQueryError(err))
}

pub fn matchmaking_err(err: MatchmakingError) -> Rejection {
    reject::custom(ServerError::MatchmakingError(err))
}

pub fn authentication_err(_: tokio_diesel::AsyncError) -> Rejection {
    reject::custom(ServerError::AuthenticationError)
}

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("error executing DB query {0}")]
    DbQueryError(#[from] tokio_diesel::AsyncError),
    #[error("matchmaking error {0:?}")]
    MatchmakingError(MatchmakingError),
    #[error("authentication error")]
    AuthenticationError,
}

impl warp::reject::Reject for ServerError {}

#[derive(Serialize)]
struct ErrorMessage {
    code: u16,
    message: String,
}

pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    let code;
    let message;

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "Page not found";
    } else if let Some(_) = err.find::<warp::filters::body::BodyDeserializeError>() {
        code = StatusCode::BAD_REQUEST;
        message = "Invalid body";
    } else if let Some(e) = err.find::<ServerError>() {
        eprintln!("ServerError: {:?}", e);
        match e {
            ServerError::DbQueryError(_) => {
                code = StatusCode::BAD_REQUEST;
                message = "Could not execute request";
            },
            ServerError::MatchmakingError(err) => {
                code = StatusCode::BAD_REQUEST;
                message = match err {
                    MatchmakingError::PlayerAlreadyInQueue => "Matchmaking failed: player already in queue",
                    MatchmakingError::PlayerNotQueued => "Matchmaking failed: player not queued yet",
                };
            },
            ServerError::AuthenticationError => {
                code = StatusCode::FORBIDDEN;
                message = "Invalid authorization token";
            },
        }
    } else {
        eprintln!("unhandled rejection {:?}", err);
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "Unhandled rejection";
    }

    let json = warp::reply::json(&ErrorMessage {
        code: code.as_u16(),
        message: message.into(),
    });

    Ok(warp::reply::with_status(json, code))
}
