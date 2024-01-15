use crate::core::{Board, Game, Player, PlayerMark, HeuristicFn};

pub struct MinMaxAi<G>
where
    G: Game,
{
    my_marker: PlayerMark,
    /// A performance counter. If we prune well, this number is small
    n_leafs_evaluated: usize,
    heuristic_fn: HeuristicFn<G>,
    max_depth: usize,
    name: String,
}

impl<G:Game> MinMaxAi<G>
{
    pub fn new(mark: PlayerMark, heuristic_fn: HeuristicFn<G>, depth: usize) -> Self {
        Self {
            my_marker: mark,
            n_leafs_evaluated: 0,
            heuristic_fn,
            max_depth: depth,
            name: match mark {
                PlayerMark::Cross => "minimax X".into(),
                PlayerMark::Naught => "minimax O".into(),
            },
        }
    }

    /// It is good to win. It is bad to lose.
    /// If we can win, we want to win fast,
    /// If we must lose or tie, we want to lose slowly
    /// It is always good to hold the mid point
    fn heuristic(&mut self, b: &G::Board) -> f64 {
        self.n_leafs_evaluated += 1;
        (self.heuristic_fn)(self.my_marker, b)
    }

    /// compute the score of a node by use of minimax
    /// Assumes I want to maximize my score, and the opponent makes moves to minimize it
    fn minimax(&mut self, node: &G::Board, depth: usize, my_move: bool) -> f64 {
        if depth == 0 || node.game_over() {
            let s = self.heuristic(node);
            // println!("Leaf node board\n {node} gets score {s}, at {depth}. Compare with {a} and {b}");
            return s;
        }
        let moves = node.valid_moves();
        let my_marker = self.my_marker; // take a copy here
        if my_move {
            // In this branch, the AI tries to find a move for itself that would maximize the score
            let mut value = -f64::INFINITY;
            let child_nodes = moves.iter().map(|addr| {
                let mut child = *node;
                child.place_mark(*addr, my_marker);
                child
            });
            for child in child_nodes {
                let newval = self.minimax(&child, depth - 1, false);
                value = value.max(newval);
            }
            value
        } else {
            // In this branch, the AI tries to find a move for the other player that would minimize the score
            let mut value = f64::INFINITY;
            let child_nodes = moves.iter().map(|addr| {
                let mut child = *node;
                child.place_mark(*addr, my_marker.other());
                child
            });
            for child in child_nodes {
                value = value.min(self.minimax(&child, depth - 1, true));
            }
            value
        }
    }
}

impl<G:Game> Player<G> for MinMaxAi<G>
{
    fn play(&mut self, b: &G::Board) -> G::Action {
        let res = b
            .valid_moves()
            .iter()
            .map(|addr| {
                let mut b2 = *b;
                b2.place_mark(*addr, self.my_marker);
                let score = self.minimax(&b2, self.max_depth, false);
                (score, addr)
            })
            .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(_, &q)| q)
            .expect("At least one element");
        println!(
            "{} heuristic evaluations computed by {}",
            self.n_leafs_evaluated, self.name
        );
        res
    }
}

#[cfg(test)]
mod test {
    use crate::{
        tictactoe::Action,
        tictactoe::{Board, TicTacToeGame},
        ttt_heuristic, min_max::MinMaxAi,
        core::Player
    };

    #[test]
    fn can_find_winning_move() {
        let b = Board::from_str("   xx    ");
        let mut ai = MinMaxAi::<TicTacToeGame>::new(crate::PlayerMark::Cross, ttt_heuristic, 10);
        let action: Action = ai.play(&b);
        assert_eq!(action, Action(6))
    }
    #[test]
    fn can_block_winning_move() {
        let b = Board::from_str("oo  x    ");
        let mut ai = MinMaxAi::<TicTacToeGame>::new(crate::PlayerMark::Cross, ttt_heuristic, 10);
        let action = ai.play(&b);
        assert_eq!(action, Action(3))
    }
}
