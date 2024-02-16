mod core;
mod game;
mod player;

use player::alpha_beta::ABAi;
use clap::{Parser, ValueEnum};
use player::console::ConsolePlayer;
use core::{Game, GameStatus, Player, PlayerMark};
use game::connect_four::{C4Board, ConnectFour};
use game::tictactoe::TicTacToe;
use game::ultimate_ttt::UltimateTicTacToe;
use player::min_max::MinMaxAi;
use rand::{rngs::StdRng, Rng as _, SeedableRng as _};
use player::random::RandomAi;
use std::f64::INFINITY;

use crate::core::run_game;
use crate::game::tictactoe::{TTTAddr, TTTBoard};
use crate::game::ultimate_ttt;
use crate::player::mcts::{get_c, MctsAi};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum PlayerType {
    Console,
    Random,
    Minimax,
    AlphaBeta,
    Mcts,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum GameType {
    /// Normal Tic-Tac-Toe
    Ttt,
    /// Ultimate Tic-Tac-Toe
    Uttt,
    /// Connect Four
    C4,
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
    #[arg(long, default_value = "7")]
    ab_depth: usize,

    /// The seed for the random number generator (when used)
    #[arg(long)]
    seed: Option<u64>,

    /// The exploration constant for the MCTS algorithm
    /// Only used for MCTS ai, if used
    /// If None, the value is determined by game-specific deafults
    #[arg(long)]
    c: Option<f64>,
}

fn ttt_heuristic(my_marker: PlayerMark, b: &<TicTacToe as Game>::Board) -> f64 {
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

/// This heuristic is too crude
/// Therefore, it does not generate a lot of pruning (since many states are considered equally good, few are pruned)
fn c4_heuristic(my_marker: PlayerMark, b: &C4Board) -> f64 {
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
    100.0 * win + markers_in_col_3 + 2.0 * markers_in_col_4 + markers_in_col_5
}

fn main() {
    let args = Args::parse();
    let seed = args.seed.unwrap_or(StdRng::from_entropy().gen());
    println!("AI seed: {}", seed); // debug output
    let mut rng = StdRng::seed_from_u64(seed);
    let c = match args.c {
        Some(c) => c,
        None => get_c(args.game),
    };
    match args.game {
        GameType::Ttt => {
            let p1: Box<dyn Player<TTTBoard,TTTAddr>> = match args.p1 {
                PlayerType::Console => Box::new(ConsolePlayer::new(PlayerMark::Naught)),
                PlayerType::Random => Box::new(RandomAi::new(PlayerMark::Naught, rng.gen())),
                PlayerType::Minimax => Box::new(MinMaxAi::<TicTacToe>::new(
                    PlayerMark::Naught,
                    ttt_heuristic,
                    args.mm_depth,
                )),
                PlayerType::AlphaBeta => {
                    Box::new(ABAi::<TicTacToe>::new(PlayerMark::Naught, ttt_heuristic, args.ab_depth))
                }
                PlayerType::Mcts => Box::new(MctsAi::<TicTacToe>::new(rng.gen(), c)),
            };
            let p2: Box<dyn Player<TTTBoard,TTTAddr>> = match args.p2 {
                PlayerType::Console => Box::new(ConsolePlayer::new(PlayerMark::Cross)),
                PlayerType::Random => Box::new(RandomAi::new(PlayerMark::Cross, rng.gen())),
                PlayerType::Minimax => Box::new(MinMaxAi::<TicTacToe>::new(
                    PlayerMark::Cross,
                    ttt_heuristic,
                    args.mm_depth,
                )),
                PlayerType::AlphaBeta => {
                    Box::new(ABAi::<TicTacToe>::new(PlayerMark::Cross, ttt_heuristic, args.ab_depth))
                }
                PlayerType::Mcts => Box::new(MctsAi::<TicTacToe>::new(rng.gen(), c)),
            };
            run_game::<TicTacToe>(p1,p2)
        }
        GameType::Uttt => {
            let p1: Box<dyn Player<ultimate_ttt::Board,ultimate_ttt::Action>> = match args.p1 {
                PlayerType::Console => Box::new(ConsolePlayer::new(PlayerMark::Naught)),
                PlayerType::Random => Box::new(RandomAi::new(PlayerMark::Naught, rng.gen())),
                PlayerType::Minimax => Box::new(MinMaxAi::<UltimateTicTacToe>::new(
                    PlayerMark::Naught,
                    uttt_heuristic,
                    args.mm_depth,
                )),
                PlayerType::AlphaBeta => {
                    Box::new(ABAi::<UltimateTicTacToe>::new(PlayerMark::Naught, uttt_heuristic, args.ab_depth))
                }
                PlayerType::Mcts => Box::new(MctsAi::<UltimateTicTacToe>::new(rng.gen(), c)),
            };
            let p2: Box<dyn Player<ultimate_ttt::Board,ultimate_ttt::Action>> = match args.p2 {
                PlayerType::Console => Box::new(ConsolePlayer::new(PlayerMark::Cross)),
                PlayerType::Random => Box::new(RandomAi::new(PlayerMark::Cross, rng.gen())),
                PlayerType::Minimax => Box::new(MinMaxAi::<UltimateTicTacToe>::new(
                    PlayerMark::Cross,
                    uttt_heuristic,
                    args.mm_depth,
                )),
                PlayerType::AlphaBeta => {
                    Box::new(ABAi::<UltimateTicTacToe>::new(PlayerMark::Cross, uttt_heuristic, args.ab_depth))
                }
                PlayerType::Mcts => Box::new(MctsAi::<UltimateTicTacToe>::new(rng.gen(), c)),
            };
            run_game::<UltimateTicTacToe>(p1,p2)
        }
        GameType::C4 => {
            let p1: Box<dyn Player<C4Board,usize>> = match args.p1 {
                PlayerType::Console => Box::new(ConsolePlayer::new(PlayerMark::Naught)),
                PlayerType::Random => Box::new(RandomAi::new(PlayerMark::Naught, rng.gen())),
                PlayerType::Minimax => Box::new(MinMaxAi::<ConnectFour>::new(
                    PlayerMark::Naught,
                    c4_heuristic,
                    args.mm_depth,
                )),
                PlayerType::AlphaBeta => {
                    Box::new(ABAi::<ConnectFour>::new(PlayerMark::Naught, c4_heuristic, args.ab_depth))
                }
                PlayerType::Mcts => Box::new(MctsAi::<ConnectFour>::new(rng.gen(), c)),
            };
            let p2: Box<dyn Player<C4Board,usize>> = match args.p2 {
                PlayerType::Console => Box::new(ConsolePlayer::new(PlayerMark::Cross)),
                PlayerType::Random => Box::new(RandomAi::new(PlayerMark::Cross, rng.gen())),
                PlayerType::Minimax => Box::new(MinMaxAi::<ConnectFour>::new(
                    PlayerMark::Cross,
                    c4_heuristic,
                    args.mm_depth,
                )),
                PlayerType::AlphaBeta => {
                    Box::new(ABAi::<ConnectFour>::new(PlayerMark::Cross, c4_heuristic, args.ab_depth))
                }
                PlayerType::Mcts => Box::new(MctsAi::<ConnectFour>::new(rng.gen(), c)),
            };
            run_game::<ConnectFour>(p1,p2)
        }
    };
}
