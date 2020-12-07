use crate::player::Player;
use crate::hive_match::{HiveMatch, HiveSession};
use hive::game_state::GameType;
use std::collections::HashMap;
use crate::client::Client;

pub struct Matchmaker<T> {
    pool: Vec<Player>,
    game_type: GameType,
    player_clients: HashMap<i32, T>,
    pending_matches: Vec<HiveMatch>,
}

#[derive(Debug, PartialEq)]
pub enum ClientStatus<T> where T: Client {
    Pending,
    Ready(HiveMatch, HiveSession<T>),
}

#[derive(Debug, PartialEq)]
pub enum PollStatus {
    Ready,
    NotReady,
}

#[derive(Debug, PartialEq)]
pub enum MatchmakingError {
    PlayerAlreadyInQueue,
    PlayerNotQueued,
}

impl<T> Matchmaker<T> where T: Client {
    pub fn new(game_type: GameType) -> Matchmaker<T> {
        Matchmaker {
            pool: Vec::new(),
            game_type,
            pending_matches: Vec::new(),
            player_clients: HashMap::new(),
        }
    }

    pub fn is_queued(&self, player: &Player) -> bool {
        self.pool.iter().find(|p| p.id == player.id).is_some()
    }

    pub fn add_to_pool(&mut self, player: &Player) -> Result<(), MatchmakingError> {
        if self.is_queued(player) || self.get_pending_match_idx(player).is_some() {
            Err(MatchmakingError::PlayerAlreadyInQueue)
        } else {
            self.pool.push(player.clone());
            Ok(())
        }
    }

    pub fn submit_client(&mut self, player: &Player, client: T) -> Result<ClientStatus<T>, MatchmakingError> {
        match self.get_pending_match_idx(&player) {
            Some(idx) => {
                let pending_match = &self.pending_matches[idx];
                let player_black = pending_match.black.id == player.id;
                let other_player_id = if player_black {
                    pending_match.white.id.unwrap()
                } else {
                    pending_match.black.id.unwrap()
                };
                match self.player_clients.remove(&other_player_id) {
                    Some(other_client) => {
                        let (black_client, white_client) = if player_black {
                            (client, other_client)
                        } else {
                            (other_client, client)
                        };
                        let pending_match = self.pending_matches.remove(idx);
                        let session = pending_match.create_session(black_client, white_client);
                        Ok(ClientStatus::Ready(pending_match, session))
                    },
                    None => {
                        self.player_clients.insert(player.id.unwrap(), client);
                        Ok(ClientStatus::Pending)
                    },
                }
            },
            None => Err(MatchmakingError::PlayerNotQueued),
        }
    }

    pub fn has_pending_match(&self, player: &Player) -> bool {
        self.get_pending_match_idx(player).is_some()
    }

    fn get_pending_match_idx(&self, player: &Player) -> Option<usize> {
        self.pending_matches.iter().position(|hive_match| {
            hive_match.white.id() == player.id() || hive_match.black.id() == player.id()
        })
    }

    pub fn poll(&mut self, player: &Player) -> Result<PollStatus, MatchmakingError> {
        if self.get_pending_match_idx(&player).is_some() {
            Ok(PollStatus::Ready)
        } else {
            if !self.is_queued(&player) {
                return Err(MatchmakingError::PlayerNotQueued);
            }
            // TODO base this on ELO
            if self.pool.len() > 1 {
                let idx = self.pool.iter()
                    .position(|p| p.id() == player.id())
                    .unwrap();
                let player = self.pool.remove(idx);
                let opponent = self.pool.pop().unwrap();
                let pending_match = HiveMatch::new(player, opponent, self.game_type);
                println!("pushing");
                self.pending_matches.push(pending_match);
                Ok(PollStatus::Ready)
            } else {
                Ok(PollStatus::NotReady)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use crate::client::ClientResult;

    #[derive(Debug, PartialEq)]
    struct FakeClient;

    #[async_trait]
    impl Client for FakeClient {
        async fn submit_command(&mut self, command: String) -> ClientResult {
            Ok("hi".to_string())
        }
    }

    #[test]
    fn test_basic_matchmaking() {
        let (mut p1, _) = Player::new("foo".into());
        p1.id = Some(1);
        let (mut p2, _) = Player::new("bar".into());
        p2.id = Some(2);
        let mut mm: Matchmaker<FakeClient> = Matchmaker::new(GameType::Base);

        // players can't check their status if not queued
        assert_eq!(mm.poll(&p1), Err(MatchmakingError::PlayerNotQueued));
        assert!(mm.add_to_pool(&p1).is_ok());
        assert_eq!(mm.poll(&p1), Ok(PollStatus::NotReady));

        // players can't re-enter the matchmaking pool while queued
        assert_eq!(mm.add_to_pool(&p1), Err(MatchmakingError::PlayerAlreadyInQueue));
        assert!(mm.add_to_pool(&p2).is_ok());
        assert_eq!(mm.poll(&p1), Ok(PollStatus::Ready));
        assert_eq!(mm.poll(&p1), Ok(PollStatus::Ready)); // idempotency
        assert_eq!(mm.poll(&p2), Ok(PollStatus::Ready));

        // even though the player's match is pending (i.e. they're not queued), they can't submit
        // until that match has started
        assert_eq!(mm.add_to_pool(&p1), Err(MatchmakingError::PlayerAlreadyInQueue));
        assert_eq!(mm.submit_client(&p1, FakeClient), Ok(ClientStatus::Pending));
        // let player re-submit a client (i.e. on disconnect)
        assert_eq!(mm.submit_client(&p1, FakeClient), Ok(ClientStatus::Pending));
        match mm.submit_client(&p2, FakeClient) {
            Ok(ClientStatus::Ready(hive_match, session)) => {}, // nice
            other => panic!("expected Ready status, got {:?}", other),
        }
        assert_eq!(mm.submit_client(&p1, FakeClient), Err(MatchmakingError::PlayerNotQueued));
    }
}
