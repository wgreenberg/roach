use std::collections::HashSet;
use std::iter::FromIterator;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Hex {
    pub x: i64,
    pub y: i64,
    pub z: i64,
}

pub const ORIGIN: Hex = Hex { x: 0, y: 0, z: 0 };

// Hexes are oriented pointy side down
// nw  /\ ne
//  w |  | e
// sw  \/ se
impl Hex {
    pub fn new(x: i64, y: i64, z: i64) -> Hex {
        assert_eq!(x + y + z, 0);
        Hex { x, y, z }
    }

    pub fn add(&self, other: Hex) -> Hex {
        Hex { x: self.x + other.x, y: self.y + other.y, z: self.z + other.z }
    }

    pub fn dist(&self, other: Hex) -> i64 {
        let dx = (self.x - other.x).abs();
        let dy = (self.y - other.y).abs();
        let dz = (self.z - other.z).abs();
        (dx + dy + dz) / 2
    }

    pub fn is_adj(&self, other: Hex) -> bool {
        self.neighbors().contains(&other)
    }

    // Directional neighbors
    pub fn ne(&self)-> Hex { self.add(Hex::new(1, 0, -1)) }
    pub fn nw(&self)-> Hex { self.add(Hex::new(0, 1, -1)) }
    pub fn se(&self)-> Hex { self.add(Hex::new(0, -1, 1)) }
    pub fn sw(&self)-> Hex { self.add(Hex::new(-1, 0, 1)) }
    pub fn e(&self) -> Hex { self.add(Hex::new(1, -1, 0)) }
    pub fn w(&self) -> Hex { self.add(Hex::new(-1, 1, 0)) }

    pub fn neighbors(&self) -> Vec<Hex> {
        vec![self.ne(), self.nw(), self.se(), self.sw(), self.e(), self.w()]
    }

    // Given a collection of hexes, return the list of unique unoccupied
    // neighboring hexes
    pub fn get_empty_neighbors(hexes: &Vec<Hex>) -> Vec<Hex> {
        let neighbors = hexes.iter()
            .flat_map(|hex| hex.neighbors());
        let neighbors_set: HashSet<Hex> = HashSet::from_iter(neighbors);
        let hexes_set = HashSet::from_iter(hexes.iter().cloned());
        neighbors_set.difference(&hexes_set).cloned().collect()
    }

    pub fn all_contiguous(hexes: &Vec<Hex>) -> bool {
        if hexes.len() == 0 { return false; }
        let mut visited: HashSet<Hex> = HashSet::new();
        let start = hexes[0];
        dfs(start, &hexes, &mut visited);
        visited.len() == hexes.len()
    }

    pub fn pathfind(&self, hexes: Vec<Hex>, limit: usize) -> Vec<Hex> {
        todo!();
    }
}

fn dfs(hex: Hex, hexes: &Vec<Hex>, visited: &mut HashSet<Hex>) {
    visited.insert(hex);
    for neighbor in hex.neighbors() {
        if hexes.contains(&neighbor) && !visited.contains(&neighbor) {
            dfs(neighbor, hexes, visited);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_utils::assert_set_equality;

    #[test]
    fn test_neighbors() {
        assert_set_equality(ORIGIN.neighbors(), vec![
            Hex::new(1, -1, 0), Hex::new(1, 0, -1), Hex::new(0, 1, -1),
            Hex::new(-1, 1, 0), Hex::new(-1, 0, 1), Hex::new(0, -1, 1),
        ]);
    }

    #[test]
    fn test_dist() {
        for neighbor in ORIGIN.neighbors() {
            assert_eq!(ORIGIN.dist(neighbor), 1);
        }
    }

    #[test]
    fn test_get_empty_neighbors() {
        let hexes = vec![ORIGIN, Hex::new(-1, 1, 0)];
        assert_set_equality(Hex::get_empty_neighbors(&hexes), vec![
            Hex::new(1, -1, 0), Hex::new(1, 0, -1), Hex::new(0, 1, -1), Hex::new(0, -1, 1), Hex::new(-1, 0, 1),
            Hex::new(-2, 1, 1), Hex::new(-2, 2, 0), Hex::new(-1, 2, -1),
        ]);
    }

    #[test]
    fn test_all_contiguous() {
        // positive cases
        assert!(Hex::all_contiguous(&vec![ORIGIN]));
        assert!(Hex::all_contiguous(&vec![ORIGIN, ORIGIN.e()]));

        // negative cases
        assert!(!Hex::all_contiguous(&vec![]));
        assert!(!Hex::all_contiguous(&vec![ORIGIN, ORIGIN.e().e()]));
        assert!(!Hex::all_contiguous(&vec![ORIGIN, ORIGIN.w(), ORIGIN.e().e()]));
    }
}
