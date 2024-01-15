use alpha_beta::ABAi;
use std::{fmt::Display, io::BufRead};
mod tictactoe;

mod console_player {
    use super::*;

    pub struct ConsolePlayer {
        pub name: String,
    }

    impl ConsolePlayer {
        pub fn new(mark: PlayerMark) -> Self {
            ConsolePlayer {
                name: match mark {
                    PlayerMark::Cross => "X".into(),
                    PlayerMark::Naught => "O".into(),
                },
            }
        }
    }
    impl Player<TicTacToeGame> for ConsolePlayer {
        fn play(&mut self, b: &<TicTacToeGame as Game>::B) -> tictactoe::Action {
            println!("Time for {} to make a move", self.name);
            print!("{}", b);
            println!("Input a number 1-9 to make a move 1 = top left, 9 = bottom right");
            let mut line = String::new();
            std::io::stdin()
                .lock()
                .read_line(&mut line)
                .expect("Could not read line");
            let num = line
                .chars()
                .next()
                .expect("At least one character must be input");
            let num = num.to_string().parse::<_>().expect("Must input number");
            if !(1..=9).contains(&num) {
                eprintln!("Number not in range 1-N_SQUARES");
            }
            println!("Got {}", num);
            tictactoe::Action(num)
        }
    }
    impl Player<UltimateTicTacToe> for ConsolePlayer {
        fn play(&mut self, b: &<UltimateTicTacToe as Game>::B) -> ultimate_ttt::Action {
            println!("Time for {} to make a move", self.name);
            print!("{}", b);
            println!("Input four numbers 1-2 to make a move 1 = top left, 2 = bottom right");
            println!("board-row board-col pos-row pos-col");
            let mut line = String::new();
            std::io::stdin()
                .lock()
                .read_line(&mut line)
                .expect("Could not read line");
            let nums = line
                .split_ascii_whitespace()
                .map(|x| x.parse::<usize>().expect("Must input number") - 1)
                .collect::<Vec<_>>();
            if nums.len() != 4 {
                eprintln!("Must input 4 numbers, 1-3 with space in between. i j k l represents board on row i, column j, and in that board, play position row k col l");
            }
            ultimate_ttt::Action::try_from((nums[0], nums[1], nums[2], nums[3])).unwrap()
        }
    }
}

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

mod random_ai {
    use super::*;
    pub struct RandomAi<Rng> {
        rng: Rng,
        pub name: String,
    }

    impl<Rng, G> Player<G> for RandomAi<Rng>
    where
        Rng: rand::Rng,
        G: Game,
    {
        fn play(&mut self, b: &G::B) -> <G::B as Board>::A {
            let moves: Vec<_> = b.valid_moves();
            let idx = self.rng.next_u32() as usize % moves.len();
            println!("Random AI `{}` plays {}", self.name, moves[idx]);
            moves[idx]
        }
    }

    impl RandomAi<rand::prelude::ThreadRng> {
        pub fn new(mark: PlayerMark) -> Self {
            Self {
                name: match mark {
                    PlayerMark::Cross => "X".into(),
                    PlayerMark::Naught => "O".into(),
                },
                rng: rand::thread_rng(),
            }
        }
    }
}

/// Implement the alpha-beta pruning to get a good minimax implementation
mod alpha_beta {
    use super::*;

    pub struct ABAi<G>
    where
        G: Game,
    {
        my_marker: PlayerMark,
        /// A performance counter. If we prune well, this number is small
        n_leafs_evaluated: usize,
        heuristic_fn: HeuristicFn<G>,
        name: String,
        max_depth: usize,
    }

    impl<G: Game> ABAi<G> {
        pub fn new(mark: PlayerMark, heuristic_fn: HeuristicFn<G>, depth: usize) -> Self {
            ABAi {
                my_marker: mark,
                n_leafs_evaluated: 0,
                heuristic_fn,
                name: match mark {
                    PlayerMark::Cross => "alphabeta X".into(),
                    PlayerMark::Naught => "alphabeta O".into(),
                },
                max_depth: depth,
            }
        }

        fn heuristic(&mut self, b: &<G as Game>::B) -> f64 {
            self.n_leafs_evaluated += 1;
            (self.heuristic_fn)(self.my_marker, b)
        }

        /// compute the score of a node by use of alpha-beta with pruning
        /// Assumes I want to maximize my score, and the opponent makes moves to minimize it
        fn alphabeta(
            &mut self,
            node: &<G as Game>::B,
            depth: usize,
            a: f64,
            b: f64,
            my_move: bool,
        ) -> f64 {
            if depth == 0 || node.game_over() {
                let s = self.heuristic(node);
                // println!("Leaf node board\n {node} gets score {s}, at {depth}. Compare with {a} and {b}");
                return s;
            }
            let moves = node.valid_moves();
            let mut a = a;
            let mut b = b;
            let my_marker = self.my_marker; // take a copy here
            if my_move {
                // In this branch, the AI tries to find a move for itself that would maximize the score
                let mut value = -f64::INFINITY;
                let child_nodes = moves.iter().map(|addr| {
                    let mut child = *node;
                    child.place_mark(*addr, my_marker);
                    child
                });
                for child in child_nodes {
                    let newval = self.alphabeta(&child, depth - 1, a, b, false);
                    value = value.max(newval);
                    a = a.max(value);
                    if value >= b {
                        break;
                    }
                }
                value
            } else {
                // In this branch, the AI tries to find a move for the other player that would minimize the score
                let mut value = f64::INFINITY;
                let child_nodes = moves.iter().map(|addr| {
                    let mut child = *node;
                    child.place_mark(*addr, my_marker.other());
                    child
                });
                for child in child_nodes {
                    value = value.min(self.alphabeta(&child, depth - 1, a, b, true));
                    b = b.min(value);
                    if value <= a {
                        break;
                    }
                }
                value
            }
        }
    }

    impl<G> Player<G> for ABAi<G>
    where
        G: Game,
    {
        fn play(&mut self, b: &<G as Game>::B) -> <G::B as Board>::A {
            let res = b
                .valid_moves()
                .iter()
                .map(|addr| {
                    let mut b2 = *b;
                    b2.place_mark(*addr, self.my_marker);
                    let score =
                        self.alphabeta(&b2, self.max_depth, -f64::INFINITY, f64::INFINITY, false);
                    (score, addr)
                })
                // .inspect(|x| println!("about to pick the best: {x:?}"))
                .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(_, &q)| q)
                .expect("At least one element");
            println!(
                "{} heuristic evaluations computed by {}",
                self.n_leafs_evaluated, self.name
            );
            res
        }
    }

    #[cfg(test)]
    mod test {
        use crate::{
            tictactoe::Action,
            tictactoe::{Board, TicTacToeGame},
            ttt_heuristic, Player,
        };

        use super::ABAi;

        #[test]
        fn can_find_winning_move() {
            let b = Board::from_str("   xx    ");
            let mut ai = ABAi::<TicTacToeGame>::new(crate::PlayerMark::Cross, ttt_heuristic, 10);
            let action = ai.play(&b);
            assert_eq!(action, Action(6))
        }
        #[test]
        fn can_block_winning_move() {
            let b = Board::from_str("oo  x    ");
            let mut ai = ABAi::<TicTacToeGame>::new(crate::PlayerMark::Cross, ttt_heuristic, 10);
            let action = ai.play(&b);
            assert_eq!(action, Action(3))
        }
    }
}

type HeuristicFn<G> = fn(PlayerMark, &<G as Game>::B) -> f64;

/// Implement a direct minimax Ai
mod min_max {
    use super::*;

    pub struct MinMaxAi<G>
    where
        G: Game,
    {
        my_marker: PlayerMark,
        /// A performance counter. If we prune well, this number is small
        n_leafs_evaluated: usize,
        heuristic_fn: HeuristicFn<G>,
        max_depth: usize,
        name: String,
    }

    impl<G> MinMaxAi<G>
    where
        G: Game,
    {
        pub fn new(mark: PlayerMark, heuristic_fn: HeuristicFn<G>, depth: usize) -> Self {
            Self {
                my_marker: mark,
                n_leafs_evaluated: 0,
                heuristic_fn,
                max_depth: depth,
                name: match mark {
                    PlayerMark::Cross => "minimax X".into(),
                    PlayerMark::Naught => "minimax O".into(),
                },
            }
        }

        /// It is good to win. It is bad to lose.
        /// If we can win, we want to win fast,
        /// If we must lose or tie, we want to lose slowly
        /// It is always good to hold the mid point
        fn heuristic(&mut self, b: &<G as Game>::B) -> f64 {
            self.n_leafs_evaluated += 1;
            (self.heuristic_fn)(self.my_marker, b)
        }

        /// compute the score of a node by use of minimax
        /// Assumes I want to maximize my score, and the opponent makes moves to minimize it
        fn minimax(&mut self, node: &<G as Game>::B, depth: usize, my_move: bool) -> f64 {
            if depth == 0 || node.game_over() {
                let s = self.heuristic(node);
                // println!("Leaf node board\n {node} gets score {s}, at {depth}. Compare with {a} and {b}");
                return s;
            }
            let moves = node.valid_moves();
            let my_marker = self.my_marker; // take a copy here
            if my_move {
                // In this branch, the AI tries to find a move for itself that would maximize the score
                let mut value = -f64::INFINITY;
                let child_nodes = moves.iter().map(|addr| {
                    let mut child = *node;
                    child.place_mark(*addr, my_marker);
                    child
                });
                for child in child_nodes {
                    let newval = self.minimax(&child, depth - 1, false);
                    value = value.max(newval);
                }
                value
            } else {
                // In this branch, the AI tries to find a move for the other player that would minimize the score
                let mut value = f64::INFINITY;
                let child_nodes = moves.iter().map(|addr| {
                    let mut child = *node;
                    child.place_mark(*addr, my_marker.other());
                    child
                });
                for child in child_nodes {
                    value = value.min(self.minimax(&child, depth - 1, true));
                }
                value
            }
        }
    }

    impl<G> Player<G> for MinMaxAi<G>
    where
        G: Game,
    {
        fn play(&mut self, b: &<G as Game>::B) -> <G::B as Board>::A {
            let res = b
                .valid_moves()
                .iter()
                .map(|addr| {
                    let mut b2 = *b;
                    b2.place_mark(*addr, self.my_marker);
                    let score = self.minimax(&b2, self.max_depth, false);
                    (score, addr)
                })
                .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(_, &q)| q)
                .expect("At least one element");
            println!(
                "{} heuristic evaluations computed by {}",
                self.n_leafs_evaluated, self.name
            );
            res
        }
    }

    #[cfg(test)]
    mod test {
        use crate::{
            tictactoe::Action,
            tictactoe::{Board, TicTacToeGame},
            ttt_heuristic, Player,
        };

        use super::ABAi;

        #[test]
        fn can_find_winning_move() {
            let b = Board::from_str("   xx    ");
            let mut ai = ABAi::<TicTacToeGame>::new(crate::PlayerMark::Cross, ttt_heuristic,10);
            let action: Action = ai.play(&b);
            assert_eq!(action, Action(6))
        }
        #[test]
        fn can_block_winning_move() {
            let b = Board::from_str("oo  x    ");
            let mut ai = ABAi::<TicTacToeGame>::new(crate::PlayerMark::Cross, ttt_heuristic,10);
            let action = ai.play(&b);
            assert_eq!(action, Action(3))
        }
    }
}

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
