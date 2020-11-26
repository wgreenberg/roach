use crate::game_state::Color;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Bug {
    Ant,
    Beetle,
    Grasshopper,
    Ladybug,
    Mosquito,
    Queen,
    Pillbug,
    Spider,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Piece {
    pub id: u8,
    pub bug: Bug,
    pub owner: Color,
}

impl Piece {
    pub fn new(bug: Bug, owner: Color) -> Piece {
        Piece { owner, bug, id: 1 }
    }

    pub fn new_set(bug: Bug, owner: Color, num_pieces: u8) -> Vec<Piece> {
        (0..num_pieces).map(|i| Piece {
            bug, owner, id: i + 1,
        }).collect()
    }
}
