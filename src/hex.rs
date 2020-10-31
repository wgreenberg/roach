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
    pub fn get_all_neighbors(hexes: Vec<Hex>) -> Vec<Hex> {
        let neighbors = hexes.iter()
            .flat_map(|hex| hex.neighbors());
        let set: HashSet<Hex> = HashSet::from_iter(neighbors);
        let hexes_set = HashSet::from_iter(hexes.iter().cloned());
        set.difference(&hexes_set).cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::hash::Hash;
    use std::fmt::Debug;

    fn assert_set_equality<T>(a: Vec<T>, b: Vec<T>)
        where T: Clone + Eq + Hash + Debug {
        let hash_a: HashSet<T> = a.iter().cloned().collect();
        let hash_b: HashSet<T> = b.iter().cloned().collect();
        assert_eq!(hash_a, hash_b);
    }

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
    fn test_get_all_neighbors() {
        let hexes = vec![ORIGIN, Hex::new(-1, 1, 0)];
        assert_set_equality(Hex::get_all_neighbors(hexes), vec![
            Hex::new(1, -1, 0), Hex::new(1, 0, -1), Hex::new(0, 1, -1), Hex::new(0, -1, 1), Hex::new(-1, 0, 1),
            Hex::new(-2, 1, 1), Hex::new(-2, 2, 0), Hex::new(-1, 2, -1),
        ]);
    }
}
