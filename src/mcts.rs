//! Monte Carlo Tree Search for simple Tic-Tac-Toe
//!
//! This module is very much work in progress, and is not yet used in the main program.

use rand::prelude::SliceRandom;
use std::hash::Hash;
use std::{collections::HashMap, fmt::Debug};

pub(crate) trait Mdp {
    type Action: Clone + Debug + PartialEq + Eq + Hash;
    type State: Sized + Debug + Clone + PartialEq + Eq + Hash;
    const DISCOUNT_FACTOR: f64; // 1= no discount, 0=only immediate reward
    fn act(s: Self::State, action: &Self::Action) -> (Self::State, f64); // This is sampling from the Sutton&Barto's p(s',r|s,a), equation 3.2
    fn is_terminal(s: &Self::State) -> bool;
    fn allowed_actions(s: &Self::State) -> Vec<Self::Action>;
    /// If the game branch factor is large, this random strategy is bad, since it will explore very inefficiently
    /// What better enginges do is to use some heuristic for the Q-function to do the rollout.
    fn rollout(s: Self::State) -> f64 {
        if Self::is_terminal(&s) {
            return 0.0;
        }
        let actions = Self::allowed_actions(&s);
        let action = actions.choose(&mut rand::thread_rng()).expect(
            "This function should never have been called on a state with no actions allowed",
        );
        let (state, reward) = Self::act(s, action);
        reward + Self::DISCOUNT_FACTOR * Self::rollout(state)
    }
}

pub(crate) fn mcts_step<M: Mdp>(
    state: &M::State,
    state_visit_counter: &mut HashMap<M::State, f64>,
    qmap: &mut QMap<M>,
) -> f64 {
    if M::is_terminal(state) {
        return 0.0;
    }
    let allowed_actions = M::allowed_actions(state);
    let t = *state_visit_counter.get(state).unwrap_or(&0.0);
    let (best_action, _) = allowed_actions
        .iter()
        .map(|action| {
            let (w, v) = qmap
                .get(&(state.clone(), action.clone()))
                .unwrap_or(&(0.0, 0.0));
            (action, ucb(*w, *v, t))
        })
        .max_by(|(_, ucb1), (_, ucb2)| ucb1.partial_cmp(ucb2).unwrap())
        .unwrap();
    let (new_state, reward) = M::act(state.clone(), best_action);
    let state_was_new = state_visit_counter.get(state).is_none();
    let g_return = if state_was_new {
        reward + M::rollout(new_state) * M::DISCOUNT_FACTOR
    } else {
        reward + mcts_step::<M>(&new_state, state_visit_counter, qmap) * M::DISCOUNT_FACTOR
    };

    // Update the Q-function and the visit counter
    state_visit_counter.insert(state.clone(), t + 1.0);
    let state_action_key = (state.clone(), best_action.clone());
    let (w, v) = qmap.get(&state_action_key).unwrap_or(&(0.0, 0.0));
    qmap.insert(state_action_key, (w + g_return, v + 1.0));

    g_return
}

pub(crate) type QMap<M> = HashMap<(<M as Mdp>::State, <M as Mdp>::Action), (f64, f64)>;

pub(crate) fn best_action<M: Mdp>(
    state: &M::State,
    qmap: &QMap<M>,
    state_visit_counter: &HashMap<M::State, f64>,
) -> M::Action {
    let allowed_actions = M::allowed_actions(state);
    let t = *state_visit_counter.get(state).unwrap_or(&0.0);
    let (best_action, _) = allowed_actions
        .into_iter()
        .map(|action| {
            let (w, v) = qmap
                .get(&(state.clone(), action.clone()))
                .unwrap_or(&(0.0, 0.0));
            (action, ucb(*w, *v, t))
        })
        .max_by(|(_, ucb1), (_, ucb2)| ucb1.partial_cmp(ucb2).unwrap())
        .unwrap();
    best_action
}

/// The UCB1 formula, 
/// the constant is something...
fn ucb(tot_g: f64, n_visists: f64, time: f64) -> f64 {
    let c: f64 = 0.5;
    if n_visists == 0.0 {
        return f64::INFINITY;
    }
    tot_g / n_visists + c * (time.ln() / n_visists).sqrt()
}

#[cfg(test)]
mod test {

    use super::*;
    use rand::{thread_rng, Rng};

    #[derive(Debug, Clone, PartialEq, Hash, Eq)]
    struct CountGameState(
        Vec<i8>
    );
    #[derive(Clone, Debug, PartialEq, Copy, Hash, Eq)]
    enum CountGameAction {
        Add,
        Sub,
    }
    struct CountGameMDP {}
    impl Mdp for CountGameMDP {
        type Action = CountGameAction;
        type State = CountGameState;
        const DISCOUNT_FACTOR: f64 = 0.99;
        fn is_terminal(s: &CountGameState) -> bool {
            let total = s.0.iter().sum::<i8>();
            total <= -10 || total >= 10
        }
        fn act(s: CountGameState, action: &Self::Action) -> (CountGameState, f64) {
            let mut s = s;
            match action {
                CountGameAction::Add => s.0.push(thread_rng().gen_range(-1..=3)),
                CountGameAction::Sub => s.0.push(thread_rng().gen_range(-3..=1)),
            };
            let reward = if s.0.iter().sum::<i8>() >= 10 { 1.0 } else { 0.0 }; // reward is 1.0 for winning
            (s, reward)
        }
        fn allowed_actions(_s: &Self::State) -> Vec<Self::Action> {
            vec![CountGameAction::Add, CountGameAction::Sub]
        }
    }

    // If I take two steps, will both children be visited once?
    #[test]
    fn test_mcts_step() {
        let root: CountGameState = CountGameState(vec![]);
        let mut state_visit_counter = HashMap::new();
        let mut qmap = HashMap::new();
        mcts_step::<CountGameMDP>(&root, &mut state_visit_counter, &mut qmap);
        mcts_step::<CountGameMDP>(&root, &mut state_visit_counter, &mut qmap);
        // The root state should have been visited twice
        assert!(state_visit_counter.contains_key(&root));
        assert_eq!(state_visit_counter[&root], 2.0);
        // check that there are two actions in the qmap associated with the root state
        let visits = qmap
            .iter()
            .filter(|((s, _), (_, _))| s == &root)
            .map(|(_, (_, v))| v)
            .collect::<Vec<_>>();
        dbg!(&qmap);
        assert_eq!(visits.len(), 2);
        assert_eq!(*visits[0], 1.0);
        assert_eq!(*visits[1], 1.0);
    }

    // If I run the game many times, Have I identified the best move?
    #[test]
    fn test_mcts() {
        let root: CountGameState = CountGameState(vec![]);
        let mut state_visit_counter = HashMap::new();
        let mut qmap = HashMap::new();
        for _ in 0..2000 {
            mcts_step::<CountGameMDP>(&root, &mut state_visit_counter, &mut qmap);
        }
        let best_move = best_action::<CountGameMDP>(&root, &qmap, &state_visit_counter);
        assert_eq!(
            best_move,
            CountGameAction::Add,
            "Stochastic test that might fail sometimes"
        );
    }
}
