use tokio_diesel::*;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use diesel::r2d2::{Pool, ConnectionManager};
use diesel::result::Error;
use crate::schema::*;
use crate::player::Player;
use crate::hive_match::*;
use crate::model::*;

pub type DBPool = Pool<ConnectionManager<PgConnection>>;

pub fn create_db_pool(db_url: &str) -> DBPool {
    Pool::builder()
        .max_size(15)
        .build(ConnectionManager::new(db_url))
        .unwrap()
}

pub async fn insert_match(db: &DBPool, hive_match: &HiveMatch) -> Result<(), AsyncError> {
    hive_match.insertable()
        .insert_into(matches::table)
        .execute_async(&db)
        .await?;
    Ok(())
}
