use std::collections::HashSet;
use std::hash::Hash;
use std::fmt::Debug;
use crate::game_state::{Turn, GameState};
use crate::game_state::Player::*;
use crate::hex::{Hex, ORIGIN};
use crate::piece::Piece;
use crate::piece::Bug::*;

pub fn check_move(game: &mut GameState, turn: Turn) {
    assert!(game.submit_turn(turn).is_ok());
}

pub fn get_valid_movements(game: &GameState) -> Vec<Turn> {
    game.get_valid_moves().iter().filter(|turn| match turn {
        Turn::Move(_, _) => true,
        _ => false,
    }).cloned().collect()
}

pub fn draw_board(game: &GameState) {
    use std::cmp;
    let pieces: Vec<(&Hex, &Piece)> = game.board.iter().collect();
    let radius = pieces.iter().fold(8, |max, (hex, _)| {
        cmp::max(max, ORIGIN.dist(**hex))
    });
    for i in -radius..radius {
        if i % 2 == 0 {
            for j in -radius..radius {
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
        for j in -radius..radius {
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
                    Pillbug => "P",
                    Ladybug => "L",
                    Mosquito => "M",
                };
                print!("|{}{}{}", color, bug, piece.id);
            } else {
                print!("|   ");
            }
        }
        print!("|");
        print!("\n");
        if i == radius - 1 && i % 2 != 0 {
            print!("  ");
        }
        if i % 2 == 0 || i == radius - 1 {
            for _ in 0..2*radius {
                print!(" \\ /");
            }
            if i != radius - 1 {
                print!(" \\");
            }
            print!("\n");
        }
    }
}

pub fn assert_set_equality<T>(got: Vec<T>, expected: Vec<T>)
    where T: Clone + Eq + Hash + Debug {
    let got_hash: HashSet<T> = got.iter().cloned().collect();
    let expected_hash: HashSet<T> = expected.iter().cloned().collect();
    if got_hash != expected_hash {
        let unwanted: HashSet<&T> = got_hash.difference(&expected_hash).collect();
        let needed: HashSet<&T> = expected_hash.difference(&got_hash).collect();
        panic!("set inequality! expected len {}, got {}\nmissing {:?}\nunwanted {:?}",
            expected_hash.len(), got_hash.len(), needed, unwanted);
    }
}
