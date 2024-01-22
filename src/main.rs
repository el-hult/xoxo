mod alpha_beta;
mod console_player;
mod core;
mod min_max;
mod random_ai;
mod tictactoe;
mod ultimate_ttt;
mod mcts;

use alpha_beta::ABAi;
use clap::{Parser, ValueEnum};
use console_player::ConsolePlayer;
use core::{Game, Player, PlayerMark};
use min_max::MinMaxAi;
use random_ai::RandomAi;
use std::{f64::INFINITY, fmt::Display, io::BufRead};
use tictactoe::TicTacToe;
use ultimate_ttt::UltimateTicTacToe;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum PlayerType {
    Console,
    Random,
    Minimax,
    AlphaBeta,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum GameType {
    /// Normal Tic-Tac-Toe
    Ttt,
    /// Ultimate Tic-Tac-Toe
    Uttt,
}

/// A Tic-Tac-Toe game for the command line, with a cool AI integrated!
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Player 1 type
    #[arg(long, default_value = "console")]
    p1: PlayerType,

    /// Player 2 type
    #[arg(long, default_value = "alpha-beta")]
    p2: PlayerType,

    /// Which game to play
    #[arg(long, default_value = "ttt")]
    game: GameType,

    /// The depth of the minimax algorithm
    /// Only used for minimax ai, if used
    #[arg(long, default_value = "3")]
    mm_depth: usize,

    /// The depth of the alpha-beta algorithm
    /// Only used for alpha-beta ai, if used
    #[arg(long, default_value = "8")]
    ab_depth: usize,

    /// The seed for the random number generator (when used)
    #[arg(long)]
    seed: Option<u64>,
}

fn ttt_heuristic(my_marker: PlayerMark, b: &<TicTacToe as Game>::Board) -> f64 {
    let n_moves_made: f64 = b.n_moves_made();
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
/// Mentioned in the thread https://boardgames.stackexchange.com/questions/49291/strategy-for-ultimate-tic-tac-toe
/// two papers on the topic are referred to:
/// https://www.cs.huji.ac.il/%7Eai/projects/2013/UlitmateTic-Tac-Toe/files/report.pdf
/// http://smpowell.com/wp-content/uploads/2021/07/Powell_Merrill_FinalPaper.pdf
///
fn uttt_heuristic(my_marker: PlayerMark, b: &<UltimateTicTacToe as Game>::Board) -> f64 {
    let n_moves_made: f64 = b.n_moves_made() as f64;
    let n_supboards_win_balance: isize = b
        .get_sup_board()
        .iter()
        .flatten()
        .map(|&x| match x {
            ultimate_ttt::BoardStatus::Won(marker) => {
                if marker == my_marker {
                    1
                } else {
                    -1
                }
            }
            _ => 0,
        })
        .sum();
    let did_win_mid_supboard =
        (b.get_sup_board()[1][1] == ultimate_ttt::BoardStatus::Won(my_marker)) as u8 as f64;
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
        None => 0.0,
        Some(None) => 0.0,
        Some(Some(mark)) => {
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

fn main() {
    let args = Args::parse();
    match args.game {
        GameType::Ttt => {
            type G = TicTacToe;
            let p1: Box<dyn Player<G>> = match args.p1 {
                PlayerType::Console => Box::new(ConsolePlayer::new(PlayerMark::Naught)),
                PlayerType::Random => Box::new(RandomAi::new(PlayerMark::Naught, args.seed)),
                PlayerType::Minimax => Box::new(MinMaxAi::new(
                    PlayerMark::Naught,
                    ttt_heuristic,
                    args.mm_depth,
                )),
                PlayerType::AlphaBeta => {
                    Box::new(ABAi::new(PlayerMark::Naught, ttt_heuristic, args.ab_depth))
                }
            };
            let p2: Box<dyn Player<G>> = match args.p2 {
                PlayerType::Console => Box::new(ConsolePlayer::new(PlayerMark::Cross)),
                PlayerType::Random => Box::new(RandomAi::new(PlayerMark::Cross, args.seed)),
                PlayerType::Minimax => Box::new(MinMaxAi::new(
                    PlayerMark::Cross,
                    ttt_heuristic,
                    args.mm_depth,
                )),
                PlayerType::AlphaBeta => {
                    Box::new(ABAi::new(PlayerMark::Cross, ttt_heuristic, args.ab_depth))
                }
            };
            let mut g = G::new(p1, p2);
            g.run()
        }
        GameType::Uttt => {
            type G = UltimateTicTacToe;
            let p1: Box<dyn Player<G>> = match args.p1 {
                PlayerType::Console => Box::new(ConsolePlayer::new(PlayerMark::Naught)),
                PlayerType::Random => Box::new(RandomAi::new(PlayerMark::Naught, args.seed)),
                PlayerType::Minimax => Box::new(MinMaxAi::new(
                    PlayerMark::Naught,
                    uttt_heuristic,
                    args.mm_depth,
                )),
                PlayerType::AlphaBeta => {
                    Box::new(ABAi::new(PlayerMark::Naught, uttt_heuristic, args.ab_depth))
                }
            };
            let p2: Box<dyn Player<G>> = match args.p2 {
                PlayerType::Console => Box::new(ConsolePlayer::new(PlayerMark::Cross)),
                PlayerType::Random => Box::new(RandomAi::new(PlayerMark::Cross, args.seed)),
                PlayerType::Minimax => Box::new(MinMaxAi::new(
                    PlayerMark::Cross,
                    uttt_heuristic,
                    args.mm_depth,
                )),
                PlayerType::AlphaBeta => {
                    Box::new(ABAi::new(PlayerMark::Cross, uttt_heuristic, args.ab_depth))
                }
            };
            let mut g = G::new(p1, p2);
            g.run()
        }
    };
}
