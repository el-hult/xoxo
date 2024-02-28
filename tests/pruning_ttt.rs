//! Integration test pruning minimax AI with TTT
use std::str::FromStr;

use xoxo::{core::{Player, PlayerMark}, game::tictactoe::{TTTAddr, TTTBoard}, player::{ttt_heuristic, ABAi}};

#[test]
fn can_find_winning_move() {
    let b = TTTBoard::from_str("   xx    ").unwrap();
    let mut ai = ABAi::<TTTBoard>::new(PlayerMark::Cross, ttt_heuristic, 10);
    let action: TTTAddr = ai.play(&b);
    assert_eq!(action, TTTAddr(6))
}
#[test]
fn can_block_winning_move() {
    let b = TTTBoard::from_str("oo  x    ").unwrap();
    let mut ai = ABAi::<TTTBoard>::new(PlayerMark::Cross, ttt_heuristic, 10);
    let action = ai.play(&b);
    assert_eq!(action, TTTAddr(3))
}