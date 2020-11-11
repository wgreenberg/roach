use std::fs::File;
use std::collections::HashMap;
use std::io::{BufReader, BufRead};
use std::path::Path;
use crate::game_state::{Turn, GameState, Player, GameType};
use crate::hex::Hex;
use crate::piece::Piece;
use crate::parser::parse_piece_string;

pub fn read_sgf_file<P: AsRef<Path>>(path: P) -> Option<GameState> {
    let mut origin: Option<Hex> = None;
    let mut last_turn: Option<Turn> = None;
    let (actions, headers): (Vec<String>, Vec<String>) = BufReader::new(File::open(&path).unwrap())
        .lines()
        .flat_map(|l| l)
        .partition(|line| line.starts_with("; "));
    let game_type_line = headers.iter().find(|line| line.starts_with("SU[")).unwrap();
    let game_type = parse_game_type(&game_type_line).unwrap();
    // seems like all the test games start w/ white
    let mut game = GameState::new_with_type(Player::White, game_type);
    for line in actions {
        if line.starts_with("; ") {
            if line.contains("move") || line.contains("dropb") || line.contains("pass") {
                if let Some(turn) = parse_turn(&line, &game.board, &mut origin) {
                    last_turn = Some(turn);
                } else {
                    panic!("failed to parse turn {}", line);
                }
            } else if line.contains("resign") {
                return Some(game);
            } else if line.contains("done]") {
                // game terminated early
                if last_turn == None {
                    return Some(game);
                }
                assert_eq!(game.submit_turn(last_turn.unwrap()), Ok(()));
                last_turn = None;
            }
        }
    }
    Some(game)
}

fn parse_game_type(input: &str) -> Option<GameType> {
    let mut tokens = input.split(|c| c == '[' || c == ']');
    tokens.next();
    match tokens.next().unwrap() {
        "Hive" => Some(GameType::Base),
        "Hive-L" => Some(GameType::PLM(false, true, false)),
        "Hive-LM" => Some(GameType::PLM(false, true, true)),
        "Hive-PLM" => Some(GameType::PLM(true, true, true)),
        _ => None,
    }
}

fn parse_turn(input: &str, board: &HashMap<Hex, Piece>, origin: &mut Option<Hex>) -> Option<Turn> {
    if input.contains("move") || input.contains("dropb") {
        let mut tokens = input.split_whitespace();
        let _semicolon = tokens.next();
        let _turn_no = tokens.next();
        let move_type = tokens.next();
        if move_type == Some("move") || move_type == Some("pmove") {
            let _color = tokens.next();
        }
        let piece = parse_piece_string(tokens.next().unwrap()).unwrap();
        let axial_col = tokens.next().unwrap();
        let axial_row = tokens.next().unwrap().parse::<i64>().unwrap();
        let dest = axial_to_hex(axial_col, axial_row);
        // wherever the first hex is in absolute space, normalize it so everything's centered
        // around (0, 0, 0)
        if origin.is_none() {
            *origin = Some(dest);
        }
        if board.values().find(|&&board_piece| piece == board_piece).is_some() {
            Some(Turn::Move(piece, dest.sub(&origin.unwrap())))
        } else {
            Some(Turn::Place(piece, dest.sub(&origin.unwrap())))
        }
    } else if input.contains("pass") {
        Some(Turn::Pass)
    } else {
        None
    }
}

fn axial_to_hex(col: &str, row: i64) -> Hex {
    let x: i64 = "ABCDEFGHIJKLMNOPQRSTUVWXYZ".find(col).unwrap() as i64;
    let z: i64 = -row;
    let y: i64 = -x-z;
    Hex::new(x, y, z)
}

#[cfg(test)]
mod tests {
    use super::*;

    //#[test]
    fn test_sgf_games() {
        std::fs::read_dir("./test_data")
            .expect("failed to open dir")
            .flat_map(|entry| entry)
            .for_each(|entry| { read_sgf_file(entry.path()); });
    }
}
