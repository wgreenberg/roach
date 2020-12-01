use crate::player::Player;
use crate::hive_match::HiveMatch;
use hive::game_state::GameType;

pub struct Matchmaker {
    pool: Vec<Player>,
    game_type: GameType,
}

impl Matchmaker {
    pub fn new(game_type: GameType) -> Matchmaker {
        Matchmaker {
            pool: Vec::new(),
            game_type,
        }
    }

    pub fn is_queued(&self, player: Player) -> bool {
        self.pool.contains(&player)
    }

    pub fn add_to_pool(&mut self, player: Player) {
        self.pool.push(player);
    }

    pub fn find_match(&mut self, player: Player) -> Option<HiveMatch> {
        // TODO base this on ELO
        if self.pool.len() > 1 {
            let idx = self.pool.iter().position(|p| *p == player).unwrap();
            let player = self.pool.remove(idx);
            self.pool.pop().map(move |opponent| HiveMatch::new(player, opponent, self.game_type))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_matchmaking() {
        let (p1, _) = Player::new("foo".into());
        let (p2, _) = Player::new("bar".into());
        let mut mm = Matchmaker::new(GameType::Base);
        mm.add_to_pool(p1.clone());
        assert_eq!(mm.find_match(p1.clone()), None);
        mm.add_to_pool(p2.clone());
        let m = mm.find_match(p1.clone());
        assert!(m.is_some());
        let m = m.unwrap();
        assert!(m.white == p1 || m.white == p2);
        assert!(m.black == p1 || m.black == p2);
        assert_eq!(mm.pool.len(), 0);
    }
}
