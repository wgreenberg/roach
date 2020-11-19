use std::convert::From;
use crate::game_state::TurnError;

#[derive(Debug, PartialEq)]
pub enum Error {
    ParserError(String),
    EngineError(String),
}

impl From<TurnError> for Error {
    fn from(err: TurnError) -> Self {
        format!("{:?}", err).into()
    }
}

impl From<&str> for Error {
    fn from(msg: &str) -> Self {
        Error::ParserError(msg.into())
    }
}

impl From<String> for Error {
    fn from(msg: String) -> Self {
        Error::ParserError(msg)
    }
}
