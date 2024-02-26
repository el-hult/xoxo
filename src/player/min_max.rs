use crate::core::{BlitzPlayer, Board, HeuristicFn, Player, PlayerMark};

pub struct MinMaxAi<B> {
    my_marker: PlayerMark,
    /// A performance counter. If we prune well, this number is small
    n_leafs_evaluated: usize,
    heuristic_fn: HeuristicFn<B>,
    max_depth: usize,
}

impl<B: Board + Clone> MinMaxAi<B> {
    pub fn new(mark: PlayerMark, heuristic_fn: HeuristicFn<B>, depth: usize) -> Self {
        Self {
            my_marker: mark,
            n_leafs_evaluated: 0,
            heuristic_fn,
            max_depth: depth,
        }
    }

    /// It is good to win. It is bad to lose.
    /// If we can win, we want to win fast,
    /// If we must lose or tie, we want to lose slowly
    /// It is always good to hold the mid point
    fn heuristic(&mut self, b: &B) -> f64 {
        self.n_leafs_evaluated += 1;
        (self.heuristic_fn)(self.my_marker, b)
    }

    /// compute the score of a node by use of minimax
    /// Assumes I want to maximize my score, and the opponent makes moves to minimize it
    fn minimax(&mut self, node: &B, depth: usize, my_move: bool) -> f64 {
        if depth == 0 || node.game_is_over() {
            let s = self.heuristic(node);
            return s;
        }
        let moves = node.valid_moves();
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
                let newval = self.minimax(&child, depth - 1, false);
                value = value.max(newval);
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
                value = value.min(self.minimax(&child, depth - 1, true));
            }
            value
        }
    }
}

impl<B: Board+Clone> BlitzPlayer<B> for MinMaxAi<B>{
    fn blitz(&mut self, b: &B, _time_remaining: std::time::Duration) -> <B as Board>::Coordinate {
        self.play(b)
    }
}

impl<B: Board + Clone> Player<B> for MinMaxAi<B> {
    fn play(&mut self, b: &B) -> B::Coordinate {
        let res = b
            .valid_moves()
            .iter()
            .map(|addr| {
                let mut b2 = (*b).clone();
                b2.place_mark(*addr, self.my_marker);
                let score = self.minimax(&b2, self.max_depth, false);
                (score, addr)
            })
            .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(_, &q)| q)
            .expect("At least one element");
        res
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use super::*;
    use crate::game::tictactoe::{TTTAddr, TTTBoard};

    pub fn ttt_heuristic(my_marker: PlayerMark, b: &TTTBoard) -> f64 {
        let n_moves_made: f64 = b.n_moves_made() as f64;
        match b.winner() {
            None => 0.0 + n_moves_made,
            Some(mark) => {
                if mark == my_marker {
                    100.0 - n_moves_made
                } else {
                    -100.0 + n_moves_made
                }
            }
        }
    }

    #[test]
    fn can_find_winning_move() {
        let b = TTTBoard::from_str("   xx    ").unwrap();
        let mut ai = MinMaxAi::<TTTBoard>::new(PlayerMark::Cross, ttt_heuristic, 10);
        let action: TTTAddr = ai.play(&b);
        assert_eq!(action, TTTAddr(6))
    }
    #[test]
    fn can_block_winning_move() {
        let b = TTTBoard::from_str("oo  x    ").unwrap();
        let mut ai = MinMaxAi::<TTTBoard>::new(PlayerMark::Cross, ttt_heuristic, 10);
        let action = ai.play(&b);
        assert_eq!(action, TTTAddr(3))
    }
}
