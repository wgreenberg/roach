use std::fmt::Debug;
use std::fs::File;
use std::io::{BufWriter, Write};

#[derive(Debug, Copy, Clone)]
pub struct MCTSOptions {
    pub max_depth: usize,
    pub exploration_coefficient: f64,
    pub n_iterations: usize,
}

impl Default for MCTSOptions {
    fn default() -> Self {
        MCTSOptions {
            max_depth: 170, // mentioned in Konz (2012)
            exploration_coefficient: 2.0, // default for UCB1
            n_iterations: 100,
        }
    }
}

#[derive(Debug)]
struct StatsNode<T> where T: MonteCarloSearchable {
    n_visits: usize,
    total_score: u64,
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
            total_score: 0,
            unexplored_actions: game.get_possible_actions(),
            game,
            idx,
            parent,
            children: Vec::new(),
        }
    }

    fn update(&mut self, score: u64) {
        self.n_visits += 1;
        self.total_score += score;
    }

    fn is_expanded(&self) -> bool {
        self.unexplored_actions.len() == 0
    }
}

#[derive(Debug)]
pub struct MCSearchTree<T> where T: MonteCarloSearchable {
    arena: Vec<StatsNode<T>>,
    options: MCTSOptions,
    maxi_player: T::Player,
}

impl<T> MCSearchTree<T> where T: MonteCarloSearchable + Debug {
    pub fn new(game: T, maxi_player: T::Player, options: MCTSOptions) -> Self {
        MCSearchTree {
            arena: vec![StatsNode::new(0, game, None)],
            options: options,
            maxi_player,
        }
    }

    pub fn find_best_action(&mut self) -> T::Action {
        for _ in 0..self.options.n_iterations {
            let v = self.select(0);
            match self.simulate(v) {
                Some(true) => self.backup(v, 1),
                _ => self.backup(v, 0),
            }
        }
        let mut best_action: Option<T::Action> = None;
        let mut most_visits = 0;
        for &i in &self.arena[0].children {
            if self.arena[i].n_visits > most_visits {
                most_visits = self.arena[i].n_visits;
                best_action = self.arena[i].game.get_last_action();
            }
        }
        best_action.unwrap()
    }

    fn best_child(&self, node: usize) -> usize {
        let v = &self.arena[node];
        let (first, rest) = v.children.split_first().unwrap();
        let mut best_ucb = self.ucb1(node, *first);
        let mut best_child = *first;
        for &v_i in rest {
            let ucb = self.ucb1(node, v_i);
            let is_better = if v.game.current_player() == self.maxi_player {
                ucb > best_ucb
            } else {
                ucb < best_ucb
            };
            if is_better {
                best_ucb = ucb;
                best_child = v_i;
            }
        }
        best_child
    }

    fn ucb1(&self, parent_i: usize, child_i: usize) -> f64 {
        let parent = &self.arena[parent_i];
        let child = &self.arena[child_i];
        let exploitation = (child.total_score as f64) / (child.n_visits as f64);
        let exploration = ((parent.n_visits as f64).ln() / (child.n_visits + 1) as f64).sqrt();
        if parent.game.current_player() == self.maxi_player {
            exploitation + self.options.exploration_coefficient * exploration
        } else {
            exploitation - self.options.exploration_coefficient * exploration
        }
    }

    fn select(&mut self, node: usize) -> usize {
        let mut v = node;
        while self.arena[v].game.get_terminal_value(self.maxi_player).is_none() {
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
        let chosen_action = v.game.select_action(&v.unexplored_actions);
        v.unexplored_actions.retain(|action| action != &chosen_action);
        let mut new_game_state = v.game.clone();
        new_game_state.apply_action(chosen_action);
        let new_child = StatsNode::new(new_idx, new_game_state, Some(node));
        v.children.push(new_idx);
        self.arena.push(new_child);
        new_idx
    }

    fn simulate(&self, node: usize) -> Option<bool> {
        self.arena[node].game.simulate(self.options.max_depth, self.maxi_player)
    }

    fn backup(&mut self, node: usize, score: u64) {
        let mut v = Some(node);
        while let Some(v_i) = v {
            self.arena[v_i].update(score);
            v = self.arena[v_i].parent;
        }
    }

    pub fn write_tree(&self, path: &str) -> std::io::Result<()> {
        let file = File::create(path)?;
        let mut w = BufWriter::new(&file);
        write!(&mut w, "digraph MCTS {{")?;
        write!(&mut w, "node [shape=record]")?;
        for node in &self.arena {
            let score = (node.total_score as f64) / (node.n_visits as f64);
            let node_str = match node.parent {
                Some(parent) => self.arena[parent].game.describe_action(node.game.get_last_action().unwrap()),
                None => "()".to_string(),
            };
            write!(&mut w, "{} [label=\"{} | score {:.2} | visits {}", node.idx, node_str, score, node.n_visits)?;
            match node.parent {
                Some(parent) => write!(&mut w, " | ucb {:.2}\"];", self.ucb1(parent, node.idx))?,
                None => write!(&mut w, "\"];")?,
            }
            for child in &node.children {
                write!(&mut w, "{} -> {};", node.idx, child)?;
            }
        }
        write!(&mut w, "}}")?;
        Ok(())
    }
}

pub trait MonteCarloSearchable: Clone + Debug {
    type Action: Debug + PartialEq;
    type Player: Copy + Clone + Debug + PartialEq;

    fn get_terminal_value(&self, player: Self::Player) -> Option<bool>;
    fn get_possible_actions(&self) -> Vec<Self::Action>;
    fn get_last_action(&self) -> Option<Self::Action>;
    fn apply_action(&mut self, action: Self::Action);
    fn select_action(&self, actions: &Vec<Self::Action>) -> Self::Action;
    fn current_player(&self) -> Self::Player;
    fn describe_action(&self, action: Self::Action) -> String;

    // simulate a random walk from this state and return the score
    fn simulate(&self, max_depth: usize, maxi_player: Self::Player) -> Option<bool> {
        let mut simulation = self.clone();
        let mut n_turns = 0;
        let result = loop {
            if n_turns > max_depth {
                break None;
            }
            match simulation.get_terminal_value(maxi_player) {
                Some(reward) => break Some(reward),
                _ => {},
            }
            let choices = simulation.get_possible_actions();
            let turn = simulation.select_action(&choices);
            simulation.apply_action(turn);
            n_turns += 1;
        };
        result
    }

    fn find_best_action_mcts(&self, options: MCTSOptions) -> Self::Action {
        let mut tree = MCSearchTree::new(self.clone(), self.current_player(), options);
        tree.find_best_action()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::iter::FromIterator;
    use rand::prelude::*;

    #[derive(Clone, Debug)]
    struct GameTree {
        child_nodes: HashMap<String, bool>,
        moves: String,
        path_so_far: String,
    }

    impl MonteCarloSearchable for GameTree {
        type Action = char;
        type Player = bool;

        fn get_terminal_value(&self, player: Self::Player) -> Option<bool> {
            self.child_nodes.get(&self.path_so_far).map(|&win_for_a| win_for_a && player)
        }
        fn get_possible_actions(&self) -> Vec<Self::Action> {
            self.moves.chars().filter(|c| !self.path_so_far.contains(*c)).collect()
        }
        fn get_last_action(&self) -> Option<Self::Action> {
            self.path_so_far.chars().last()
        }
        fn apply_action(&mut self, action: Self::Action) {
            self.path_so_far.push(action);
        }
        fn select_action(&self, actions: &Vec<Self::Action>) -> Self::Action {
            let mut rng = thread_rng();
            *actions.choose(&mut rng).unwrap()
        }
        fn current_player(&self) -> Self::Player {
            true
        }
        fn describe_action(&self, action: Self::Action) -> String {
            action.to_string()
        }
    }

    // example tree from
    // https://www.geeksforgeeks.org/minimax-algorithm-in-game-theory-set-1-introduction/
    fn get_connect_2_tree() -> GameTree {
        GameTree {
            moves: "123".to_string(),
            child_nodes: HashMap::from_iter(vec![
                ("123".into(), false),
                ("132".into(), true),
                ("213".into(), true),
                ("231".into(), true),
                ("321".into(), false),
                ("312".into(), true),
            ]),
            path_so_far: String::new(),
        }
    }

    #[test]
    fn test_exploration() {
        let game_tree = get_connect_2_tree();
        let mut search_tree = MCSearchTree::new(game_tree, true, MCTSOptions::default());
        let v = search_tree.select(0);
        assert!(!&search_tree.arena[v].is_expanded());
        let child1 = search_tree.expand(v);
        assert_eq!(search_tree.arena[v].children, vec![child1]);
        assert_eq!(search_tree.arena[v].unexplored_actions.len(), 1);
        assert_eq!(search_tree.arena[v].children.len(), 1);
        let child2 = search_tree.expand(v);
        assert_eq!(search_tree.arena[v].children, vec![child1, child2]);
        assert_eq!(search_tree.arena[v].unexplored_actions.len(), 0);
        assert_eq!(search_tree.arena[v].children.len(), 2);
    }

    #[test]
    fn test_chooses_right_answer() {
        let game_tree = get_connect_2_tree();
        let mut search_tree = MCSearchTree::new(game_tree, true, MCTSOptions::default());
        assert_eq!(search_tree.find_best_action(), '2');
    }
}
