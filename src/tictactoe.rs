use super::Game;
use super::{Player, PlayerMark};
use crate::core::Board;

/// Represents a coordinate on the board
///
///  1 2 3
///  4 5 6
///  7 8 9
///
/// invariant: the number inside must be 1-9
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct TTTAddr(pub usize);

impl std::fmt::Display for TTTAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

/// The first member is the board entries from top left row wise to bottom right.
/// The second member is the victory counters. +1 for naughts. -1 for crosses.
/// Someone wins on a +3 or -3.
/// It holds 8 numbers: 3 rows (top to bottom), 3 columns (left to rifht) and two diagonals (first the one that points to southeast, and the the one to northeast)
#[derive(Clone, Copy, Debug)]
pub struct TTTBoard([Option<PlayerMark>; 9], [i32; 8]);

impl Board<TTTAddr> for TTTBoard {
    fn valid_moves(&self) -> Vec<TTTAddr> {
        self.0
            .iter()
            .enumerate()
            .filter_map(|(num, &mark)| {
                if mark.is_none() {
                    Some(TTTAddr(num + 1))
                } else {
                    None
                }
            })
            .collect()
    }
    fn game_over(&self) -> bool {
        let board_full = self.0.iter().all(|&q| q.is_some());
        let won = self.winner().is_some();
        won || board_full
    }
    fn place_mark(&mut self, a: TTTAddr, marker: PlayerMark) {
        let addr = a.0;
        if !(1..=9).contains(&addr) {
            panic!("Bad input!")
        }
        let num = addr - 1;
        if self.0[num].is_some() {
            panic!("There is already a marker there! Invalid move just played!")
        }
        let row = num / 3;
        let col = num % 3;
        let delta = match marker {
            PlayerMark::Naught => 1,
            PlayerMark::Cross => -1,
        };
        self.1[row] += delta;
        self.1[3 + col] += delta;
        if row == col {
            self.1[6] += delta;
        }
        if row == 2 - col {
            self.1[7] += delta;
        }
        self.0[num] = Some(marker);
    }
}

impl TTTBoard {

    /// Is there a winner?
    pub fn winner(&self) -> Option<PlayerMark> {
        let naught_won = self.1.iter().any(|&x| x == 3);
        let cross_won = self.1.iter().any(|&x| x == -3);
        if naught_won && !cross_won {
            Some(PlayerMark::Naught)
        } else if !naught_won && cross_won {
            Some(PlayerMark::Cross)
        } else if !naught_won && !cross_won {
            None
        } else {
            panic!("Logic error. Both win!?")
        }
    }

    #[cfg(test)]
    pub fn from_str(s: &str) -> Self {
        let mut b: Self = Self::new();
        assert!(s.len() == 9);
        s.chars().enumerate().for_each(|(num, c)| match c {
            'x' => b.place_mark(TTTAddr(num + 1), PlayerMark::Cross),
            'o' => b.place_mark(TTTAddr(num + 1), PlayerMark::Naught),
            ' ' => {}
            _ => panic!("Invalid string slice! MAy only contain x o or blank space"),
        });
        b
    }

    fn new() -> Self {
        Self([None; 9], [0; 8])
    }

    pub fn n_moves_made(&self) -> f64 {
        self.0.iter().map(|&q| q.is_some() as u64 as f64).sum()
    }
}

impl std::fmt::Display for TTTBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let m = |m | match m {
            None => ' ',
            Some(PlayerMark::Cross) => 'X',
            Some(PlayerMark::Naught) => 'O',
        };
        writeln!(f, " ------- ")?;
        write!(f, "| ")?;
        self.0[0..3]
            .iter()
            .try_for_each(|&mark| write!(f, "{} ", m(mark)))?;
        writeln!(f, "|")?;
        write!(f, "| ")?;
        self.0[3..6]
            .iter()
            .try_for_each(|&mark| write!(f, "{} ", m(mark)))?;
        writeln!(f, "|")?;
        write!(f, "| ")?;
        self.0[6..9]
            .iter()
            .try_for_each(|&mark| write!(f, "{} ", m(mark)))?;
        writeln!(f, "|")?;
        writeln!(f, " ------- ")
    }
}

/// The holder of the game state, and the state of the players
/// This struct is a bit nasty, because it is kind of a world-object
pub struct TicTacToe {
    player1: Box<dyn Player<TicTacToe>>,
    player2: Box<dyn Player<TicTacToe>>,
    board: TTTBoard,
}

impl Game for TicTacToe {
    type Board = TTTBoard;
    type Coordinate = TTTAddr;
    fn run(&mut self) {
        let mut is_naught = true;
        while !self.board.game_over() {
            let action = if is_naught {
                self.player1.play(&self.board)
            } else {
                self.player2.play(&self.board)
            };
            self.update(action, is_naught);
            is_naught = !is_naught;
        }
        println!("{}", &self.board);
        if let Some(p) = self.board.winner() {
            println!("Player {:?} won", p);
        }
        println!("Game over.");
    }
}

impl TicTacToe {
    pub fn new(naughts: Box<dyn Player<TicTacToe>>, crosses: Box<dyn Player<TicTacToe>>) -> Self {
        Self {
            player1: naughts,
            player2: crosses,
            board: TTTBoard::new(),
        }
    }

    fn update(&mut self, a: TTTAddr, is_naught: bool) {
        let player_mark = if is_naught {
            PlayerMark::Naught
        } else {
            PlayerMark::Cross
        };
        match a {
            TTTAddr(num) => {
                println!("Player {player_mark:?} placed marker at {num}");
                self.board.place_mark(TTTAddr(num), player_mark);
            }
        };
    }

}
