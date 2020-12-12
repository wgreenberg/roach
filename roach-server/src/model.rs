use crate::db::DBPool;
use crate::hive_match::{HiveMatch, MatchOutcome};
use crate::player::Player;
use crate::schema::*;
use tokio_diesel::*;
use diesel::prelude::*;
use hive::game_state::{GameStatus, Color};
use hive::parser;
use std::convert::From;
use chrono::prelude::*;

#[derive(Debug, Insertable)]
#[table_name = "matches"]
pub struct MatchRowInsertable {
    pub white_player_id: i32,
    pub black_player_id: i32,
    pub game_type: String,
    pub winner_id: Option<i32>,
    pub loser_id: Option<i32>,
    pub is_draw: bool,
    pub is_fault: bool,
    pub time_started: DateTime<Utc>,
    pub time_finished: DateTime<Utc>,
    pub comment: String,
    pub game_string: String,
}

#[derive(Debug, Queryable)]
pub struct MatchRow {
    pub id: i32,
    pub white_player_id: i32,
    pub black_player_id: i32,
    pub game_type: String,
    pub winner_id: Option<i32>,
    pub loser_id: Option<i32>,
    pub is_draw: bool,
    pub is_fault: bool,
    pub time_started: DateTime<Utc>,
    pub time_finished: DateTime<Utc>,
    pub comment: String,
    pub game_string: String,
}

impl MatchRow {
    pub async fn into_match(&self, db: &DBPool) -> Result<HiveMatch, AsyncError> {
        let mut players = players::table
            .filter(players::id.eq(self.white_player_id).or(players::id.eq(self.black_player_id)))
            .get_results_async::<PlayerRow>(&db)
            .await?;
        assert!(players.len() == 2);
        let game_type = parser::parse_game_type(&self.game_type).expect("failed to parse game type");
        let (white, black): (Player, Player) = if players[0].id == self.white_player_id {
            (players.remove(0).into(), players.remove(0).into())
        } else {
            (players.remove(1).into(), players.remove(0).into())
        };
        let status: GameStatus = match (self.is_draw, self.winner_id) {
            (true, _) => GameStatus::Draw,
            (false, winner_id) if winner_id.unwrap() == white.id() => GameStatus::Win(Color::White),
            (false, winner_id) if winner_id.unwrap() == black.id() => GameStatus::Win(Color::Black),
            _ => panic!("invalid game status"),
        };
        let outcome = MatchOutcome {
            status,
            comment: self.comment.clone(),
            game_string: self.game_string.clone(),
            is_fault: self.is_fault,
            time_started: self.time_started,
            time_finished: self.time_finished,
        };
        Ok(HiveMatch { id: Some(self.id), white, black, game_type, outcome: Some(outcome) })
    }
}

#[derive(Insertable)]
#[table_name = "players"]
pub struct PlayerRowInsertable {
    pub name: String,
    pub elo: i32,
    pub token_hash: String,
}

impl From<&Player> for PlayerRowInsertable {
    fn from(player: &Player) -> Self {
        PlayerRowInsertable {
            name: player.name.clone(),
            elo: player.elo,
            token_hash: player.token_hash.clone(),
        }
    }
}

#[derive(Queryable)]
pub struct PlayerRow {
    pub id: i32,
    pub name: String,
    pub elo: i32,
    pub token_hash: String,
}

impl From<PlayerRow> for Player {
    fn from(row: PlayerRow) -> Player {
        Player {
            id: Some(row.id),
            name: row.name,
            elo: row.elo,
            token_hash: row.token_hash,
        }
    }
}
