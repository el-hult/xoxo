//! Monte Carlo Tree Search for simple Tic-Tac-Toe
//!
//! This module is very much work in progress, and is not yet used in the main program.

use rand::prelude::SliceRandom;

type Ucb = f64;
const INFINITY: Ucb = f64::INFINITY;
type Counter = f64;

trait MCTSState: Sized + Copy + PartialEq {
    type Action: Copy;
    fn act(self, action: &Self::Action) -> Self;
    fn is_win(&self) -> bool;
    fn is_terminal(&self) -> bool;
    fn allowed_actions(&self) -> Vec<Self::Action>;
    /// If the game branch factor is large, this random strategy is bad, since it will explore very inefficiently
    /// What better enginges do is to use some heuristic for the Q-function to do the rollout.
    fn random_rollout(&self) -> bool {
        let mut state: Self = *self;
        loop {
            if state.is_win() {
                return true;
            } else if state.is_terminal() {
                return false;
            }
            let actions = state.allowed_actions();
            let action = actions.choose(&mut rand::thread_rng()).unwrap();
            state = state.act(action);
        }
    }
}

#[derive(Debug, PartialEq)]
struct ActionNode<Action,State> {
    action: Action,
    wins: Counter,
    visits: Counter, // visits might be 1 more than the sum of the childrens visits, since the first visist is a MCTS rollout visit
    children: Vec<StateNode<Action,State>>,
}

impl<A,S> ActionNode<A,S> 
where S: MCTSState<Action=A>,
A: Copy
{
    /// As part of MCTS we want to take this action from some state,
    /// observe some reward, update ourselves, and then tell our parent how it went
    fn step_down(&mut self, state: S) -> Counter {
        let new_state = state.act(&self.action);
        let win = if let Some (relevant_child)= self.children.iter_mut().find(|c| c.state == new_state){
            relevant_child.step_mcts()
        } else {
            let mut new_child = StateNode::new(new_state);
            new_child.visits = 1.0;
            self.children.push(new_child);
            let win: Counter = new_state.random_rollout() as u8 as Counter;
            win
        };
        self.wins += win;
        self.visits += 1.0;
        win
    }
}

#[derive(Debug, PartialEq)]
struct StateNode<Action, State> {
    state: State,
    children: Option<Vec<ActionNode<Action,State>>>,
    visits: Counter,
}

impl<A, S> StateNode<A, S>
where
    S:  MCTSState<Action = A>,
    A: Copy
{
    fn new(state: S) -> Self {
        Self {
            state,
            children: None,
            visits: 0.0,
        }
    }
    fn add_child(&mut self, m: A) {
        let mut children = match self.children.take() {
            Some(c) => c,
            None => Vec::new(),
        };
        children.push(ActionNode {
            action: m,
            wins: 0.0,
            visits: 0.0,
            children: Vec::new(),
        });
        self.children = Some(children);
    }
    pub fn best_action(&self) -> Option<S::Action> {
        if let Some(children) = &self.children {
            children
                .iter()
                .max_by(|a, b| {
                    let ucb_a = ucb(a.wins, a.visits, self.visits);
                    let ucb_b = ucb(b.wins, b.visits, self.visits);
                    ucb_a.total_cmp(&ucb_b)
                })
                .map(|bc| bc.action)
        } else {
            None
        }
    }
    /// If there are actions to take, try one.
    /// In other cases, you are terminal state, and can return the score of that
    fn try_best_action(&mut self) -> Counter {
        if self.state.is_terminal() {
            if self.state.is_win() {
                return 1.0;
            } else {
                return 0.0;
            }
        }

        if let Some(ref mut  children) = self.children {
            let best_cild = children.iter_mut().max_by(|a, b| {
                let ucb_a = ucb(a.wins, a.visits, self.visits);
                let ucb_b = ucb(b.wins, b.visits, self.visits);
                ucb_a.total_cmp(&ucb_b)
            });
            let win = best_cild.map(|bc| bc.step_down(self.state));
            return win.unwrap()
            
        }
        panic!("This state should not be terminal, so there should have been some action to take")
    }
    fn action_ucbs(&self) -> Option<Vec<(A, Ucb)>> {
        if let Some(c) = &self.children {
            let v = c.iter().map(|an| {
                let ucb_val = ucb(an.wins, an.visits, self.visits);
                (an.action, ucb_val)
            }).collect();
            Some(v)
        } else {
            None
        }
    }
    /// If you don't have any children yet, add them in!
    fn enumerate_actions(&mut self) {
        assert!(!self.state.is_terminal(),
            "Never try to add actions to terminal state.");
        self.state.allowed_actions().into_iter().for_each(|m| {
            self.add_child(m);
        });
    }
    // Return 1 for win, 0 for not-win
    fn simulate(&self) -> Counter {
        match self.state.random_rollout() {
            true => 1.0,
            false => 0.0,
        }
    }
    /// One step of MCTS traverses the tree down stochastically and then back up
    /// It tells us the reward that it found from the rollout
    /// It will also update all nodes in the tree, so that the next step will be better informed
    fn step_mcts(&mut self) -> Counter {
        let win = 
            // 1. Selection
            // If this node has children, we should pick a child, do MCTS on that, and observe a reward
            if self.children.is_some() {
                self.try_best_action()
            } else if self.state.is_terminal() {
                if self.state.is_win() {
                1.0} else {0.0}
            } else {

                // 2. Expansion
                // We have observed that this node is not terminal, and has no children,
                // thus, we should add in all action node children before continuing
                self.enumerate_actions();

                // 3. Simulation
                // Now that we have actions, pick one of them. Make a roll out.
                self.try_best_action()
            };
        // 4. Backpropagation
        // Since we implemended in a recursive form, I just book-keep myself, and return the reward to my caller
        self.visits += 1.0;
        win
    }
}

fn ucb(w: Counter, v: Counter, t: Counter) -> f64 {
    if v == 0.0 {
        return INFINITY;
    }
     // The UCB1 formula, with a constant of 2.0
    w / v + 2.0 * (t.ln() / v).sqrt()
}

#[cfg(test)]
mod test {

    use super::*;
    use rand::{thread_rng, Rng};


    #[derive(Debug,Clone, Copy, PartialEq)]
    struct CountGameState(i8);
    #[derive(Clone, Debug, PartialEq, Copy)]
    enum CountGameAction {
        Add,
        Sub,
    }
    impl MCTSState for CountGameState {
        type Action = CountGameAction;
        fn is_win(&self) -> bool {
            self.0 >= 10
        }
        fn is_terminal(&self) -> bool {
            self.0 <= -10 || self.0 >= 10
        }
        fn act(mut self, action: &Self::Action) -> Self {
            match action {
                CountGameAction::Add => self.0 += thread_rng().gen_range(-1..=3),
                CountGameAction::Sub => self.0 += thread_rng().gen_range(-3..=1),
            };
            self
        }
        fn allowed_actions(&self) -> Vec<Self::Action> {
            vec![CountGameAction::Add, CountGameAction::Sub]
        }
    }

    // If I take two steps, will both children be visited once?
    #[test]
    fn test_mcts_step() {
        let mut root =
            StateNode::new(CountGameState(0));
        root.step_mcts();
        root.step_mcts();
        let visits: Vec<_> = root.children.unwrap().iter().map(|an| an.visits).collect();
        assert_eq!(visits.len(), 2);
        assert_eq!(visits[0], 1.0);
        assert_eq!(visits[1], 1.0);
    }

    // If I run the game many times, Have I identified the best move?
    #[test]
    fn test_mcts() {
        let mut root =StateNode::new(CountGameState(0));
        for _ in 0..1000 {
            root.step_mcts();
        }
        let best_move = root
            .best_action()
            .expect("This move should have valid moves");
        assert_eq!(best_move, CountGameAction::Add, "Stochastic test that might fail sometimes");
    }
}
