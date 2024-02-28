use log::debug;

use crate::core::{BlitzPlayer, Board, HeuristicFn, Player, PlayerMark};

pub struct ABAi<B> {
    my_marker: PlayerMark,
    /// A performance counter. If we prune well, this number is small
    n_leafs_evaluated: usize,
    heuristic_fn: HeuristicFn<B>,
    max_depth: usize,
}

impl<B: Board + Clone> ABAi<B> {
    pub fn new(mark: PlayerMark, heuristic_fn: HeuristicFn<B>, depth: usize) -> Self {
        ABAi {
            my_marker: mark,
            n_leafs_evaluated: 0,
            heuristic_fn,
            max_depth: depth,
        }
    }

    fn heuristic(&mut self, b: &B) -> f64 {
        self.n_leafs_evaluated += 1;
        (self.heuristic_fn)(self.my_marker, b)
    }

    /// compute the score of a node by use of alpha-beta with pruning
    /// Assumes I want to maximize my score, and the opponent makes moves to minimize it
    fn alphabeta(&mut self, node: &B, depth: usize, a: f64, b: f64, my_move: bool) -> f64 {
        if depth == 0 || node.game_is_over() {
            let s = self.heuristic(node);
            // println!("Leaf node board\n {node} gets score {s}, at {depth}. Compare with {a} and {b}");
            return s;
        }
        let moves = node.valid_moves();
        let mut a = a;
        let mut b = b;
        let my_marker = self.my_marker; // take a copy here
        if my_move {
            // In this branch, the AI tries to find a move for itself that would maximize the score
            let mut value = -f64::INFINITY;
            let child_nodes = moves.iter().map(|addr| {
                let mut child = (*node).clone();
                child.place_mark(*addr, my_marker);
                child
            });
            for child in child_nodes {
                let newval = self.alphabeta(&child, depth - 1, a, b, false);
                value = value.max(newval);
                a = a.max(value);
                if value >= b {
                    break;
                }
            }
            value
        } else {
            // In this branch, the AI tries to find a move for the other player that would minimize the score
            let mut value = f64::INFINITY;
            let child_nodes = moves.iter().map(|addr| {
                let mut child = (*node).clone();
                child.place_mark(*addr, my_marker.other());
                child
            });
            for child in child_nodes {
                value = value.min(self.alphabeta(&child, depth - 1, a, b, true));
                b = b.min(value);
                if value <= a {
                    break;
                }
            }
            value
        }
    }
}

impl<B: Board + Clone> BlitzPlayer<B> for ABAi<B> {
    fn blitz(&mut self, b: &B, _time_remaining: std::time::Duration) -> <B as Board>::Coordinate {
        self.play(b)
    }
}


impl<B: Board + Clone> Player<B> for ABAi<B> {
    fn play(&mut self, b: &B) -> B::Coordinate {
        let res = b
            .valid_moves()
            .iter()
            .map(|addr| {
                let mut b2 = (*b).clone();
                b2.place_mark(*addr, self.my_marker);
                let score =
                    self.alphabeta(&b2, self.max_depth, -f64::INFINITY, f64::INFINITY, false);
                (score, addr)
            })
            // .inspect(|x| println!("about to pick the best: {x:?}"))
            .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(_, &q)| q)
            .expect("At least one element");
        res
    }
}

impl<M> Drop for ABAi<M> {
    fn drop(&mut self) {
        debug!("ABAi evaluated {} leaf nodes", self.n_leafs_evaluated);
    }
}