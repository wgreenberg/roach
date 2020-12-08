use std::convert::Infallible;
use warp::{Filter, Rejection};
use crate::db::DBPool;
use crate::player::{Player, hash_string};
use crate::model::PlayerRow;
use crate::err_handler::{authentication_err, db_query_err};
use crate::schema::players;
use tokio_diesel::*;
use diesel::prelude::*;
use diesel::result::Error;

pub fn with<T>(item: T) -> impl Filter<Extract = (T,), Error = Infallible> + Clone where T: Clone + Send {
    warp::any().map(move || item.clone())
}

pub fn with_player_auth(db: DBPool) -> impl Filter<Extract = (Player,), Error = Rejection> + Clone {
    warp::filters::header::header("x-player-auth")
        .and(with(db))
        .and_then(|token: String, db: DBPool| async move {
            players::table
                .filter(players::token_hash.eq(hash_string(&token)))
                .get_result_async::<PlayerRow>(&db)
                .await
                .map_err(|err| match err {
                    AsyncError::Error(Error::NotFound) => authentication_err(err),
                    _ => db_query_err(err),
                })
                .map(|row| row.into())
        })
}
