use diesel::pg::PgConnection;
use crate::player::Player;
use crate::model::{MatchRow, PlayerRow, PlayerRowInsertable};
use crate::hive_match::HiveMatch;
use diesel::r2d2::{Pool, ConnectionManager};
use crate::schema::{players, matches};
use tokio_diesel::*;
use diesel::prelude::*;

pub type DBPool = Pool<ConnectionManager<PgConnection>>;

type Result<T> = std::result::Result<T, AsyncError>;

pub fn create_db_pool(db_url: &str) -> DBPool {
    Pool::builder()
        .max_size(15)
        .build(ConnectionManager::new(db_url))
        .unwrap()
}

pub async fn health_check(db: &DBPool) -> Result<()> {
    diesel::sql_query("SELECT 1")
        .execute_async(&db)
        .await?;
    Ok(())
}

pub async fn find_players(db: &DBPool) -> Result<Vec<Player>> {
    Ok(players::table
        .load_async::<PlayerRow>(db)
        .await?
        .drain(..)
        .map(|row| row.into())
        .collect())
}

pub async fn find_player(db: &DBPool, player_id: i32) -> Result<Player> {
    Ok(players::table
        .filter(players::id.eq(player_id))
        .get_result_async::<PlayerRow>(db)
        .await?
        .into())
}

pub async fn find_match(db: &DBPool, match_id: i32) -> Result<HiveMatch> {
    Ok(matches::table
        .filter(matches::id.eq(match_id))
        .get_result_async::<MatchRow>(&db)
        .await?
        .into_match(&db)
        .await?)
}

async fn match_rows_into_matches(db: &DBPool, rows: Vec<MatchRow>) -> Result<Vec<HiveMatch>> {
    let mut matches = Vec::new();
    for row in rows {
        matches.push(row.into_match(db).await?);
    }
    Ok(matches)
}

pub async fn find_matches(db: &DBPool) -> Result<Vec<HiveMatch>> {
    let match_rows = matches::table
        .get_results_async::<MatchRow>(&db)
        .await?;
    match_rows_into_matches(&db, match_rows).await
}

pub async fn find_player_matches(db: &DBPool, player_id: i32) -> Result<Vec<HiveMatch>> {
    let match_rows = matches::table
        .filter(matches::white_player_id.eq(player_id).or(matches::black_player_id.eq(player_id)))
        .get_results_async::<MatchRow>(db)
        .await?;
    match_rows_into_matches(&db, match_rows).await
}

pub async fn insert_match(db: &DBPool, hive_match: HiveMatch) -> Result<()> {
    hive_match.insertable()
        .insert_into(matches::table)
        .execute_async(&db)
        .await?;
    Ok(())
}

pub async fn insert_player(db: &DBPool, player: Player) -> Result<Player> {
    let row: PlayerRowInsertable = (&player).into();
    Ok(row.insert_into(players::table)
        .get_result_async::<PlayerRow>(&db)
        .await?
        .into())
}
