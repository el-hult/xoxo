use std::collections::HashMap;

use crate::{core::Player, mcts::{best_action, mcts_step, Mdp, QMap}, tictactoe::{TTTAddr, TTTBoard, TicTacToe}};

pub(crate) struct MctsAi<T:Mdp> {
    qmap: QMap<T>,
    state_visit_counter: HashMap<T::State, f64>,
}

impl<T:Mdp> MctsAi<T> {
    pub fn new() -> Self {
        MctsAi {
            qmap: QMap::<T>::new(),
            state_visit_counter: HashMap::new(),
        }
    }
}

impl Player<TicTacToe> for MctsAi<TicTacToe>{
    fn play(&mut self, b: &TTTBoard) -> TTTAddr {
        for _ in 0..1000 {
            mcts_step::<TicTacToe>(b, &mut self.state_visit_counter, &mut self.qmap);
        }
        
        best_action::<TicTacToe>(b, &self.qmap, &self.state_visit_counter)
    }
}
