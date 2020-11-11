use std::fmt::Debug;

// how deep we're willing to go down a tree before bailing out (resulting in a draw)
const MAX_DEPTH: usize = 100;
const EXPLORATION_COEFF: f64 = 2.0;

#[derive(Debug)]
struct StatsNode<T> where T: MonteCarloSearchable {
    n_visits: usize,
    total_score: f64,
    game: T,
    unexplored_actions: Vec<T::Action>,

    idx: usize,
    parent: Option<usize>,
    children: Vec<usize>,
}

impl<T> StatsNode<T> where T: MonteCarloSearchable + Debug {
    fn new(idx: usize, game: T, parent: Option<usize>) -> Self {
        StatsNode {
            n_visits: 0,
            total_score: 0.0,
            unexplored_actions: game.get_possible_actions(),
            game,
            idx,
            parent,
            children: Vec::new(),
        }
    }

    fn update(&mut self, score: f64) {
        self.n_visits += 1;
        self.total_score += score;
    }

    fn is_terminal(&self) -> bool {
        self.game.get_terminal_value().is_some()
    }

    fn is_expanded(&self) -> bool {
        self.unexplored_actions.len() == 0
    }
}

#[derive(Debug)]
struct MCSearchTree<T> where T: MonteCarloSearchable {
    arena: Vec<StatsNode<T>>,
}

impl<T> MCSearchTree<T> where T: MonteCarloSearchable + Debug {
    fn new(game: T) -> Self {
        MCSearchTree {
            arena: vec![StatsNode::new(0, game, None)],
        }
    }

    pub fn find_best_action(&mut self) -> T::Action {
        for _ in 0..100 {
            let v = self.select(0);
            let reward = self.simulate(v);
            self.backup(v, reward);
        }
        self.arena[self.best_child(0)].game.get_last_action().unwrap()
    }

    fn best_child(&self, node: usize) -> usize {
        let mut max_ucp = f64::NEG_INFINITY;
        let mut best_child: Option<usize> = None;
        let v = &self.arena[node];
        for &v_i in &v.children {
            let v = &self.arena[v_i];
            let exploitation = v.total_score / (v.n_visits as f64);
            let exploration = (2.0 * (v.n_visits as f64).ln()) / (v.n_visits + 1) as f64;
            let ucp = exploitation + EXPLORATION_COEFF * exploration;
            if ucp > max_ucp {
                max_ucp = ucp;
                best_child = Some(v_i);
            }
        }
        best_child.unwrap()
    }

    // TreePolicy
    fn select(&mut self, node: usize) -> usize {
        let mut v = node;
        while !self.arena[v].is_terminal() {
            if !self.arena[v].is_expanded() {
                return self.expand(v);
            } else {
                v = self.best_child(v);
            }
        }
        v
    }

    fn expand(&mut self, node: usize) -> usize{
        let new_idx = self.arena.len();
        let v = &mut self.arena[node];
        let next_unexplored_action = v.unexplored_actions.pop().unwrap();
        let new_game_state = v.game.apply_action(next_unexplored_action);
        let new_child = StatsNode::new(new_idx, new_game_state, Some(node));
        v.children.push(new_idx);
        self.arena.push(new_child);
        new_idx
    }

    fn simulate(&mut self, node: usize) -> f64 {
        self.arena[node].game.simulate()
    }

    fn backup(&mut self, node: usize, score: f64) {
        let mut v = Some(node);
        let mut score = score;
        while let Some(v_i) = v {
            self.arena[v_i].update(score);
            score = -score;
            v = self.arena[v_i].parent;
        }
    }
}

pub trait MonteCarloSearchable: Clone + Debug {
    type Action: Debug;

    // simulate a random walk from this state and return the score
    fn simulate(&self) -> f64;
    // Some(reward) if the game is over, None if we can keep playing
    fn get_terminal_value(&self) -> Option<f64>;
    fn get_possible_actions(&self) -> Vec<Self::Action>;
    fn get_last_action(&self) -> Option<Self::Action>;
    fn apply_action(&self, action: Self::Action) -> Self;

    fn find_best_action_mcts(&self) -> Self::Action {
        let mut tree = MCSearchTree::new(self.clone());
        tree.find_best_action()
    }
}
