use std::convert::Infallible;
use warp::{Filter, Rejection};
use diesel::sqlite::SqliteConnection;
use diesel::r2d2;
use crate::db::DBPool;

pub fn with_db(db: DBPool) -> impl Filter<Extract = (DBPool,), Error = Infallible> + Clone {
    warp::any().map(move || db.clone())
}
