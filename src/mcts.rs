//! Monte Carlo Tree Search for simple Tic-Tac-Toe
//!
//! This module is very much work in progress, and is not yet used in the main program.

use std::{
    cell::{RefCell, RefMut},
    rc::Rc,
};

type UCB = f64;
const INFINITY: UCB = f64::INFINITY;
type NodePtr<T> = Rc<RefCell<TreeNode<T>>>;
type Counter = f64;

#[derive(Debug, Clone, PartialEq)]
struct TreeNode<T> {
    parent: Option<NodePtr<T>>,
    children: Vec<(T, UCB, NodePtr<T>)>,
    wins: Counter,
    visits: Counter,
}

impl<T> TreeNode<T> {
    fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            parent: None,
            children: vec![],
            wins: 0.0,
            visits: 0.0,
        }))
    }
    fn add_child(parent: NodePtr<T>, m: T) {
        let child = TreeNode::new();
        child.borrow_mut().parent = Some(parent.clone());
        parent.borrow_mut().children.push((m, INFINITY, child));
    }
    fn select_child(&self) -> Option<NodePtr<T>> {
        self.children
            .iter()
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .map(|m| m.2.clone())
    }
    fn expand_children(&mut self) {
        todo!("Create all child nodes based on the allowed moves in this position")
    }
    fn simulate(&self) -> Counter {
        todo!("Play a game where all moves are random, and return if there was a win or loss. Return 1 for win, 0 for loss.")
    }
    fn update_child_ucbs(&mut self) {
        let t = self.visits;
        for (_, ucb, c) in self.children.iter_mut() {
            let r: RefMut<'_, TreeNode<T>> = c.borrow_mut();
            let w = r.wins;
            let v = r.visits;
            *ucb = w as f64 / v as f64 + 2.0 * (t as f64).ln() / v as f64;
        }
    }
}
fn step_mcts<T>(root: NodePtr<T>) {
    // 1. Selection
    let mut curr_node: NodePtr<T> = root;
    loop {
        let c2 = curr_node.borrow().select_child();
        if let Some(c) = c2 {
            curr_node = c.clone();
        } else {
            break;
        }
    }

    // 2. Expansion
    curr_node.borrow_mut().expand_children();

    // 3. Simulation
    let node_to_simulate = curr_node
        .borrow()
        .select_child()
        .expect("I just expanded the children. There should be children!")
        .clone();
    let win: Counter = node_to_simulate.borrow().simulate();

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

    use crate::tictactoe::TTTAddr;

    use super::*;

    #[test]
    fn test_tree_node() {
        let root = TreeNode::new();
        TreeNode::add_child(root.clone(), TTTAddr(1));
        let root2 = root.borrow();
        let child = root2.children[0].2.borrow();
        let child_parent = match child.parent {
            Some(ref p) => p,
            None => panic!("No parent!"),
        };
        assert!(Rc::ptr_eq(child_parent, &root));
    }
}
