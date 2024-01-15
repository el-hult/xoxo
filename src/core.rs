//! The core abstractions for this application
//! 

use std::fmt::Display;



#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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
pub trait Player<G>
where
    G: Game,
{
    /// The play function is the main mechanic for the AIs
    /// You observe the whole board through a reference, and can do whatever you like, and then you return an action representing where to play
    fn play(&mut self, b: &G::B) -> <G::B as Board>::A;
}


pub type HeuristicFn<G> = fn(PlayerMark, &<G as Game>::B) -> f64;


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