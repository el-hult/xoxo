use std::time::Duration;

use log::debug;

use crate::core::{BlitzPlayer, Board, GameEndStatus, GameStatus, PlayerMark};

pub mod connect_four;
pub mod tictactoe;
pub mod ultimate_ttt;

pub fn run_blitz_game<B: Board>(
    mut p1: Box<dyn BlitzPlayer<B>>,
    mut p2: Box<dyn BlitzPlayer<B>>,
    think_time: Duration,
) -> (GameEndStatus, Duration, Duration) {
    let mut current_player = PlayerMark::Naught;
    let mut board = B::default();
    let mut time_remaining_naughts = think_time;
    let mut time_remaining_crosses = think_time;
    while !board.game_is_over() {
        let t0 = std::time::Instant::now();
        let action = match current_player {
            PlayerMark::Naught => p1.blitz(&board, time_remaining_naughts),
            PlayerMark::Cross => p2.blitz(&board, time_remaining_crosses),
        };
        let t1 = std::time::Instant::now();
        match current_player {
            PlayerMark::Naught => {
                time_remaining_naughts = time_remaining_naughts
                    .checked_sub(t1.duration_since(t0))
                    .unwrap_or(Duration::ZERO);
                if time_remaining_naughts == Duration::ZERO {
                    debug!("{} ran out of time", PlayerMark::Naught);
                    return (
                        GameEndStatus::X,
                        time_remaining_naughts,
                        time_remaining_crosses,
                    );
                }
            }
            PlayerMark::Cross => {
                time_remaining_crosses = time_remaining_crosses
                    .checked_sub(t1.duration_since(t0))
                    .unwrap_or(Duration::ZERO);
                if time_remaining_crosses == Duration::ZERO {
                    debug!("{} ran out of time", PlayerMark::Cross);
                    return (
                        GameEndStatus::O,
                        time_remaining_naughts,
                        time_remaining_crosses,
                    );
                }
            }
        }
        debug!("Player {} played {}", current_player, &action);
        board.place_mark(action, current_player);
        debug!("\n{}", board);
        current_player = current_player.other();
    }
    debug!(
        "Time remaining: {:?} and {:?}",
        time_remaining_naughts, time_remaining_crosses
    );
    debug!("Game over");
    let winstatus = match board.game_status() {
        GameStatus::Draw => GameEndStatus::Draw,
        GameStatus::Won(PlayerMark::Cross) => GameEndStatus::X,
        GameStatus::Won(PlayerMark::Naught) => GameEndStatus::O,
        GameStatus::Undecided => unreachable!(),
    };
    debug!("Game ended with {}", winstatus);
    (winstatus, time_remaining_naughts, time_remaining_crosses)
}
