use tokio_diesel::*;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use diesel::r2d2::{Pool, ConnectionManager};
use diesel::result::Error;
use crate::schema::*;
use crate::player::Player;
use crate::hive_match::*;
use crate::model::*;

pub type DBPool = Pool<ConnectionManager<SqliteConnection>>;

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

pub async fn find_notstarted_match_for_player(db: &DBPool, player: &Player) -> Result<Option<HiveMatch>, AsyncError> {
    let result = matches::table
        .filter(matches::white_player_id.eq(player.id.unwrap())
            .or(matches::black_player_id.eq(player.id.unwrap())))
        .left_outer_join(match_outcomes::table)
        .filter(match_outcomes::id.is_null())
        .get_result_async::<(MatchRow, Option<MatchOutcomeRow>)>(&db)
        .await
        .optional()?;
    match result {
        Some((match_row, _)) => Ok(Some(match_row.into_match(db).await?)),
        None => Ok(None),
    }
}
