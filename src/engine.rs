use crate::game_state::{GameState, Player, GameType, GameStatus, Turn};
use crate::piece::Piece;
use crate::hex::ORIGIN;
use crate::piece::Bug::*;
use crate::game_state::Player::*;
use crate::parser::*;
use crate::error::Error;
use std::convert::From;
use std::mem;
use std::fmt;

pub type EngineResult<T> = Result<T, Error>;

pub struct Engine {
    game: Option<GameState>,
}

#[derive(PartialEq, Debug)]
pub struct Output {
    text: Option<String>,
}

impl Output {
    fn empty() -> Output { Output { text: None } }
}

impl From<EngineResult<String>> for Output {
    fn from(res: EngineResult<String>) -> Self {
        match res {
            Ok(text) => Output { text: Some(text) },
            Err(err) => Output { text: Some(format!("err {:#?}", err)) },
        }
    }
}

impl ToString for Output {
    fn to_string(&self) -> String {
        match &self.text {
            Some(text) => format!("{}\nok", text),
            None => "ok".into(),
        }
    }
}

impl From<String> for Output {
    fn from(s: String) -> Self { Output { text: Some(s) } }
}

impl From<&str> for Output {
    fn from(s: &str) -> Self { Output { text: Some(s.to_string()) } }
}

fn get_piece_string(piece: &Piece) -> String {
    let color = match piece.owner {
        White => "w",
        Black => "b",
    };
    let piece_name = match piece.bug {
        Ant => "A",
        Beetle => "B",
        Ladybug => "L",
        Pillbug => "P",
        Spider => "S",
        Queen => "Q",
        Mosquito => "M",
        Grasshopper => "G",
    };
    format!("{}{}{}", color, piece_name, piece.id)
}

fn get_turn_string(turn: &Turn, game: &GameState) -> String {
    match turn {
        Turn::Move(target, hex) | Turn::Place(target, hex) => {
            let dest_neighbor = hex.neighbors().iter()
                .find_map(|neighbor| game.board.get_key_value(neighbor));
            if let Some((neighbor_hex, neighbor_piece)) = dest_neighbor {
                let from = get_piece_string(target);
                let to = get_piece_string(neighbor_piece);
                match hex.sub(*neighbor_hex) {
                    s if s == ORIGIN.w() => format!("{} -{}", from, to),
                    s if s == ORIGIN.nw() => format!("{} \\{}", from, to),
                    s if s == ORIGIN.sw() => format!("{} /{}", from, to),
                    s if s == ORIGIN.e() => format!("{} {}-", from, to),
                    s if s == ORIGIN.ne() => format!("{} {}/", from, to),
                    s if s == ORIGIN.se() => format!("{} {}\\", from, to),
                    s => panic!("invalid neighbor hex {:#?}", s),
                }
            } else {
                get_piece_string(target)
            }
        },
        Turn::Pass => "pass".to_string(),
    }
}

impl fmt::Display for GameState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let turn = format!("{}[{}]", self.current_player, (self.turn_no() + 1)/2);
        // insanely we have to replay each turn one by one to convert them into UHP notation
        match self.turns.first() {
            Some(Turn::Place(piece, _)) => {
                let mut replay = GameState::new_with_type(piece.owner, self.game_type);
                let mut turns: Vec<String> = vec![];
                for turn in &self.turns {
                    turns.push(get_turn_string(turn, &replay));
                    assert!(replay.submit_turn(turn.clone()).is_ok());
                }
                write!(f, "{};{};{};{}", self.game_type, self.status, turn, turns.join(";"))
            },
            _ => write!(f, "{};{};{}", self.game_type, self.status, turn),
        }
    }
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            White => write!(f, "White"),
            Black => write!(f, "Black"),
        }
    }
}

impl fmt::Display for GameType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GameType::Base => write!(f, "Base"),
            GameType::PLM(is_p, is_l, is_m) => {
                let p = if *is_p { "P" } else { "" };
                let l = if *is_l { "L" } else { "" };
                let m = if *is_m { "M" } else { "" };
                write!(f, "Base+{}{}{}", p, l, m)
            },
        }
    }
}

impl fmt::Display for GameStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GameStatus::NotStarted => write!(f, "NotStarted"),
            GameStatus::InProgress => write!(f, "InProgress"),
            GameStatus::Draw => write!(f, "Draw"),
            GameStatus::Win(player) => write!(f, "{}Wins", player),
        }
    }
}

impl Engine {
    pub fn new() -> Engine { Engine { game: None } }

    fn handle_newgame(&mut self, newgame: &str) -> EngineResult<String> {
        if newgame == "newgame" {
            self.game = Some(GameState::new(Black));
        } else {
            if let Some(arg) = newgame.strip_prefix("newgame ") {
                if let Ok(game_type) = parse_game_type(arg) {
                    self.game = Some(GameState::new_with_type(Black, game_type));
                } else if let Ok(game) = parse_game_string(arg) {
                    self.game = Some(game);
                }
            }
        }

        if self.game.is_none() {
            Err(format!("unrecognized newgame arg {}", newgame).into())
        } else {
            self.get_game_string()
        }
    }

    pub fn handle_command(&mut self, input: &str) -> String {
        match input {
            newgame if newgame.starts_with("newgame") => self.handle_newgame(newgame).into(),
            play if play.starts_with("play ") => self.handle_turn(play).into(),
            "pass" => self.handle_turn("play pass").into(),
            "validmoves" => self.get_valid_moves().into(),
            "undo" => self.handle_undo("undo 1").into(),
            undo if undo.starts_with("undo ") => self.handle_undo(undo).into(),
            "options" => Output::empty(),
            "info" => self.get_info(),
            _ => format!("unrecognized command {}", input).into(),
        }.to_string()
    }

    fn handle_undo(&mut self, input: &str) -> EngineResult<String> {
        let game_turns = match &self.game {
            Some(game) => game.turns.len(),
            _ => return Err(Error::EngineError("game not created yet".into())),
        };
        let n_turns = input.strip_prefix("undo ").unwrap()
            .parse::<usize>().or(Err("please specify a number"))?;
        if n_turns > game_turns {
            return Err(Error::EngineError("cannot undo more turns than exist".into()));
        }
        let old_game = mem::take(&mut self.game).unwrap();
        let mut new_turns = old_game.turns.clone();
        new_turns.truncate(new_turns.len() - n_turns);
        if let Some(Turn::Place(piece, _)) = old_game.turns.first() {
            let mut new_game = GameState::new_with_type(piece.owner, old_game.game_type);
            for turn in new_turns {
                assert!(new_game.submit_turn(turn).is_ok());
            }
            let result = Ok(format!("{}", new_game));
            self.game = Some(new_game);
            return result;
        } else {
            unreachable!();
        }
    }

    fn get_info(&self) -> Output { "id Bazinga v1.0\nMosquito;Ladybug;Pillbug".into() }

    fn get_valid_moves(&self) -> EngineResult<String> {
        match &self.game {
            Some(game) => Ok(game.get_valid_moves().iter()
                .map(|turn| get_turn_string(turn, game))
                .collect::<Vec<String>>()
                .join(";")),
            None => Err(Error::EngineError("game not created yet".into())),
        }
    }

    fn handle_turn(&mut self, input: &str) -> EngineResult<String> {
        match &mut self.game {
            Some(game) => {
                let move_string = input.strip_prefix("play ").unwrap();
                let turn = parse_move_string(move_string, &game.board, &game.stacks)?;
                game.submit_turn(turn)?;
                Ok(format!("{}", game))
            },
            None => Err(Error::EngineError("game not created yet".into())),
        }
    }

    fn get_game_string(&self) -> EngineResult<String> {
        match &self.game {
            Some(game) => Ok(format!("{}", game)),
            None => Err(Error::EngineError("game not created yet".into())),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_basics() {
        let mut engine = Engine::new();
        assert_eq!(engine.handle_command("newgame Base"), "Base;NotStarted;Black[1]\nok");
        assert_eq!(engine.handle_command("validmoves"), "bA1;bG1;bB1;bS1\nok");
        assert!(engine.handle_command("play wQ").starts_with("err"));
        assert_eq!(engine.handle_command("play bS1"), "Base;InProgress;White[1];bS1\nok");
    }

    #[test]
    fn test_newgame_inprogress() {
        let mut engine = Engine::new();
        assert_eq!(engine.handle_command("newgame Base;InProgress;White[3];wS1;bG1 -wS1;wA1 wS1/;bG2 /bG1"),
                                         "Base;InProgress;White[3];wS1;bG1 -wS1;wA1 wS1/;bG2 /bG1\nok");
    }

    #[test]
    fn test_undo() {
        let mut engine = Engine::new();
        assert_eq!(engine.handle_command("newgame Base;InProgress;White[3];wS1;bG1 -wS1;wA1 wS1/;bG2 /bG1"),
                                         "Base;InProgress;White[3];wS1;bG1 -wS1;wA1 wS1/;bG2 /bG1\nok");
        assert_eq!(engine.handle_command("undo"), "Base;InProgress;Black[2];wS1;bG1 -wS1;wA1 wS1/\nok");
        assert_eq!(engine.handle_command("undo 2"), "Base;InProgress;Black[1];wS1\nok");
    }
}
