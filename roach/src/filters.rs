use std::convert::Infallible;
use warp::{Filter, Rejection};
use crate::db::{PlayerDB};

pub fn with_player_db(db: PlayerDB) -> impl Filter<Extract = (PlayerDB,), Error = Infallible> + Clone {
    warp::any().map(move || db.clone())
}
