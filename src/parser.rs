use std::collections::HashMap;
use crate::game_state::{Turn, GameState};
use crate::game_state::Player::*;
use crate::hex::{Hex, ORIGIN};
use crate::piece::Piece;
use crate::piece::Bug::*;

struct Parser;

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

impl Parser {
    fn new() -> Parser { Parser {} }
    fn parse_move_string(&self, input: &str, board: &HashMap<Hex, Piece>) -> Option<Turn> {
        let mut tokens = input.split_whitespace();
        let piece = self.parse_piece_string(tokens.next()?)?;
        if let Some(dest_str) = tokens.next() {
            let (dest_piece, dir, side) = match dest_str.chars().nth(0) {
                Some('w') | Some('b') => {
                    let (piece_str, dest_str) = dest_str.split_at(dest_str.len() - 1);
                    (self.parse_piece_string(piece_str)?, dest_str, "right")
                },
                _ => {
                    let (dest_str, piece_str) = dest_str.split_at(1);
                    (self.parse_piece_string(piece_str)?, dest_str, "left")
                },
            };
            let target_hex = board.iter()
                .find_map(|(&key, &value)| if value == dest_piece { Some(key) } else { None })?;
            let dest_hex = match (side, dir) {
                ("right", "-") => target_hex.e(),
                ("right", "/") => target_hex.ne(),
                ("right", "\\") => target_hex.se(),
                ("left", "-") => target_hex.w(),
                ("left", "/") => target_hex.sw(),
                ("left", "\\") => target_hex.nw(),
                _ => return None,
            };
            if board.values().find(|&&board_piece| piece == board_piece).is_some() {
                Some(Turn::Move(piece, dest_hex))
            } else {
                Some(Turn::Place(piece, dest_hex))
            }
        } else {
            Some(Turn::Place(piece, ORIGIN))
        }
    }

    fn parse_piece_string(&self, input: &str) -> Option<Piece> {
        let mut chars = input.chars();
        let player = match chars.next()? {
            'w' => White,
            'b' => Black,
            _ => return None,
        };
        let bug = match chars.next()? {
            'A' => Ant,
            'B' => Beetle,
            'G' => Grasshopper,
            'Q' => Queen,
            'S' => Spider,
            _ => return None,
        };
        if let Some(id_char) = chars.next() {
            let id = id_char.to_string().parse::<u8>().ok()?;
            Some(Piece { owner: player, bug, id })
        } else {
            Some(Piece::new(bug, player))
        }
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
        let p = Parser::new();

        assert_eq!(p.parse_move_string("wS1", &board), Some(Turn::Place(Piece::new(Spider, White), ORIGIN)));
        assert_eq!(p.parse_move_string("wS1 wQ-", &board), Some(Turn::Place(Piece::new(Spider, White), ORIGIN.e())));
        assert_eq!(p.parse_move_string("bA1 /wQ", &board), Some(Turn::Move(Piece::new(Ant, Black), ORIGIN.sw())));

        assert_eq!(p.parse_move_string("foo", &board), None);
        assert_eq!(p.parse_move_string("wwQ", &board), None);
        assert_eq!(p.parse_move_string("wQ foo", &board), None);
        assert_eq!(p.parse_move_string("wQ -bQ2", &board), None);
    }
}
