// evaluation scores are positive for player A, and negative for player B
pub struct Evaluation<T> {
    pub node: T,
    pub score: f64,
    pub explanation: String,
}

fn max<T>(a: Evaluation<T>, b: Evaluation<T>) -> Evaluation<T> {
    if a.score >= b.score { a } else { b }
}

pub trait GameTree: Sized {
    type Action;

    fn get_children(&self) -> Vec<Self>;
    fn is_terminal(&self) -> bool;
    fn evaluate_node(&self) -> Evaluation<Self::Action>;
    fn get_node(&self) -> Self::Action;
    fn is_player_a_up(&self) -> bool;

    fn negamax(&self, depth: usize, color: i8) -> Evaluation<Self::Action> {
        if depth == 0 || self.is_terminal() {
            let mut eval = self.evaluate_node();
            eval.score *= color as f64;
            eval
        } else {
            let mut max_eval: Option<Evaluation<Self::Action>> = None;
            for child in self.get_children() {
                let mut child_eval = child.negamax(depth - 1, -color);
                child_eval.score = -child_eval.score;
                child_eval.node = child.get_node();
                max_eval = match max_eval {
                    Some(m) => Some(max(m, child_eval)),
                    None => Some(child_eval),
                }
            }
            max_eval.unwrap()
        }
    }

    fn find_best_action(&self, depth: usize) -> Self::Action {
        if self.is_player_a_up() {
            self.negamax(depth, 1).node
        } else {
            self.negamax(depth, -1).node
        }
    }
}
