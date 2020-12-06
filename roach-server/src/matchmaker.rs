use crate::player::Player;
use crate::hive_match::HiveMatch;
use hive::game_state::GameType;

pub struct Matchmaker {
    pool: Vec<Player>,
    game_type: GameType,
}

#[derive(Debug)]
pub enum MatchmakingError {
    PlayerAlreadyInQueue,
    PlayerNotQueued,
}

impl Matchmaker {
    pub fn new(game_type: GameType) -> Matchmaker {
        Matchmaker {
            pool: Vec::new(),
            game_type,
        }
    }

    pub fn is_queued(&self, player: &Player) -> bool {
        self.pool.iter().find(|p| p.id == player.id).is_some()
    }

    pub fn add_to_pool(&mut self, player: Player) -> Result<(), MatchmakingError> {
        if self.is_queued(&player) {
            Err(MatchmakingError::PlayerAlreadyInQueue)
        } else {
            self.pool.push(player);
            Ok(())
        }
    }

    pub fn find_match(&mut self, player: Player) -> Result<Option<HiveMatch>, MatchmakingError> {
        if !self.is_queued(&player) {
            return Err(MatchmakingError::PlayerNotQueued);
        }

        // TODO base this on ELO
        if self.pool.len() > 1 {
            let idx = self.pool.iter().position(|p| *p == player).unwrap();
            let player = self.pool.remove(idx);
            Ok(self.pool.pop()
                .map(move |opponent| HiveMatch::new(player, opponent, self.game_type)))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_matchmaking() {
        let (mut p1, _) = Player::new("foo".into());
        p1.id = Some(1);
        let (mut p2, _) = Player::new("bar".into());
        p2.id = Some(2);
        let mut mm = Matchmaker::new(GameType::Base);
        assert!(mm.find_match(p1.clone()).is_err());
        assert!(mm.add_to_pool(p1.clone()).is_ok());
        assert_eq!(mm.find_match(p1.clone()).unwrap(), None);
        assert!(mm.add_to_pool(p2.clone()).is_ok());
        let m = mm.find_match(p1.clone()).unwrap();
        assert!(m.is_some());
        let m = m.unwrap();
        assert!(m.white == p1 || m.white == p2);
        assert!(m.black == p1 || m.black == p2);
        assert_eq!(mm.pool.len(), 0);
    }

    #[test]
    fn test_preventing_duplicates() {
        let (p1, _) = Player::new("foo".into());
        let mut mm = Matchmaker::new(GameType::Base);
        assert!(mm.add_to_pool(p1.clone()).is_ok());
        assert!(mm.add_to_pool(p1.clone()).is_err());
    }
}
