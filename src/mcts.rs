//! Monte Carlo Tree Search for simple Tic-Tac-Toe
//!
//! This module is very much work in progress, and is not yet used in the main program.

use std::fmt::Debug;

use rand::prelude::SliceRandom;

pub(crate) trait Mdp {
    type Action: Clone + Debug + PartialEq;
    type State: Sized + Debug + Clone + PartialEq;
    const DISCOUNT_FACTOR: f64; // 1= no discount, 0=only immediate reward
    fn act(s: Self::State, action: &Self::Action) -> (Self::State, f64); // This is sampling from the Sutton&Barto's p(s',r|s,a), equation 3.2
    fn is_terminal(s: &Self::State) -> bool;
    fn allowed_actions(s: &Self::State) -> Vec<Self::Action>;
    /// If the game branch factor is large, this random strategy is bad, since it will explore very inefficiently
    /// What better enginges do is to use some heuristic for the Q-function to do the rollout.
    fn rollout(s: Self::State) -> f64 {
        let actions = Self::allowed_actions(&s);
        let action = actions.choose(&mut rand::thread_rng()).unwrap();
        let (state, reward) = Self::act(s, action);
        if Self::is_terminal(&state) {
            reward
        } else {
            reward + Self::DISCOUNT_FACTOR * Self::rollout(state)
        }
    }
}

#[derive(Debug, PartialEq)]
struct ActionNode<T: Mdp> {
    action: T::Action,
    tot_reward: f64,
    visits: f64, // visits might be 1 more than the sum of the childrens visits, since the first visist is a MCTS rollout visit
    children: Vec<StateNode<T>>,
}

impl<S: Mdp> ActionNode<S> {
    /// If we took the action in `self` from a certain state,
    /// we will make a mcts step down. That means either
    /// 1. step into a node that was already expanded, then step in to that State node, continue with the node selection (MCTS phase 1)
    /// 2. step into a NEW node. In that case, we make Rollout (MCTS phase 3)
    /// Return the relevant reward observed to the parent
    fn mcts_step(&mut self, state: &S::State) -> f64 {
        let (new_state, reward0) = S::act(state.clone(), &self.action);

        let seen_state_node = self
            .children
            .iter_mut()
            .find(|child| child.state == new_state);
        let reward1 = if let Some(relevant_child) = seen_state_node {
            relevant_child.mcts_step()
        } else {
            let mut new_child = StateNode::new(new_state.clone());
            new_child.visits = 1.0;
            self.children.push(new_child);
            S::rollout(new_state)
        };
        let reward = reward0 + S::DISCOUNT_FACTOR * reward1;
        self.tot_reward += reward;
        self.visits += 1.0;
        reward
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct StateNode<T: Mdp> {
    state: T::State,
    children: Option<Vec<ActionNode<T>>>,
    visits: f64,
}

impl<S: Mdp> StateNode<S> {
    pub fn new(state: S::State) -> Self {
        Self {
            state,
            children: None,
            visits: 0.0,
        }
    }
    pub fn get_state(&self) -> &S::State {
        &self.state
    }
    pub fn best_action(&self) -> Option<&S::Action> {
        if let Some(children) = &self.children {
            children
                .iter()
                .max_by(|a, b| {
                    let ucb_a = ucb(a.tot_reward, a.visits, self.visits);
                    let ucb_b = ucb(b.tot_reward, b.visits, self.visits);
                    ucb_a.total_cmp(&ucb_b)
                })
                .map(|bc| &bc.action)
        } else {
            None
        }
    }

    /// If you have a list of actions one could take from this state, return a vector with UCB-action pairs
    fn action_q_ucbs(&self) -> Option<Vec<(&S::Action, f64, f64)>> {
        if let Some(c) = &self.children {
            let v = c
                .iter()
                .map(|an| {
                    let ucb_val = ucb(an.tot_reward, an.visits, self.visits);
                    (&an.action, an.tot_reward/ an.visits, ucb_val)
                })
                .collect();
            Some(v)
        } else {
            None
        }
    }
    /// If you don't have any children yet, add them in!
    fn enumerate_actions(&mut self) {
        if self.children.is_some() {
            panic!("We should not enumerate actions if we already have done so!")
        }
        let mut children = Vec::new();
        S::allowed_actions(&self.state).into_iter().for_each(|m| {
            children.push(ActionNode {
                action: m,
                tot_reward: 0.0,
                visits: 0.0,
                children: Vec::new(),
            });
        });
        self.children = Some(children);
    }

    /// MCTS
    /// In MCTS, when dealing with a state-node, we can only do thing:
    /// 1. take the best action from the current state
    /// 2. record that you visited this state
    pub fn mcts_step(&mut self) -> f64 {
        self.visits += 1.0;
        if S::is_terminal(&self.state) {
            return 0.0; // No actions can be taken from a terminal state. And reward is only given when taking actions.
        }
        if self.children.is_none() {
            self.enumerate_actions();
        }
        let children = self.children.as_mut().expect("We should have children here. If we don't, we should have added them in the previous step");
        let best_action = children
            .iter_mut()
            .max_by(|a, b| {
                let ucb_a = ucb(a.tot_reward, a.visits, self.visits);
                let ucb_b = ucb(b.tot_reward, b.visits, self.visits);
                ucb_a.total_cmp(&ucb_b)
            })
            .expect("There must be at least one child. This state is not terminal.");
        best_action.mcts_step(&self.state)
    }
}

/// The UCB1 formula, with a constant of 2.0
fn ucb(w: f64, v: f64, t: f64) -> f64 {
    if v == 0.0 {
        return f64::INFINITY;
    }
    // The UCB1 formula, with a constant of 2.0
    w / v + 2.0 * (t.ln() / v).sqrt()
}

#[cfg(test)]
mod test {

    use super::*;
    use rand::{thread_rng, Rng};

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct CountGameState(i8);
    #[derive(Clone, Debug, PartialEq, Copy)]
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
            s.0 <= -10 || s.0 >= 10
        }
        fn act(s: CountGameState, action: &Self::Action) -> (CountGameState, f64) {
            let mut s = s;
            match action {
                CountGameAction::Add => s.0 += thread_rng().gen_range(-1..=3),
                CountGameAction::Sub => s.0 += thread_rng().gen_range(-3..=1),
            };
            let reward = if s.0 >= 10 { 1.0 } else { 0.0 }; // reward is 1.0 for winning
            (s, reward)
        }
        fn allowed_actions(s: &Self::State) -> Vec<Self::Action> {
            vec![CountGameAction::Add, CountGameAction::Sub]
        }
    }

    // If I take two steps, will both children be visited once?
    #[test]
    fn test_mcts_step() {
        let mut root: StateNode<CountGameMDP> = StateNode::new(CountGameState(0));
        root.mcts_step();
        root.mcts_step();
        let visits: Vec<_> = root.children.unwrap().iter().map(|an| an.visits).collect();
        assert_eq!(visits.len(), 2);
        assert_eq!(visits[0], 1.0);
        assert_eq!(visits[1], 1.0);
    }

    // If I run the game many times, Have I identified the best move?
    #[test]
    fn test_mcts() {
        let mut root: StateNode<CountGameMDP> = StateNode::new(CountGameState(0));
        for _ in 0..2000 {
            root.mcts_step();
        }
        let best_move = root
            .best_action()
            .expect("This move should have valid moves");
        dbg!(root.action_q_ucbs());
        assert_eq!(
            *best_move,
            CountGameAction::Add,
            "Stochastic test that might fail sometimes"
        );
    }
}
