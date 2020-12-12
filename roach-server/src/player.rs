use serde::Serialize;
use sha2::{Sha256, Digest};
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

const INITIAL_ELO: i32 = 1500;

#[derive(PartialEq, Debug, Serialize, Clone)]
pub struct Player {
    pub id: Option<i32>,
    pub name: String,
    pub elo: i32,

    #[serde(skip_serializing)]
    pub token_hash: String,
}

#[derive(Debug, Serialize, Default)]
pub struct PlayerStatistics {
    pub n_wins: u64,
    pub n_losses: u64,
    pub n_draws: u64,
    pub n_fault_wins: u64,
    pub n_fault_losses: u64,
    pub n_games: u64, // sum of all of the above
}

pub fn hash_string(string: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(string);
    format!("{:x}", hasher.finalize())
}

fn random_token() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .collect()
}

impl Player {
    pub fn new(name: String) -> (Player, String) {
        let mut player = Player {
            id: None,
            name,
            elo: INITIAL_ELO,
            token_hash: "".to_string(),
        };
        let token = player.roll_token();
        (player, token)
    }

    pub fn id(&self) -> i32 {
        let err_str = format!("ERR: tried to get id of non-inserted player {:?}", self);
        self.id.expect(&err_str)
    }

    pub fn roll_token(&mut self) -> String {
        let token = random_token();
        self.token_hash = hash_string(&token);
        token
    }
}
