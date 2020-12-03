use diesel::sqlite::SqliteConnection;
use diesel::r2d2::{Pool, ConnectionManager};
use diesel::result::Error;
use tokio_diesel::*;
use crate::schema::*;
use crate::player::Player;
use crate::hive_match::*;
use hive::parser;
use diesel::prelude::*;

pub type DBPool = Pool<ConnectionManager<SqliteConnection>>;

pub fn create_db_pool(db_url: &str) -> DBPool {
    Pool::builder()
        .max_size(15)
        .build(ConnectionManager::new(db_url))
        .unwrap()
}

no_arg_sql_function!(
    last_insert_rowid,
    diesel::sql_types::Integer,
    "Represents the SQL last_insert_row() function"
);

pub fn get_last_row_id(conn: &SqliteConnection) -> Result<i32, Error> {
    diesel::select(last_insert_rowid).get_result::<i32>(conn)
}

pub async fn insert_match(db: &DBPool, hive_match: &HiveMatch) -> Result<(), AsyncError> {
    hive_match.insertable()
        .insert_into(matches::table)
        .execute_async(&db)
        .await?;
    Ok(())
}

impl MatchRow {
    pub async fn into_match(&self, db: &DBPool) -> Result<HiveMatch, AsyncError> {
        let mut players = players::table
            .filter(players::id.eq(self.white_player_id).or(players::id.eq(self.black_player_id)))
            .get_results_async::<Player>(&db)
            .await?;
        assert!(players.len() == 2);
        let game_type = parser::parse_game_type(&self.game_type).expect("failed to parse game type");
        let (white, black) = if players[0].id == self.white_player_id {
            (players.remove(0), players.remove(0))
        } else {
            (players.remove(1), players.remove(0))
        };
        Ok(HiveMatch { id: Some(self.id), white, black, game_type })
    }
}

pub async fn find_notstarted_match_for_player(db: &DBPool, player: &Player) -> Result<Option<HiveMatch>, AsyncError> {
    let result = matches::table
        .filter(matches::white_player_id.eq(player.id)
            .or(matches::black_player_id.eq(player.id)))
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
