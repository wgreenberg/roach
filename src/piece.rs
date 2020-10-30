use crate::game_state::Player;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Bug {
    Ant,
    Beetle,
    Grasshopper,
    //Ladybug,
    //Mosquito,
    Queen,
    //Pillbug,
    Spider,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Piece {
    pub id: u8,
    pub bug: Bug,
    pub owner: Player,
}

impl Piece {
    pub fn new(bug: Bug, owner: Player) -> Piece {
        Piece { owner, bug, id: 1 }
    }

    pub fn new_set(bug: Bug, owner: Player, num_pieces: u8) -> Vec<Piece> {
        (1..num_pieces).map(|i| Piece {
            bug, owner, id: i,
        }).collect()
    }
}
