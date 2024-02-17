pub mod alpha_beta;
pub mod min_max;
pub mod random;
pub mod mcts;
pub mod console;
mod heuristics;

pub use min_max::MinMaxAi;
pub use random::RandomAi;
pub use heuristics::{ttt_heuristic, c4_heuristic, uttt_heuristic};