//! The core abstractions for this application
//! 

use std::fmt::Display;



#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
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
    fn play(&mut self, b: &G::Board) -> G::Coordinate;
}


pub type HeuristicFn<G> = fn(PlayerMark, &<G as Game>::Board) -> f64;


/// All games in this project is about placing PlayerMarker on a board
/// The markers are either crosses or naughts, symbolizing the two players
pub trait Game {
    /// The coordinate type is the type of the coordinates on the board
    type Coordinate: Copy + Display;
    type Board: Board<Self::Coordinate> + Copy;
    fn run(&mut self);
}

pub trait Board<Coordinate>: Display {
    /// The coordinates where you are allowed to place your marker in this turn.
    fn valid_moves(&self) -> Vec<Coordinate>;
    fn place_mark(&mut self, a: Coordinate, marker: PlayerMark);
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