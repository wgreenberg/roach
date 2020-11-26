use std::collections::HashMap;
use crate::game_state::{Turn, GameState, GameType, GameStatus, Color};
use crate::game_state::Color::*;
use crate::hex::{Hex, ORIGIN};
use crate::piece::Piece;
use crate::piece::Bug::*;
use crate::error::Error;
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

pub type ParserResult<T> = Result<T, Error>;

pub fn parse_game_string(input: &str) -> ParserResult<GameState> {
    let mut tokens = input.split(";");
    let game_type = parse_game_type(tokens.next().ok_or("empty GameType")?)?;
    let game_status = parse_game_status(tokens.next().ok_or("empty GameState")?)?;
    let first_player = parse_first_player(tokens.next().ok_or("empty TurnString")?, tokens.clone().count())?;
    let mut game = GameState::new_with_type(first_player, game_type);
    for token in tokens {
        if let Err(err) = game.submit_turn(parse_move_string(token, &game.board, &game.stacks)?) {
            return Err(format!("invalid turn {}: {:?}", token, err).into());
        }
    }
    if game.status != game_status {
        return Err(format!("game status {:?} incorrect (actually {:?})", game_status, game.status).into());
    }
    Ok(game)
}

pub fn parse_first_player(input: &str, n_turns: usize) -> ParserResult<Color> {
    let mut tokens = input.split(|c| c == '[' || c == ']');
    let current_player = match tokens.next().ok_or("expected White or Black")? {
        "White" => White,
        "Black" => Black,
        c => return Err(format!("unexpected player string {}", c).into()),
    };
    if n_turns % 2 == 0 {
        Ok(current_player)
    } else {
        Ok(current_player.other())
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
        other if other.starts_with("Base+") => match other.strip_prefix("Base+") {
            Some(expansion) => {
                let p = expansion.contains("P");
                let l = expansion.contains("L");
                let m = expansion.contains("M");
                if expansion.contains(|c| !['P', 'L', 'M'].contains(&c)) {
                    Err(format!("unrecognized expansion {}", expansion).into())
                } else {
                    Ok(GameType::PLM(p, l, m))
                }
            },
            _ => Err(format!("unrecognized GameType {}", other).into()),
        },
        other => Err(format!("unrecognized GameType {}", other).into()),
    }
}

pub fn parse_move_string(input: &str, board: &HashMap<Hex, Piece>, stacks: &HashMap<Hex, Vec<Piece>>) -> ParserResult<Turn> {
    if input == "pass" {
        return Ok(Turn::Pass);
    }
    let mut tokens = input.split_whitespace();
    let piece = parse_piece_string(tokens.next().ok_or("empty input")?)?;
    if let Some(dest_str) = tokens.next() {
        let (dest_piece, direction) = match dest_str.chars().nth(0) {
            Some('w') | Some('b') => {
                if dest_str.contains(|c| c == '-' || c == '/' || c == '\\') {
                    let (piece_str, dest_str) = dest_str.split_at(dest_str.len() - 1);
                    (parse_piece_string(piece_str)?, Some(("east", dest_str)))
                } else {
                    (parse_piece_string(dest_str)?, None)
                }
            },
            _ => {
                let (dest_str, piece_str) = dest_str.split_at(1);
                (parse_piece_string(piece_str)?, Some(("west", dest_str)))
            },
        };
        let target_hex = board.iter()
            .find_map(|(&key, &value)| if value == dest_piece { Some(key) } else { None })
            .or_else(|| stacks.iter()
                .find_map(|(&key, stack)| if stack.contains(&dest_piece) { Some(key) } else { None }))
            .ok_or(format!("target piece not present on board: {:?}", piece))?;
        let dest_hex = match direction {
            Some(("east", "-")) => target_hex.e(),
            Some(("east", "/")) => target_hex.ne(),
            Some(("east", "\\")) => target_hex.se(),
            Some(("west", "-")) => target_hex.w(),
            Some(("west", "/")) => target_hex.sw(),
            Some(("west", "\\")) => target_hex.nw(),
            Some((_, c)) => return Err(format!("unrecognized direction {}", c).into()),
            None => target_hex,
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
        'L' => Ladybug,
        'M' => Mosquito,
        'P' => Pillbug,
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
        let stacks = HashMap::new();

        assert_eq!(parse_move_string("wS1", &board, &stacks), Ok(Turn::Place(Piece::new(Spider, White), ORIGIN)));
        assert_eq!(parse_move_string("wS1 wQ-", &board, &stacks), Ok(Turn::Place(Piece::new(Spider, White), ORIGIN.e())));
        assert_eq!(parse_move_string("bA1 /wQ", &board, &stacks), Ok(Turn::Move(Piece::new(Ant, Black), ORIGIN.sw())));

        assert!(parse_move_string("foo", &board, &stacks).is_err());
        assert!(parse_move_string("wwQ", &board, &stacks).is_err());
        assert!(parse_move_string("wQ foo", &board, &stacks).is_err());
        assert!(parse_move_string("wQ -bQ2", &board, &stacks).is_err());
    }

    #[test]
    fn test_stacking_moves() {
        let board: HashMap<Hex, Piece> = HashMap::from_iter(vec![
            (ORIGIN, Piece::new(Queen, White)),
            (ORIGIN.w(), Piece::new(Beetle, Black)),
        ].iter().cloned());
        let stacks = HashMap::new();

        assert_eq!(parse_move_string("bB1 wQ", &board, &stacks), Ok(Turn::Move(Piece::new(Beetle, Black), ORIGIN)));
    }

    #[test]
    fn test_moves_involving_stacks() {
        let board: HashMap<Hex, Piece> = HashMap::from_iter(vec![
            (ORIGIN, Piece::new(Beetle, White)),
            (ORIGIN.w(), Piece::new(Ant, Black)),
        ].iter().cloned());
        let stacks: HashMap<Hex, Vec<Piece>> = HashMap::from_iter(vec![
            (ORIGIN, vec![Piece::new(Queen, White)]),
        ].iter().cloned());

        assert_eq!(parse_move_string("wB1 wQ1-", &board, &stacks), Ok(Turn::Move(Piece::new(Beetle, White), ORIGIN.e())));
    }

    #[test]
    fn test_parse_game_string() {
        assert!(parse_game_string("Base;NotStarted;White[1]").is_ok());
        assert!(parse_game_string("Base;InProgress;White[3];wS1;bG1 -wS1;wA1 wS1/;bG2 /bG1").is_ok());
    }

    #[test]
    fn test_parse_game_type() {
        assert_eq!(parse_game_type("Base"), Ok(GameType::Base));
        assert_eq!(parse_game_type("Base+MP"), Ok(GameType::PLM(true, false, true)));
    }

    #[test]
    fn test_parse_first_player() {
        assert_eq!(parse_first_player("Black[1]", 0), Ok(Black));
        assert_eq!(parse_first_player("White[1]", 0), Ok(White));
        assert_eq!(parse_first_player("Black[1]", 1), Ok(White));
        assert_eq!(parse_first_player("White[1]", 1), Ok(Black));
    }
}
