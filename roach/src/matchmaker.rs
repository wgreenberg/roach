use crate::player::Player;
use crate::hive_match::HiveMatch;
use hive::game_state::GameType;

pub struct Matchmaker<'a> {
    pool: Vec<&'a Player>,
    game_type: GameType,
}

impl<'a> Matchmaker<'a> {
    pub fn new(game_type: GameType) -> Matchmaker<'a> {
        Matchmaker {
            pool: Vec::new(),
            game_type,
        }
    }

    pub fn add_to_pool(&mut self, player: &'a Player) {
        self.pool.push(player);
    }

    pub fn find_potential_matches(&self) -> Vec<HiveMatch<'a>> {
        // TODO base this on ELO
        self.pool.chunks_exact(2)
            .map(|chunk| HiveMatch::new(chunk[0], chunk[1], self.game_type))
            .collect()
    }

    pub fn confirm_match(&mut self, hive_match: &HiveMatch<'a>) {
        self.pool.retain(|&player| player != hive_match.white && player != hive_match.black);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_matchmaking() {
        let p1 = Player::new("foo".into());
        let p2 = Player::new("bar".into());
        let mut mm = Matchmaker::new(GameType::Base);
        mm.add_to_pool(&p1);
        assert_eq!(mm.find_potential_matches().len(), 0);
        mm.add_to_pool(&p2);
        assert_eq!(mm.pool, vec![&p1, &p2]);
        let matches = mm.find_potential_matches();
        assert_eq!(matches.len(), 1);
        assert!([matches[0].white, matches[0].black].contains(&&p1));
        assert!([matches[0].white, matches[0].black].contains(&&p2));
        mm.confirm_match(&matches[0]);
        assert_eq!(mm.pool.len(), 0);
    }
}
