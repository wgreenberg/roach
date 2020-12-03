use serde::{Serialize, Serializer};
use crate::player::Player;
use crate::client::{Client, ClientError};
use crate::schema::{matches, match_outcomes};
use hive::game_state::{GameStatus, GameType, Color, GameState, TurnError};
use hive::parser::{parse_move_string, parse_game_string};
use hive::error::Error;
use hive::engine;
use std::convert::From;

fn serialize_game_type<S>(game_type: &GameType, s: S) -> Result<S::Ok, S::Error> where S: Serializer {
    s.serialize_str(&format!("{}", game_type))
}

#[derive(Debug, Insertable)]
#[table_name = "matches"]
pub struct MatchRowInsertable {
    pub white_player_id: i32,
    pub black_player_id: i32,
    pub game_type: String,
}

#[derive(Debug, Queryable)]
pub struct MatchRow {
    pub id: i32,
    pub white_player_id: i32,
    pub black_player_id: i32,
    pub game_type: String,
}

#[derive(PartialEq, Debug, Serialize, Clone)]
pub struct HiveMatch {
    pub id: Option<i32>,
    pub black: Player,
    pub white: Player,
    #[serde(serialize_with = "serialize_game_type")]
    pub game_type: GameType,
}

#[derive(Debug)]
pub struct MatchOutcome {
    pub status: GameStatus,
    pub comment: String,
    pub game_string: String,
    pub is_fault: bool,
}

#[derive(Queryable)]
pub struct MatchOutcomeRow {
    pub id: i32,
    pub match_id: i32,
    pub winner_id: Option<i32>,
    pub loser_id: Option<i32>,
    pub is_draw: bool,
    pub is_fault: bool,
    pub comment: String,
    pub game_string: String,
}

#[derive(Debug, Insertable)]
#[table_name = "match_outcomes"]
pub struct MatchOutcomeRowInsertable {
    pub match_id: i32,
    pub winner_id: Option<i32>,
    pub loser_id: Option<i32>,
    pub is_draw: bool,
    pub is_fault: bool,
    pub comment: String,
    pub game_string: String,
}

impl MatchOutcome {
    pub fn insertable(&self, hive_match: &HiveMatch) -> MatchOutcomeRowInsertable {
        let white_id = hive_match.white.id;
        let black_id = hive_match.black.id;
        let (winner_id, loser_id, is_draw) = match self.status {
            GameStatus::Win(Color::Black) => (Some(black_id), Some(white_id), false),
            GameStatus::Win(Color::White) => (Some(white_id), Some(black_id), false),
            GameStatus::Draw => (None, None, true),
            _ => panic!("game isn't over yet!"),
        };
        MatchOutcomeRowInsertable {
            match_id: hive_match.id.unwrap(),
            winner_id,
            loser_id,
            is_draw,
            is_fault: self.is_fault, 
            game_string: self.game_string.clone(),
            comment: self.comment.clone(),
        }
    }
}

type MatchResult = Result<MatchOutcome, MatchError>;

#[derive(PartialEq, Debug)]
pub enum MatchErrorWithBlame {
    White(MatchError),
    Black(MatchError),
    Server(MatchError),
}

#[derive(PartialEq, Debug)]
pub enum MatchError {
    InvalidState(String),
    WebsocketFailure(String),
    InvalidTurn(String),
    ProtocolError(String),
}

impl From<TurnError> for MatchError {
    fn from(err: TurnError) -> Self {
        err.into()
    }
}

impl From<ClientError> for MatchError {
    fn from(err: ClientError) -> Self {
        MatchError::WebsocketFailure(format!("{:?}", err))
    }
}

impl From<Error> for MatchError {
    fn from(err: Error) -> Self {
        match err {
            Error::ParserError(s) => MatchError::ProtocolError(format!("Failed to parse turn: {}", s)),
            Error::EngineError(s) => MatchError::InvalidTurn(format!("Invalid move: {}", s)),
        }
    }
}

fn strip_engine_output(output: &str) -> Result<&str, MatchError> {
    output.strip_suffix("\nok")
        .ok_or(MatchError::ProtocolError(format!("Invalid engine output {}", output)))
}

impl HiveMatch {
    pub fn new(p1: Player, p2: Player, game_type: GameType) -> HiveMatch {
        // TODO randomize this
        HiveMatch {
            id: None,
            white: p1,
            black: p2,
            game_type,
        }
    }

    pub fn insertable(&self) -> MatchRowInsertable {
        MatchRowInsertable {
            white_player_id: self.white.id,
            black_player_id: self.black.id,
            game_type: format!("{}", self.game_type),
        }
    }

    pub fn contains_player(&self, player: &Player) -> bool {
        self.white.id == player.id || self.black.id == player.id
    }

    pub fn create_session<T>(&mut self, b_client: T, w_client: T) -> HiveSession<T> where T: Client {
        let first_player = Color::Black; // TODO randomize this
        HiveSession {
            b_client,
            w_client,
            game: GameState::new_with_type(first_player, self.game_type),
        }
    }
}

pub struct HiveSession<T> where T: Client {
    w_client: T,
    b_client: T,
    game: GameState,
}

fn white<T>(err: T) -> MatchErrorWithBlame where T: Into<MatchError> {
    MatchErrorWithBlame::White(err.into())
}

fn black<T>(err: T) -> MatchErrorWithBlame where T: Into<MatchError> {
    MatchErrorWithBlame::White(err.into())
}

impl<T> HiveSession<T> where T: Client {
    async fn initialize(&mut self) -> Result<(), MatchErrorWithBlame> {
        let cmd = format!("newgame {}", self.game);
        let w_state = self.w_client.submit_command(cmd.clone()).await.map_err(white)?;
        self.check_game_state(w_state).map_err(white)?;
        let b_state = self.b_client.submit_command(cmd.clone()).await.map_err(black)?;
        self.check_game_state(b_state).map_err(black)?;
        Ok(())
    }

    async fn play_turn(&mut self) -> Result<(), MatchErrorWithBlame> {
        let play_cmd = match self.game.current_player {
            Color::White => {
                let bestmove_output = self.w_client.submit_command("bestmove".into())
                    .await.map_err(white)?;
                let turn_string = strip_engine_output(&bestmove_output).map_err(white)?;
                let turn = parse_move_string(turn_string, &self.game.board, &self.game.stacks).map_err(white)?;
                self.game.submit_turn(turn).map_err(white)?;
                format!("play {}", turn_string)
            },
            Color::Black => {
                let bestmove_output = self.b_client.submit_command("bestmove".into())
                    .await.map_err(black)?;
                let turn_string = strip_engine_output(&bestmove_output).map_err(black)?;
                let turn = parse_move_string(turn_string, &self.game.board, &self.game.stacks).map_err(black)?;
                self.game.submit_turn(turn).map_err(black)?;
                format!("play {}", turn_string)
            }
        };
        let w_client_state = self.w_client.submit_command(play_cmd.clone()).await.map_err(white)?;
        self.check_game_state(w_client_state).map_err(white)?;
        let b_client_state = self.b_client.submit_command(play_cmd.clone()).await.map_err(black)?;
        self.check_game_state(b_client_state).map_err(black)?;
        Ok(())
    }

    fn check_game_state(&self, output: String) -> Result<(), MatchError> {
        let game_string = strip_engine_output(&output)?;
        let received_game = parse_game_string(&game_string)?;
        if self.game != received_game {
            let err_str = format!("Invalid game state: expected {}, received {}", self.game, game_string);
            Err(MatchError::InvalidState(err_str))
        } else {
            Ok(())
        }
    }

    async fn run_game(&mut self) -> Result<GameStatus, MatchErrorWithBlame> {
        self.initialize().await?;
        while !self.game.is_over() {
            self.play_turn().await?;
        }
        Ok(self.game.status.clone())
    }

    pub async fn play(&mut self) -> MatchResult {
        let game_string = format!("{}", self.game);
        match self.run_game().await {
            Ok(status) => Ok(MatchOutcome {
                status,
                game_string,
                comment: "Game finished normally".to_string(),
                is_fault: false,
            }),
            Err(err) => {
                let (status, comment) = match err {
                    MatchErrorWithBlame::White(err) => (GameStatus::Win(Color::Black), format!("{:?}", err)),
                    MatchErrorWithBlame::Black(err) => (GameStatus::Win(Color::White), format!("{:?}", err)),
                    MatchErrorWithBlame::Server(err) => return Err(err),
                };
                Ok(MatchOutcome {
                    status,
                    game_string,
                    comment,
                    is_fault: true,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use crate::client::ClientResult;

    struct MockClient {
        requests: Vec<String>,
        responses: Vec<ClientResult>,
    }

    impl MockClient {
        fn new(mut responses: Vec<ClientResult>) -> MockClient {
            responses.reverse();
            MockClient { requests: Vec::new(), responses }
        }
    }

    #[async_trait]
    impl Client for MockClient {
        async fn submit_command(&mut self, command: String) -> ClientResult {
            self.requests.push(command);
            self.responses.pop().expect("MockClient ran out of responses!")
        }
    }

    #[tokio::test]
    async fn test_session_init() {
        let mut session = HiveSession {
            b_client: MockClient::new(vec![
                Ok("Base;NotStarted;Black[1]\nok".into()),
            ]),
            w_client: MockClient::new(vec![
                Ok("Base;NotStarted;Black[1]\nok".into()),
            ]),
            game: GameState::new(Color::Black),
        };
        assert_eq!(session.initialize().await, Ok(()));
        assert_eq!(session.b_client.requests, vec!["newgame Base;NotStarted;Black[1]"]);
        assert_eq!(session.w_client.requests, vec!["newgame Base;NotStarted;Black[1]"]);

        let mut session = HiveSession {
            b_client: MockClient::new(vec![
                Ok("Base;NotStarted;White[1]\nok".into()),
            ]),
            w_client: MockClient::new(vec![
                Ok("Base;NotStarted;Black[1]\nok".into()),
            ]),
            game: GameState::new(Color::Black),
        };
        assert_eq!(session.initialize().await.is_err(), true);
    }

    #[tokio::test]
    async fn test_session_turns() {
        let mut session = HiveSession {
            b_client: MockClient::new(vec![
                Ok("bS1\nok".into()),
                Ok("Base;InProgress;White[1];bS1\nok".into()),
            ]),
            w_client: MockClient::new(vec![
                Ok("Base;InProgress;White[1];bS1\nok".into()),
            ]),
            game: GameState::new(Color::Black),
        };
        assert_eq!(session.play_turn().await, Ok(()));
        assert_eq!(session.b_client.requests, vec!["bestmove", "play bS1"]);
        assert_eq!(session.w_client.requests, vec!["play bS1"]);

        let mut session = HiveSession {
            b_client: MockClient::new(vec![
                Ok("bS1\nok".into()),
                Ok("Base;InProgress;White[1];bS1\nok".into()),
            ]),
            w_client: MockClient::new(vec![
                Ok("Base;InProgress;White[1];bA1\nok".into()),
            ]),
            game: GameState::new(Color::Black),
        };
        assert_eq!(session.play_turn().await.is_err(), true);
        assert_eq!(session.b_client.requests, vec!["bestmove"]);
        assert_eq!(session.w_client.requests, vec!["play bS1"]);
    }
}
