use serde::Serialize;
use sha2::{Sha256, Digest};
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use crate::schema::players;
use crate::model::PlayerRowInsertable;

const INITIAL_ELO: i32 = 1500;

#[derive(PartialEq, Debug, Serialize, Clone)]
pub struct Player {
    pub id: Option<i32>,
    pub name: String,
    pub elo: i32,

    #[serde(skip_serializing)]
    pub token_hash: String,
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

    pub fn roll_token(&mut self) -> String {
        let token = random_token();
        self.token_hash = hash_string(&token);
        token
    }
}
