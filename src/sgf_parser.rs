use std::fs::File;
use std::io::{BufReader, BufRead};
use crate::game_state::{Turn, GameState};
use crate::game_state::Player::*;
use crate::hex::{Hex, ORIGIN};
use crate::piece::Piece;
use crate::piece::Bug::*;
use crate::parser::parse_piece_string;

fn read_sgf_file(path: &str) -> GameState {
    let file = File::open(path).unwrap();
    let mut game = GameState::new();
    let mut origin: Option<Hex> = None;
    for maybe_line in BufReader::new(file).lines() {
        let line = maybe_line.unwrap();
        if line.starts_with("; ") {
            if let Some(turn) = parse_turn(&line, &game.unplayed_pieces, &mut origin) {
                dbg!(&turn);
                assert!(game.submit_turn(turn).is_ok());
            }
        }
    }
    game
}

fn parse_turn(input: &str, unplayed_pieces: &Vec<Piece>, origin: &mut Option<Hex>) -> Option<Turn> {
    if input.contains("move") || input.contains("dropb") {
        let mut tokens = input.split_whitespace();
        let _semicolon = tokens.next();
        let _turn_no = tokens.next();
        let move_type = tokens.next();
        if move_type == Some("move") {
            let _color = tokens.next();
        }
        let piece = parse_piece_string(tokens.next().unwrap()).unwrap();
        let axial_col = tokens.next().unwrap();
        let axial_row = tokens.next().unwrap().to_string().parse::<i64>().unwrap();
        let dest = axial_to_hex(axial_col, axial_row);
        if origin.is_none() {
            *origin = Some(dest);
        }
        if unplayed_pieces.contains(&piece) {
            Some(Turn::Place(piece, dest.sub(origin.unwrap())))
        } else {
            Some(Turn::Move(piece, dest.sub(origin.unwrap())))
        }
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

    fn draw_board(game: &GameState) {
        use std::cmp;
        let pieces: Vec<(&Hex, &Piece)> = game.board.iter().collect();
        let furthest_dist = pieces.iter().fold(0, |max, (hex, _)| {
            cmp::max(max, ORIGIN.dist(**hex))
        });
        let height = furthest_dist * 2;
        let width = furthest_dist * 2;
        let row_start = height/2;
        let col_start = width/2;
        for i in -row_start..row_start {
            if i % 2 == 0 {
                for j in -col_start..col_start {
                    let x = j - (i - (i & 1))/2;
                    let z = i;
                    let y = -x - z;
                    if Hex::new(x, y, z) == ORIGIN {
                        print!(" /*\\");
                    } else {
                        print!(" / \\");
                    }
                }
                if i != 0 {
                    print!(" /");
                }
                print!("\n");
            }
            if i % 2 != 0 {
                print!("  ");
            }
            for j in -col_start..col_start {
                let x = j - (i - (i & 1))/2;
                let z = i;
                let y = -x - z;
                let lookup = Hex::new(x, y, z);
                if let Some(piece) = game.board.get(&lookup) {
                    let color = match piece.owner {
                        White => "w",
                        Black => "b",
                    };
                    let bug = match piece.bug {
                        Queen => "Q",
                        Ant => "A",
                        Spider => "S",
                        Beetle => "B",
                        Grasshopper => "G",
                    };
                    print!("|{}{}{}", color, bug, piece.id);
                } else {
                    print!("|   ");
                }
            }
            print!("|");
            print!("\n");
            if i == row_start - 1 && i % 2 != 0 {
                print!("  ");
            }
            if i % 2 == 0 || i == row_start - 1 {
                for _ in 0..width {
                    print!(" \\ /");
                }
                if i != row_start - 1 {
                    print!(" \\");
                }
                print!("\n");
            }
        }
    }
    #[test]
    fn test() {
        let _game = read_sgf_file("./test_data/HV-Dumbot0-Babamots-2020-03-29-0618.sgf");
    }
}
