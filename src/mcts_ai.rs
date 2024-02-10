use std::collections::HashMap;


use rand::{rngs::StdRng, SeedableRng as _};

use crate::{core::Player, mcts::{best_action, run_train_steps, Mdp, QMap}, tictactoe::{TTTAddr, TTTBoard, TicTacToe}, ultimate_ttt::{self, UltimateTicTacToe}};

pub(crate) struct MctsAi<T:Mdp> {
    qmap: QMap<T>,
    state_visit_counter: HashMap<T::State, f64>,
    rng: StdRng,
}

impl<T:Mdp> MctsAi<T> {
    pub fn new(seed: u64) -> Self {
        MctsAi {
            qmap: QMap::<T>::new(),
            state_visit_counter: HashMap::new(),
            rng: StdRng::seed_from_u64(seed)
            }
        }
    }

impl Player<TicTacToe> for MctsAi<TicTacToe>{
    fn play(&mut self, b: &TTTBoard) -> TTTAddr {
        run_train_steps::<TicTacToe>(b, &mut self.qmap, &mut self.state_visit_counter, &mut self.rng,2000);
        best_action::<TicTacToe>(b, &self.qmap, &self.state_visit_counter, &mut self.rng)
    }
}
impl Player<UltimateTicTacToe> for MctsAi<UltimateTicTacToe>{
    fn play(&mut self, b: &ultimate_ttt::Board) -> ultimate_ttt::Action {
        run_train_steps::<UltimateTicTacToe>(b, &mut self.qmap, &mut self.state_visit_counter, &mut self.rng,2000);
        best_action::<UltimateTicTacToe>(b, &self.qmap, &self.state_visit_counter, &mut self.rng)
    }
}
