use std::collections::HashSet;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Hex {
    pub x: i8,
    pub y: i8,
    pub z: i8,
}

pub const ORIGIN: Hex = Hex { x: 0, y: 0, z: 0 };

// Hexes are oriented pointy side down
// nw  /\ ne
//  w |  | e
// sw  \/ se
impl Hex {
    pub fn new(x: i8, y: i8, z: i8) -> Hex {
        assert_eq!(x + y + z, 0);
        Hex { x, y, z }
    }

    pub fn add(&self, other: &Hex) -> Hex {
        Hex { x: self.x + other.x, y: self.y + other.y, z: self.z + other.z }
    }

    pub fn sub(&self, other: &Hex) -> Hex {
        Hex { x: self.x - other.x, y: self.y - other.y, z: self.z - other.z }
    }

    pub fn dist(&self, other: &Hex) -> i8 {
        let dx = (self.x - other.x).abs();
        let dy = (self.y - other.y).abs();
        let dz = (self.z - other.z).abs();
        (dx + dy + dz) / 2
    }

    pub fn is_adj(&self, other: &Hex) -> bool {
        self.dist(other) == 1
    }

    // Directional neighbors
    pub fn ne(&self)-> Hex { self.add(&Hex::new(1, 0, -1)) }
    pub fn nw(&self)-> Hex { self.add(&Hex::new(0, 1, -1)) }
    pub fn se(&self)-> Hex { self.add(&Hex::new(0, -1, 1)) }
    pub fn sw(&self)-> Hex { self.add(&Hex::new(-1, 0, 1)) }
    pub fn e(&self) -> Hex { self.add(&Hex::new(1, -1, 0)) }
    pub fn w(&self) -> Hex { self.add(&Hex::new(-1, 1, 0)) }

    pub fn neighbors(&self) -> Vec<Hex> {
        vec![self.ne(), self.e(), self.se(), self.sw(), self.w(), self.nw()]
    }

    // Given a collection of hexes, return the list of unique unoccupied
    // neighboring hexes
    pub fn get_empty_neighbors(hexes: &Vec<Hex>) -> Vec<Hex> {
        let mut neighbors: Vec<Hex> = hexes.iter()
            .flat_map(|hex| hex.neighbors())
            .filter(|hex| !hexes.contains(hex))
            .collect();
        neighbors.sort_unstable();
        neighbors.dedup();
        neighbors
    }

    pub fn all_contiguous(hexes: &Vec<Hex>) -> bool {
        if hexes.len() == 0 { return false; }
        let mut visited = Vec::new();
        let start = hexes[0];
        dfs(start, &hexes, &mut visited);
        visited.len() == hexes.len()
    }

    /* A.get_pincers(B) == Some((p1, p2))
     *       / \
     *      | p1|
     *     / \ / \
     *    | A | B |
     *     \ / \ /
     *      | p2|
     *       \ /
     */
    pub fn get_pincers(&self, other: &Hex) -> Option<(Hex, Hex)> {
        if self.dist(other) == 1 {
            let vec = other.sub(self);
            let p1 = Hex::new(-vec.z, -vec.x, -vec.y);
            let p2 = Hex::new(-vec.y, -vec.z, -vec.x);
            Some((self.add(&p1), self.add(&p2)))
        } else {
            None
        }
    }

    pub fn pathfind(&self, hexes: &Vec<Hex>, barriers: &Vec<Hex>, dist: Option<usize>) -> Vec<Hex> {
        if dist == Some(0) { return vec![*self]; }
        let mut visited: HashSet<Hex> = HashSet::new();
        dfs_with_gate_checks(*self, hexes, barriers, &mut visited, 0, dist).iter()
            .filter(|&h| h != self)
            .cloned().collect()
    }
}

fn dfs_with_gate_checks(hex: Hex, hexes: &Vec<Hex>, barriers: &Vec<Hex>, visited: &mut HashSet<Hex>, dist: usize, max_dist: Option<usize>) -> Vec<Hex> {
    if let Some(max) = max_dist {
        if dist == max {
            return vec![hex];
        }
    }

    visited.insert(hex);
    let mut result = match max_dist {
        Some(_) => Vec::new(),
        None => vec![hex],
    };
    for neighbor in hex.neighbors() {
        if hexes.contains(&neighbor) && !visited.contains(&neighbor) {
            if barriers.len() > 0 {
                let (pincer_a, pincer_b) = hex.get_pincers(&neighbor).unwrap();
                // the move is invalid if both pincers are present (too small a gap to slide in),
                // or if neither are present (jumping a gap)
                match (barriers.contains(&pincer_a), barriers.contains(&pincer_b)) {
                    (true, true) | (false, false) => continue,
                    _ => {},
                }
            }
            let mut v = visited.clone();
            result.extend(dfs_with_gate_checks(neighbor, hexes, barriers, &mut v, dist + 1, max_dist));
        }
    }
    return result;
}

fn dfs(hex: Hex, hexes: &Vec<Hex>, visited: &mut Vec<Hex>) {
    visited.push(hex);
    for neighbor in hex.neighbors() {
        if hexes.contains(&neighbor) && !visited.contains(&neighbor) {
            dfs(neighbor, hexes, visited);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::assert_set_equality;

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
            assert_eq!(ORIGIN.dist(&neighbor), 1);
        }

        let mut inner_ring = ORIGIN.neighbors();
        inner_ring.push(ORIGIN);
        for neighbor in Hex::get_empty_neighbors(&inner_ring) {
            assert_eq!(ORIGIN.dist(&neighbor), 2);
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

    #[test]
    fn test_pathfinding() {
        /*     / \ / \     / \ / \
         *    |   | x |   | x | x |
         *     \ / \ / \ / \ / \ / \
         *      |   | x | x |   | x |
         *       \ / \ / \ / \ / \ /
         *        |   | s |   | x |
         *         \ / \ / \ / \ / \
         * 's' is start     |   | x |
         * 'x' is a barrier  \ / \ /
         */
        let map = vec![
            ORIGIN,
            ORIGIN.e(), ORIGIN.e().se(), ORIGIN.e().ne(),
            ORIGIN.w(), ORIGIN.w().nw(), ORIGIN.w().nw().nw(),
        ];
        let barriers = vec![
            ORIGIN.ne(), ORIGIN.e().e(), ORIGIN.nw(), ORIGIN.nw().nw(), ORIGIN.ne().ne(),
            ORIGIN.e().e().ne(), ORIGIN.e().e().se(),
        ];
        assert_set_equality(ORIGIN.pathfind(&map, &barriers, Some(0)), vec![ORIGIN]);
        assert_set_equality(ORIGIN.pathfind(&map, &barriers, Some(1)), vec![
            ORIGIN.e(), ORIGIN.w()
        ]);
        assert_set_equality(ORIGIN.pathfind(&map, &barriers, Some(2)), vec![
            ORIGIN.e().se(), ORIGIN.w().nw(),
        ]);
        assert_set_equality(ORIGIN.pathfind(&map, &barriers, None), vec![
            ORIGIN.e(), ORIGIN.e().se(),
            ORIGIN.w(), ORIGIN.w().nw(), ORIGIN.w().nw().nw(),
        ]);
    }

    fn test_pathfinding_with_gap() {
        let barriers = vec![
            ORIGIN,
            ORIGIN.sw(),
            ORIGIN.sw().se(),
            ORIGIN.sw().se().e(),
            ORIGIN.sw().se().e().ne(),
        ];
        let map = Hex::get_empty_neighbors(&barriers);
        assert_set_equality(ORIGIN.ne().pathfind(&map, &barriers, Some(2)), vec![
            ORIGIN.e().e(),
            ORIGIN.w(),
        ]);
    }

    #[test]
    fn test_pathfinding_multiple_paths() {
        let barriers = vec![];
        let map = vec![ORIGIN, ORIGIN.nw(), ORIGIN.w(), ORIGIN.w().w(), ORIGIN.nw().ne()];
        assert_set_equality(ORIGIN.pathfind(&map, &barriers, Some(2)), vec![
            ORIGIN.nw(), ORIGIN.w(), ORIGIN.w().w(), ORIGIN.nw().ne()
        ]);
    }

    #[test]
    fn test_get_pincers() {
        assert_eq!(ORIGIN.get_pincers(&ORIGIN), None);
        let p = ORIGIN.get_pincers(&ORIGIN.e());
        assert!(p == Some((ORIGIN.se(), ORIGIN.ne())) || p == Some((ORIGIN.ne(), ORIGIN.se())));
        let p = ORIGIN.get_pincers(&ORIGIN.nw());
        assert!(p == Some((ORIGIN.w(), ORIGIN.ne())) || p == Some((ORIGIN.ne(), ORIGIN.w())));
    }
}
