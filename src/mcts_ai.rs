use std::collections::HashMap;


use rand::{rngs::StdRng, SeedableRng as _};

use crate::{core::{Game, Player}, mcts::{best_action, run_train_steps, Mdp, QMap}};

pub(crate) struct MctsAi<T:Mdp> {
    qmap: QMap<T>,
    state_visit_counter: HashMap<T::State, f64>,
    rng: StdRng,
    c: f64
}

impl<T:Mdp> MctsAi<T> {
    /// seed is for the RNG, c is the exploration constant in the UCB1 formula
    pub fn new(seed: u64, c:f64) -> Self {
        MctsAi {
            qmap: QMap::<T>::new(),
            state_visit_counter: HashMap::new(),
            rng: StdRng::seed_from_u64(seed),
            c
            }
        }
    }

impl<T,A,B> Player<T> for MctsAi<T>
where T: Mdp<Action=A,State=B> + Game<Board=B,Coordinate=A>,
{
    fn play(&mut self, b: &B) -> A {
        run_train_steps::<T>(b, self.c, &mut self.qmap, &mut self.state_visit_counter, &mut self.rng,10000);
        best_action::<T>(b, self.c, &self.qmap, &self.state_visit_counter, &mut self.rng)
    }
}

pub(crate) fn get_c(game: crate::GameType) -> f64 {
    match  game {
        crate::GameType::Ttt => 1.0,
        crate::GameType::Uttt => 0.75,
        crate::GameType::C4 => 1.0,
    }
}
