use std::collections::HashMap;
use crate::game_state::{Turn, GameState, GameType, GameStatus};
use crate::game_state::Player::*;
use crate::hex::{Hex, ORIGIN};
use crate::piece::Piece;
use crate::piece::Bug::*;
use std::convert::From;
use std::result::Result;

// newgame -> GameString
//   newgame
//   newgame GameTypeString
//   newgame GameString

// play MoveString -> GameString

// pass -> GameString (same as "play pass")

// validmoves -> [MoveString]

// undo [MoveString] -> GameString

// options -> Ok

// info -> InfoString

// InfoString: name/version of the engine, plus expansion capabilities (separated by a newline)

// GameString: complete state of the game
//   GameTypeString;GameStateString;TurnString[;MoveString[;...]]

// GameTypeString: expansion pieces (if any)
//   Base[+[MLP]]

// GameStateString: whether the game is in progress or not

// TurnString: which side's turn it is, as well as turn number (game number / 2)
//   (White|Black)[n]

// MoveString
//   (Piece[ PieceLocation]|pass) e.g. "wS1" or "bS1 wS1/"

#[derive(Debug, PartialEq)]
pub enum Error {
    ParserError(String),
}

pub type ParserResult<T> = Result<T, Error>;

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

pub fn parse_game_string(input: &str) -> ParserResult<GameState> {
    let mut tokens = input.split(";");
    let game_type = parse_game_type(tokens.next().ok_or("empty GameType")?)?;
    let game_status = parse_game_status(tokens.next().ok_or("empty GameState")?)?;
    let turn_no = parse_game_turn(tokens.next().ok_or("empty TurnString")?)?;
    let mut game = GameState::new_with_type(White, game_type);
    for token in tokens {
        if let Err(err) = game.submit_turn(parse_move_string(token, &game.board)?) {
            return Err(format!("invalid turn {}: {:?}", token, err).into());
        }
    }
    if game.turn_no() != turn_no {
        return Err(format!("turn incorrect (actually {})", game.turn_no()).into());
    }
    if game.status != game_status {
        return Err(format!("game status {:?} incorrect (actually {:?})", game_status, game.status).into());
    }
    Ok(game)
}

pub fn parse_game_turn(input: &str) -> ParserResult<usize> {
    let mut tokens = input.split(|c| c == '[' || c == ']');
    let player = tokens.next().ok_or("expected White or Black")?;
    let num = tokens.next().ok_or("expected White or Black")?.to_string()
        .parse::<usize>().or(Err("failed to parse turn number"))?;
    match (player, num) {
        ("White", n) => Ok(n*2 - 1),
        ("Black", n) => Ok(n*2),
        (c, _) => Err(format!("unexpected player string {}", c).into()),
    }
}

pub fn parse_game_status(input: &str) -> ParserResult<GameStatus> {
    match input {
        "NotStarted" => Ok(GameStatus::NotStarted),
        "InProgress" => Ok(GameStatus::InProgress),
        "Draw" => Ok(GameStatus::Draw),
        "WhiteWins" => Ok(GameStatus::Win(White)),
        "BlackWins" => Ok(GameStatus::Win(Black)),
        c => Err(format!("unrecognized GameStatus {}", c).into()),
    }
}

pub fn parse_game_type(input: &str) -> ParserResult<GameType> {
    match input {
        "Base" => Ok(GameType::Base),
        other => Err(format!("unrecognized GameType {}", other).into()),
    }
}

pub fn parse_move_string(input: &str, board: &HashMap<Hex, Piece>) -> ParserResult<Turn> {
    let mut tokens = input.split_whitespace();
    let piece = parse_piece_string(tokens.next().ok_or("empty input")?)?;
    if let Some(dest_str) = tokens.next() {
        let (dest_piece, dir, side) = match dest_str.chars().nth(0) {
            Some('w') | Some('b') => {
                let (piece_str, dest_str) = dest_str.split_at(dest_str.len() - 1);
                (parse_piece_string(piece_str)?, dest_str, "east")
            },
            _ => {
                let (dest_str, piece_str) = dest_str.split_at(1);
                (parse_piece_string(piece_str)?, dest_str, "west")
            },
        };
        let target_hex = board.iter()
            .find_map(|(&key, &value)| if value == dest_piece { Some(key) } else { None })
            .ok_or("target piece not present on board")?;
        let dest_hex = match (side, dir) {
            ("east", "-") => target_hex.e(),
            ("east", "/") => target_hex.ne(),
            ("east", "\\") => target_hex.se(),
            ("west", "-") => target_hex.w(),
            ("west", "/") => target_hex.sw(),
            ("west", "\\") => target_hex.nw(),
            (_, c) => return Err(format!("unrecognized direction {}", c).into()),
        };
        if board.values().find(|&&board_piece| piece == board_piece).is_some() {
            Ok(Turn::Move(piece, dest_hex))
        } else {
            Ok(Turn::Place(piece, dest_hex))
        }
    } else {
        Ok(Turn::Place(piece, ORIGIN))
    }
}

pub fn parse_piece_string(input: &str) -> ParserResult<Piece> {
    let mut chars = input.chars();
    let player = match chars.next().ok_or("empty piece string")? {
        'w' => White,
        'b' => Black,
        c => return Err(format!("unknown player {}", c).into()),
    };
    let bug = match chars.next().ok_or("no bug character found")? {
        'A' => Ant,
        'B' => Beetle,
        'G' => Grasshopper,
        'Q' => Queen,
        'S' => Spider,
        c => return Err(format!("unknown piece {}", c).into()),
    };
    if let Some(id_char) = chars.next() {
        let id = id_char.to_string().parse::<u8>().or(Err("failed to parse id"))?;
        Ok(Piece { owner: player, bug, id })
    } else {
        Ok(Piece::new(bug, player))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::iter::FromIterator;

    #[test]
    fn test_parse_move_string() {
        let board: HashMap<Hex, Piece> = HashMap::from_iter(vec![
            (ORIGIN, Piece::new(Queen, White)),
            (ORIGIN.w(), Piece::new(Ant, Black)),
        ].iter().cloned());

        assert_eq!(parse_move_string("wS1", &board), Ok(Turn::Place(Piece::new(Spider, White), ORIGIN)));
        assert_eq!(parse_move_string("wS1 wQ-", &board), Ok(Turn::Place(Piece::new(Spider, White), ORIGIN.e())));
        assert_eq!(parse_move_string("bA1 /wQ", &board), Ok(Turn::Move(Piece::new(Ant, Black), ORIGIN.sw())));

        assert!(parse_move_string("foo", &board).is_err());
        assert!(parse_move_string("wwQ", &board).is_err());
        assert!(parse_move_string("wQ foo", &board).is_err());
        assert!(parse_move_string("wQ -bQ2", &board).is_err());
    }

    #[test]
    fn test_parse_game_string() {
        assert!(parse_game_string("Base;NotStarted;White[1]").is_ok());
        assert!(parse_game_string("Base;InProgress;White[3];wS1;bG1 -wS1;wA1 wS1/;bG2 /bG1").is_ok());
    }
}
