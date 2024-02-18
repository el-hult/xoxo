//! The core abstractions for this application
//!

use serde::{Deserialize, Serialize};
use std::fmt::Display;

use clap::ValueEnum;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PlayerMark {
    Cross,
    Naught,
}

impl PlayerMark {
    pub fn other(&self) -> Self {
        match *self {
            Self::Cross => Self::Naught,
            Self::Naught => Self::Cross,
        }
    }
}

/// The Player trait is the struct that represents a player.
pub trait Player<B: Board> {
    /// The play function is the main mechanic for the AIs
    /// You observe the whole board through a reference, and can do whatever you like, and then you return an action representing where to play
    fn play(&mut self, b: &B) -> B::Coordinate;
}

/// The BlitzPlayer trait is a trait for players that are able to blitz the game, i.e. play games with time limits.
/// The time_remaining parameter is the time left for the player to make all remaining moves.
/// Just like in chess, the player loses if they run out of time.
/// It is up to the player to decide how to budget their time over the course of the game.
pub trait BlitzPlayer<B: Board> {
    fn blitz(&mut self, b: &B, time_remaining: std::time::Duration) -> B::Coordinate;
}

pub type HeuristicFn<B> = fn(PlayerMark, &B) -> f64;

pub trait Board: Display + Default {
    type Coordinate: Display + Copy;
    /// The coordinates where you are allowed to place your marker in this turn.
    fn valid_moves(&self) -> Vec<Self::Coordinate>;
    fn place_mark(&mut self, a: Self::Coordinate, marker: PlayerMark);
    fn game_status(&self) -> GameStatus;
    fn current_player(&self) -> PlayerMark;
    fn game_is_over(&self) -> bool {
        !matches!(self.game_status(), GameStatus::Undecided)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq, Ord, PartialOrd)]
pub enum GameStatus {
    Undecided,
    Draw,
    Won(PlayerMark),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Serialize, Deserialize)]
pub enum GameType {
    /// Normal Tic-Tac-Toe
    Ttt,
    /// Ultimate Tic-Tac-Toe
    Uttt,
    /// Connect Four
    C4,
}

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub enum GameEndStatus {
    Draw,
    /// X won
    X,
    /// O won
    O,
}

pub fn run_game<B: Board>(mut p1: Box<dyn Player<B>>, mut p2: Box<dyn Player<B>>) -> GameEndStatus{
    let mut current_player = PlayerMark::Naught;
    let mut board = B::default();
    while !board.game_is_over() {
        let action = match current_player {
            PlayerMark::Naught => p1.play(&board),
            PlayerMark::Cross => p2.play(&board),
        };
        board.place_mark(action, current_player);
        current_player = current_player.other();
    }
    println!("{}", &board);
    if let GameStatus::Won(p) = board.game_status() {
        println!("Player {:?} won", p);
    }
    println!("Game over.");

    match board.game_status() {
        GameStatus::Draw => GameEndStatus::Draw,
        GameStatus::Won(PlayerMark::Cross) => GameEndStatus::X,
        GameStatus::Won(PlayerMark::Naught) => GameEndStatus::O,
        GameStatus::Undecided => unreachable!(),
    }
}
