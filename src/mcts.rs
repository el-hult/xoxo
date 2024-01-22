//! Monte Carlo Tree Search for simple Tic-Tac-Toe
//! 
//! This module is very much work in progress, and is not yet used in the main program.

use std::{cell::RefCell, rc::Rc};

use super::tictactoe::TTTAddr;


#[derive(Debug, Clone, PartialEq, Eq)]
struct TreeNode {
    parent: Option<Rc<RefCell<TreeNode>>>,
    children: Vec<(TTTAddr,Rc<RefCell<TreeNode>>)>,
}

impl TreeNode {
    fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self { parent: None, children: vec![] }))
    }
    fn add_child(parent: Rc<RefCell<TreeNode>>, m: TTTAddr) {
        let child = TreeNode::new();
        child.borrow_mut().parent = Some(parent.clone());
        parent.borrow_mut().children.push((m,child));
    }
}

#[cfg(test)]
mod test{

    use super::*;

    #[test]
    fn test_tree_node() {
        let root = TreeNode::new();
        TreeNode::add_child(root.clone(),TTTAddr(1));
        let root2 = root.borrow();
        let child = root2.children[0].1.borrow();
        let child_parent = match child.parent {
            Some(ref p) => p,
            None => panic!("No parent!")
        };
        assert!(Rc::ptr_eq(child_parent, &root));
    }
}