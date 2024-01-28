//! Monte Carlo Tree Search for simple Tic-Tac-Toe
//!
//! This module is very much work in progress, and is not yet used in the main program.

use rand::prelude::SliceRandom;
use std::{
    cell::{RefCell, RefMut},
    rc::Rc,
};

type Ucb = f64;
const INFINITY: Ucb = f64::INFINITY;
type NodePtr<Action, State> = Rc<RefCell<TreeNode<Action, State>>>;
type Counter = f64;

trait GameState: Sized + Copy {
    type Action;
    fn act(self, action: &Self::Action) -> Self;
    fn is_win(&self) -> bool;
    fn is_loss(&self) -> bool;
    fn allowed_actions(&self) -> Vec<Self::Action>;
    fn rollout(&self) -> bool {
        let mut state: Self = *self;
        loop {
            let actions = state.allowed_actions();
            let action = actions.choose(&mut rand::thread_rng()).unwrap();
            state = state.act(action);
            if state.is_win() {
                return true;
            } else if state.is_loss() {
                return false;
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct TreeNode<Action, State> {
    parent: Option<NodePtr<Action, State>>,
    state: State,
    children: Vec<(Action, Ucb, NodePtr<Action, State>)>,
    wins: Counter,
    visits: Counter,
}

impl<Action, State> TreeNode<Action, State>
where
    State: Clone + GameState<Action = Action>,
    Action: Clone,
{
    fn new(state: State) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            parent: None,
            state,
            children: vec![],
            wins: 0.0,
            visits: 0.0,
        }))
    }
    fn add_child(parent: NodePtr<Action, State>, m: Action) {
        let child_state = parent.borrow().state.act(&m);
        let child = TreeNode::new(child_state);
        child.borrow_mut().parent = Some(parent.clone());
        parent.borrow_mut().children.push((m, INFINITY, child));
    }
    fn select_child(&self) -> Option<NodePtr<Action, State>> {
        self.children
            .iter()
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .map(|m| m.2.clone())
    }
    fn expand_children(self_ptr: NodePtr<Action, State>) {
        let actions = self_ptr.borrow().state.allowed_actions();
        actions.into_iter().for_each(|m| {
            TreeNode::add_child(self_ptr.clone(), m);
        });
    }
    // Return 1 for win, 0 for not-win
    fn simulate(&self) -> Counter {
        match self.state.rollout() {
            true => 1.0,
            false => 0.0,
        }
    }
    fn update_child_ucbs(&mut self) {
        let t = self.visits;
        for (_, ucb, c) in self.children.iter_mut() {
            let r: RefMut<'_, TreeNode<Action, State>> = c.borrow_mut();
            let w = r.wins;
            let v = r.visits;
            *ucb = if v == 0.0 {
                INFINITY
            } else {
                w / v + 2.0 * (t).ln() / v
            }
        }
    }
    fn best_action(&self) -> Option<Action> {
        self.children
            .iter()
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .map(|m| m.0.clone())
    }
}
fn step_mcts<Action, State>(root: &NodePtr<Action, State>)
where
    State: Clone + GameState<Action = Action>,
    Action: Clone,
{
    // 1. Selection
    let mut curr_node: NodePtr<Action, State> = root.clone();
    loop {
        let c2 = curr_node.borrow().select_child();
        if let Some(c) = c2 {
            curr_node = c.clone();
        } else {
            break;
        }
    }

    // 2. Expansion
    TreeNode::expand_children(curr_node.clone());

    // 3. Simulation
    let t = curr_node
        .borrow()
        .select_child()
        .expect("I just expanded the children. There should be children!")
        .clone();
    curr_node = t;
    let win: Counter = curr_node.borrow().simulate();

    // 4. Backpropagation
    loop {
        {
            let mut curr_node_mut = curr_node.borrow_mut();
            curr_node_mut.visits += 1.0;
            curr_node_mut.wins += win;
            curr_node_mut.update_child_ucbs();
        }
        let maybe_parent = curr_node.borrow().parent.clone();
        curr_node = match maybe_parent {
            Some(p) => p.clone(),
            None => break,
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;

    /// Can I create a child with a valid, working back pointer?
    #[test]
    fn test_tree_node() {
        let root = TreeNode::new(CountGameState(0));
        TreeNode::add_child(root.clone(), CountGameAction::Add);
        let root2 = root.borrow();
        let child = root2.children[0].2.borrow();
        let child_parent = match child.parent {
            Some(ref p) => p,
            None => panic!("No parent!"),
        };
        assert!(Rc::ptr_eq(child_parent, &root));
    }

    #[derive(Clone, Copy)]
    struct CountGameState(i8);
    #[derive(Clone, Debug, PartialEq)]
    enum CountGameAction {
        Add,
        Sub,
    }
    impl GameState for CountGameState {
        type Action = CountGameAction;
        fn is_win(&self) -> bool {
            self.0 == 100
        }
        fn is_loss(&self) -> bool {
            self.0 == -100
        }
        fn act(mut self, action: &Self::Action) -> Self {
            match action {
                CountGameAction::Add => self.0 += 1,
                CountGameAction::Sub => self.0 -= 1,
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
        let root: Rc<RefCell<TreeNode<CountGameAction, CountGameState>>> =
            TreeNode::new(CountGameState(0));
        step_mcts(&root);
        step_mcts(&root);
        let bind = root.borrow();
        let visits: Vec<_> = bind.children.iter().map(|(_, _, c)| c).collect();
        assert_eq!(visits.len(), 2);
        assert_eq!(visits[0].borrow().visits, 1.0);
        assert_eq!(visits[1].borrow().visits, 1.0);
    }

    // If I run the game 100 times, Have I identified the best move?
    #[test]
    fn test_mcts() {
        let root: Rc<RefCell<TreeNode<CountGameAction, CountGameState>>> =
            TreeNode::new(CountGameState(0));
        for _ in 0..100 {
            step_mcts(&root);
        }
        let best_move = root
            .borrow()
            .best_action()
            .expect("This move should have valid moves");
        assert_eq!(best_move, CountGameAction::Add);
    }
}
