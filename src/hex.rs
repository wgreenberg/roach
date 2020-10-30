#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Hex {
    pub x: i64,
    pub y: i64,
    pub z: i64,
}

pub const ORIGIN: Hex = Hex { x: 0, y: 0, z: 0 };

impl Hex {
    fn new(x: i64, y: i64, z: i64) -> Hex {
        assert_eq!(x + y + z, 0);
        Hex { x, y, z }
    }

    fn add(&self, other: Hex) -> Hex {
        Hex { x: self.x + other.x, y: self.y + other.y, z: self.z + other.z }
    }

    fn dist(&self, other: Hex) -> i64 {
        let dx = (self.x - other.x).abs();
        let dy = (self.y - other.y).abs();
        let dz = (self.z - other.z).abs();
        (dx + dy + dz) / 2
    }

    fn neighbors(&self) -> Vec<Hex> {
        vec![
            self.add(Hex::new(1, -1, 0)),
            self.add(Hex::new(1, 0, -1)),
            self.add(Hex::new(0, 1, -1)),
            self.add(Hex::new(-1, 1, 0)),
            self.add(Hex::new(-1, 0, 1)),
            self.add(Hex::new(0, -1, 1)),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
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
}
