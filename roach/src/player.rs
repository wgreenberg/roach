const INITIAL_ELO: f64 = 1500.0;

#[derive(PartialEq, Debug)]
pub struct Player {
    pub name: String,
    token: String,
    pub elo_score: f64,
}

impl Player {
    pub fn new(name: String) -> Player {
        Player {
            token: name.clone(),
            name,
            elo_score: INITIAL_ELO,
        }
    }
}
