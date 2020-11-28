use serde::Serialize;

const INITIAL_ELO: f64 = 1500.0;

#[derive(PartialEq, Debug, Serialize, Clone)]
pub struct Player {
    pub name: String,
    pub elo_score: f64,
}

impl Player {
    pub fn new(name: String) -> Player {
        Player {
            name,
            elo_score: INITIAL_ELO,
        }
    }
}
