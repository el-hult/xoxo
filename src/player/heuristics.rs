use std::f64::INFINITY;

use crate::{
    core::{GameStatus, PlayerMark},
    game::{connect_four::C4Board, tictactoe::TTTBoard, ultimate_ttt::UTTTBoard},
};

pub fn ttt_heuristic(my_marker: PlayerMark, b: &TTTBoard) -> f64 {
    let n_moves_made: f64 = b.n_moves_made() as f64;
    match b.winner() {
        None => 0.0 + n_moves_made,
        Some(mark) => {
            if mark == my_marker {
                100.0 - n_moves_made
            } else {
                -100.0 + n_moves_made
            }
        }
    }
}
/// A variant of the heurstic of Powell and Merrill for Ultimate Tic-Tac-Toe
/// Mentioned in the thread <https://boardgames.stackexchange.com/questions/49291/strategy-for-ultimate-tic-tac-toe>
/// two papers on the topic are referred to:
/// <https://www.cs.huji.ac.il/%7Eai/projects/2013/UlitmateTic-Tac-Toe/files/report.pdf>
/// <http://smpowell.com/wp-content/uploads/2021/07/Powell_Merrill_FinalPaper.pdf>
///
pub fn uttt_heuristic(my_marker: PlayerMark, b: &UTTTBoard) -> f64 {
    let n_moves_made: f64 = b.n_moves_made() as f64;
    let n_supboards_win_balance: isize = b
        .get_sup_board()
        .iter()
        .flatten()
        .map(|&x| match x {
            GameStatus::Won(marker) => {
                if marker == my_marker {
                    1
                } else {
                    -1
                }
            }
            _ => 0,
        })
        .sum();
    let did_win_mid_supboard = (b.get_sup_board()[1][1] == GameStatus::Won(my_marker)) as u8 as f64;
    let midpoint_balance = {
        let board = b.get_board();
        let mut n = 0;
        for sub_board in board.iter().flatten() {
            n += match sub_board[1][1] {
                None => 0,
                Some(PlayerMark::Cross) => -1,
                Some(PlayerMark::Naught) => 1,
            }
        }
        n as f64
    };
    let win_bonus = match b.get_winner() {
        GameStatus::Undecided | GameStatus::Draw => 0.0,
        GameStatus::Won(mark) => {
            if mark == my_marker {
                INFINITY
            } else {
                -INFINITY
            }
        }
    };
    win_bonus
        + n_moves_made * 1.0
        + n_supboards_win_balance as f64 * 100.0
        + did_win_mid_supboard * 30.0
        + 10.0 * midpoint_balance
}

pub fn c4_heuristic(my_marker: PlayerMark, b: &C4Board) -> f64 {
    let raw_board: [[Option<PlayerMark>; 6]; 7] = (*b).into();
    let markers_in_col_3 = raw_board[2]
        .iter()
        .filter(|&&x| x == Some(my_marker))
        .count() as f64;
    let markers_in_col_4 = raw_board[3]
        .iter()
        .filter(|&&x| x == Some(my_marker))
        .count() as f64;
    let markers_in_col_5 = raw_board[4]
        .iter()
        .filter(|&&x| x == Some(my_marker))
        .count() as f64;
    let three_in_rows = {
        let mut k = 0;
        for row in 0..6 {
            let mut j = 0;
            for col in raw_board.iter().take(4) {
                if col[row] == Some(my_marker) {
                    j += 1;
                } else {
                    continue;
                }
            }
            if j == 3 {
                k += 1;
            }
        }
        k as f64
    };
    let win = match b.winner() {
        Some(mark) => {
            if mark == my_marker {
                1.0
            } else {
                -1.0
            }
        }
        _ => 0.0,
    };
    100.0 * win + markers_in_col_3 + 2.0 * markers_in_col_4 + markers_in_col_5 + 5.0 * three_in_rows
}
