use crate::core::Board as BoardTrait;
use super::Game;
use super::{Player, PlayerMark};
const BOARD_SIZE: usize = 3; // If I want to change board size in the future, this variable may be useful. But as of now, there are several hard coded rules fixed to the 3x3 setting
const N_SQUARES: usize = BOARD_SIZE * BOARD_SIZE;

/// To play at a certain coordinate, you wrap a number representing the coordinate in `Action::MoveAt(number)`
/// The coordinate numbers for e 3x3 game are
///
///  1 2 3
///  4 5 6
///  7 8 9
///
/// Like the numbers on a phone. :)
///
///
/// invariant: the number inside must be 1-N_SQUARES
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Action(pub usize);
impl Action {
    pub fn move_at(idx: usize) -> Action {
        if (1..=N_SQUARES).contains(&idx) {
            Action(idx)
        } else {
            panic!("Tried to make an invalid move. Must be 1-N_SQUARES, but got {idx}")
        }
    }
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

/// The first member is the board entries from top left row wise to bottom right.
/// The second member is the victory counters. +1 for naughts. -1 for crosses.
/// Someone wins on a +3 or -3.
/// It holds 8 numbers: 3 rows (top to bottom), 3 columns (left to rifht) and two diagonals (first the one that points to southeast, and the the one to northeast)
#[derive(Clone, Copy, Debug)]
pub struct Board([Marker; N_SQUARES], [i32; 8]);

impl crate::core::Board<Action> for Board {
    fn valid_moves(&self) -> Vec<Action> {
        self.empty_addresses()
            .iter()
            .map(|&q| Action::move_at(q))
            .collect()
    }
    fn game_over(&self) -> bool {
        let board_full = self.0.iter().all(|&q| q.is_some());
        let won = self.winner().is_some();
        won || board_full
    }
    fn place_mark(&mut self, a: Action, marker: PlayerMark) {
        self.place(a.0, marker)
    }
}

impl Board {
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
    pub fn place(&mut self, addr: usize, p: PlayerMark) {
        if !(1..=N_SQUARES).contains(&addr) {
            panic!("Bad input!")
        }
        let num = addr - 1;
        if self.0[num].is_some() {
            panic!("There is already a marker there! Invalid move just played!")
        }
        let row = num / 3;
        let col = num % 3;
        let delta = match p {
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
        self.0[num] = Some(p);
    }

    #[cfg(test)]
    pub fn from_str(s: &str) -> Self {
        let mut b: Self = Self::new();
        assert!(s.len() == N_SQUARES);
        s.chars().enumerate().for_each(|(num, c)| match c {
            'x' => b.place(num + 1, PlayerMark::Cross),
            'o' => b.place(num + 1, PlayerMark::Naught),
            ' ' => {}
            _ => panic!("Invalid string slice! MAy only contain x o or blank space"),
        });
        b
    }

    fn new() -> Self {
        Self([None; N_SQUARES], [0; 8])
    }

    /// with the 1-N_SQUARES convention
    fn empty_addresses(&self) -> Vec<usize> {
        self.0
            .iter()
            .enumerate()
            .filter_map(|(num, &mark)| if mark.is_none() { Some(num + 1) } else { None })
            .collect()
    }

    pub fn n_moves_made(&self) -> f64 {
        self.0.iter().map(|&q| q.is_some() as u64 as f64).sum()
    }
}

impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let m = |m: Marker| match m {
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
        self.0[6..N_SQUARES]
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
    board: Board,
}

impl Game for TicTacToe {
    type Board = Board;
    type Action = Action;
    fn run(&mut self) {
        let mut is_naught = true;
        while self.is_running() {
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

type Marker = Option<PlayerMark>;

impl TicTacToe {
    pub fn new(
        naughts: Box<dyn Player<TicTacToe>>,
        crosses: Box<dyn Player<TicTacToe>>,
    ) -> Self {
        Self {
            player1: naughts,
            player2: crosses,
            board: Board::new(),
        }
    }

    fn update(&mut self, a: Action, is_naught: bool) {
        let player_mark = if is_naught {
            PlayerMark::Naught
        } else {
            PlayerMark::Cross
        };
        match a {
            Action(num) => {
                println!("Player {player_mark:?} placed marker at {num}");
                self.board.place(num, player_mark);
            }
        };
    }

    fn is_running(&self) -> bool {
        !self.board.game_over()
    }
}

