use alpha_beta::ABAi;
use std::{fmt::Display, io::BufRead};
mod tictactoe;

mod console_player;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PlayerMark {
    Cross,
    Naught,
}

impl PlayerMark {
    fn other(&self) -> Self {
        match *self {
            Self::Cross => Self::Naught,
            Self::Naught => Self::Cross,
        }
    }
}

/// The Player trait is the struct that represents a player.
pub trait Player<G>
where
    G: Game,
{
    /// The play function is the main mechanic for the AIs
    /// You observe the whole board through a reference, and can do whatever you like, and then you return an action representing where to play
    fn play(&mut self, b: &G::B) -> <G::B as Board>::A;
}
mod random_ai;

mod alpha_beta;

type HeuristicFn<G> = fn(PlayerMark, &<G as Game>::B) -> f64;

mod min_max;

use clap::{Parser, ValueEnum};
use console_player::ConsolePlayer;
use min_max::MinMaxAi;
use random_ai::RandomAi;
use tictactoe::TicTacToeGame;
mod ultimate_ttt;
use ultimate_ttt::UltimateTicTacToe;

pub trait Game {
    type B: Board + Copy;
    fn run(&mut self);
}

pub trait Board: Display {
    /// Actions that can be taken on this board
    /// They should be some simple kind of data, so they must implement Copy
    type A: Copy + Display;
    fn valid_moves(&self) -> Vec<Self::A>;
    fn place_mark(&mut self, a: Self::A, marker: PlayerMark);
    fn game_over(&self) -> bool;
}

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
}

fn ttt_heuristic(my_marker: PlayerMark, b: &<TicTacToeGame as Game>::B) -> f64 {
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
fn uttt_heuristic(my_marker: PlayerMark, b: &<UltimateTicTacToe as Game>::B) -> f64 {
    let n_moves_made: f64 = b.n_moves_made() as f64;
    let n_supboards_won = b
        .get_sup_board()
        .iter()
        .flatten()
        .filter(|&&x| x == ultimate_ttt::BoardStatus::Won(my_marker))
        .count() as f64;
    match b.get_winner() {
        None => 0.0 + n_moves_made + n_supboards_won,
        Some(None) => 0.0 + n_moves_made + n_supboards_won,
        Some(Some(mark)) => {
            if mark == my_marker {
                1000.0 - n_moves_made + n_supboards_won
            } else {
                -1000.0 + n_moves_made + n_supboards_won
            }
        }
    }
}

fn main() {
    let args = Args::parse();
    match args.game {
        GameType::Ttt => {
            let p1: Box<dyn Player<TicTacToeGame>> = match args.p1 {
                PlayerType::Console => Box::new(ConsolePlayer::new(PlayerMark::Naught)),
                PlayerType::Random => Box::new(RandomAi::new(PlayerMark::Naught)),
                PlayerType::Minimax => Box::new(MinMaxAi::new(
                    PlayerMark::Naught,
                    ttt_heuristic,
                    args.mm_depth,
                )),
                PlayerType::AlphaBeta => {
                    Box::new(ABAi::new(PlayerMark::Naught, ttt_heuristic, args.ab_depth))
                }
            };
            let p2: Box<dyn Player<TicTacToeGame>> = match args.p2 {
                PlayerType::Console => Box::new(ConsolePlayer::new(PlayerMark::Cross)),
                PlayerType::Random => Box::new(RandomAi::new(PlayerMark::Cross)),
                PlayerType::Minimax => Box::new(MinMaxAi::new(
                    PlayerMark::Cross,
                    ttt_heuristic,
                    args.mm_depth,
                )),
                PlayerType::AlphaBeta => {
                    Box::new(ABAi::new(PlayerMark::Cross, ttt_heuristic, args.ab_depth))
                }
            };
            let mut g = TicTacToeGame::new(p1, p2);
            g.run()
        }
        GameType::Uttt => {
            let p1: Box<dyn Player<UltimateTicTacToe>> = match args.p1 {
                PlayerType::Console => Box::new(ConsolePlayer::new(PlayerMark::Naught)),
                PlayerType::Random => Box::new(RandomAi::new(PlayerMark::Naught)),
                PlayerType::Minimax => Box::new(MinMaxAi::new(
                    PlayerMark::Naught,
                    uttt_heuristic,
                    args.mm_depth,
                )),
                PlayerType::AlphaBeta => {
                    Box::new(ABAi::new(PlayerMark::Naught, uttt_heuristic, args.ab_depth))
                }
            };
            let p2: Box<dyn Player<UltimateTicTacToe>> = match args.p2 {
                PlayerType::Console => Box::new(ConsolePlayer::new(PlayerMark::Cross)),
                PlayerType::Random => Box::new(RandomAi::new(PlayerMark::Cross)),
                PlayerType::Minimax => Box::new(MinMaxAi::new(
                    PlayerMark::Cross,
                    uttt_heuristic,
                    args.mm_depth,
                )),
                PlayerType::AlphaBeta => {
                    Box::new(ABAi::new(PlayerMark::Cross, uttt_heuristic, args.ab_depth))
                }
            };
            let mut g = UltimateTicTacToe::new(p1, p2);
            g.run()
        }
    };
}
