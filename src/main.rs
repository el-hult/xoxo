use std::io::BufRead;
const BOARD_SIZE: usize = 3; // If I want to change board size in the future, this variable may be useful. But as of now, there are several hard coded rules fixed to the 3x3 setting
const N_SQUARES: usize = BOARD_SIZE * BOARD_SIZE;
use alpha_beta::ABAi;

struct ConsolePlayer {
    name: String,
}

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

#[derive(Debug, PartialEq, Eq)]
enum Action {
    /// invariant: the number inside must be 1-N_SQUARES
    MoveAt(usize),
}
impl Action {
    fn move_at(idx: usize) -> Action {
        if (1..=N_SQUARES).contains(&idx) {
            Action::MoveAt(idx)
        } else {
            panic!("LOGIC ERROR!")
        }
    }
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

/// The first member is the board entries from top left row wise to bottom right.
/// The second member is the victory counters. +1 for naughts. -1 for crosses.
/// Someone wins on a +3 or -3.
/// It holds 8 numbers: 3 rows (top to bottom), 3 columns (left to rifht) and two diagonals (first the one that points to southeast, and the the one to northeast)
#[derive(Clone, Copy, Debug)]
struct Board([Marker; N_SQUARES], [i32; 8]);

impl Board {
    fn winner(&self) -> Option<PlayerMark> {
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
    fn place(&mut self, addr: usize, p: PlayerMark) {
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

    fn game_over(&self) -> bool {
        let board_full = self.0.iter().all(|&q| q.is_some());
        let won = self.winner().is_some();
        won || board_full
    }

    #[cfg(test)]
    fn from_str(s: &str) -> Self {
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

/// The Player trait is the struct that represents a player.
trait Player {
    /// The play function is the main mechanic.
    /// You observe the whole board through a reference, and can do whatever you like, and then you return an action representing where to play
    fn play(&mut self, b: &Board) -> Action;
}

impl Player for ConsolePlayer {
    fn play(&mut self, b: &Board) -> Action {
        println!("Time for {} to make a move", self.name);
        print!("{}", b);
        println!(
            "Input a number 1-{N_SQUARES} to make a move 1 = top left, {N_SQUARES} = bottom right"
        );
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
        if !(1..=N_SQUARES).contains(&num) {
            eprintln!("Number not in range 1-N_SQUARES");
        }
        println!("Got {}", num);
        Action::MoveAt(num)
    }
}

struct RandomAi<Rng> {
    rng: Rng,
}

impl<Rng> Player for RandomAi<Rng>
where
    Rng: rand::Rng,
{
    fn play(&mut self, b: &Board) -> Action {
        let free_spots: Vec<_> =
            b.0.iter()
                .enumerate()
                .filter_map(|(num, &mark)| if mark.is_none() { Some(num) } else { None })
                .collect();
        let idx = self.rng.next_u32() as usize % free_spots.len();
        Action::move_at(free_spots[idx] + 1)
    }
}

impl RandomAi<rand::prelude::ThreadRng> {
    pub fn new() -> Self {
        Self {
            rng: rand::thread_rng(),
        }
    }
}

/// Implement the alpha-beta pruning to get a good minimax implementation
mod alpha_beta {
    use super::*;

    pub struct ABAi {
        my_marker: PlayerMark,
        /// A performance counter. If we prune well, this number is small
        n_leafs_evaluated: usize,
    }

    impl ABAi {
        pub fn new(mark: PlayerMark) -> Self {
            ABAi {
                my_marker: mark,
                n_leafs_evaluated: 0,
            }
        }

        /// It is good to win. It is bad to lose.
        /// If we can win, we want to win fast,
        /// If we must lose or tie, we want to lose slowly
        /// It is always good to hold the mid point
        fn heuristic(&mut self, b: &Board) -> f64 {
            self.n_leafs_evaluated += 1;
            let n_moves_made: f64 = b.0.iter().map(|&q| q.is_some() as u64 as f64).sum();
            match b.winner() {
                None => 0.0 + n_moves_made,
                Some(mark) => {
                    if mark == self.my_marker {
                        100.0 - n_moves_made
                    } else {
                        -100.0 + n_moves_made
                    }
                }
            }
        }

        /// compute the score of a node by use of alpha-beta with pruning
        /// Assumes I want to maximize my score, and the opponent makes moves to minimize it
        fn alphabeta(&mut self, node: &Board, depth: usize, a: f64, b: f64, my_move: bool) -> f64 {
            if depth == 0 || node.game_over() {
                let s = self.heuristic(node);
                // println!("Leaf node board\n {node} gets score {s}, at {depth}. Compare with {a} and {b}");
                return s;
            }
            let moves = empty_addresses(node);
            let mut a = a;
            let mut b = b;
            let my_marker = self.my_marker; // take a copy here
            if my_move {
                // In this branch, the AI tries to find a move for itself that would maximize the score
                let mut value = -f64::INFINITY;
                let child_nodes = moves.iter().map(|addr| {
                    let mut child = *node;
                    child.place(*addr, my_marker);
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
                    child.place(*addr, my_marker.other());
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

    /// with the 1-N_SQUARES convention
    fn empty_addresses(b: &Board) -> Vec<usize> {
        b.0.iter()
            .enumerate()
            .filter_map(|(num, &mark)| if mark.is_none() { Some(num + 1) } else { None })
            .collect()
    }

    impl Player for ABAi {
        fn play(&mut self, b: &crate::Board) -> crate::Action {
            let res = empty_addresses(b)
                .iter()
                .map(|addr| {
                    let mut b2 = *b;
                    b2.place(*addr, self.my_marker);
                    let score = self.alphabeta(&b2, 10, -f64::INFINITY, f64::INFINITY, false);
                    (score, addr)
                })
                // .inspect(|x| println!("about to pick the best: {x:?}"))
                .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(_, &q)| Action::move_at(q))
                .expect("At least one element");
            println!("Up to {} heuristic evaluations", self.n_leafs_evaluated);
            res
        }
    }

    #[cfg(test)]
    mod test {
        use crate::{Action, Board, Player};

        use super::ABAi;

        #[test]
        fn can_find_winning_move() {
            let b = Board::from_str("   xx    ");
            let mut ai = ABAi::new(crate::PlayerMark::Cross);
            let action = ai.play(&b);
            assert!(action == Action::MoveAt(6))
        }
        #[test]
        fn can_block_winning_move() {
            let b = Board::from_str("oo  x    ");
            let mut ai = ABAi::new(crate::PlayerMark::Cross);
            let action = ai.play(&b);
            assert_eq!(action, Action::MoveAt(3))
        }
    }
}

/// Implement a direct minimax Ai
mod min_max {
    use super::*;

    pub struct MinMaxAi {
        my_marker: PlayerMark,
        /// A performance counter. If we prune well, this number is small
        n_leafs_evaluated: usize,
    }

    impl MinMaxAi {
        pub fn new(mark: PlayerMark) -> Self {
            Self {
                my_marker: mark,
                n_leafs_evaluated: 0,
            }
        }

        /// It is good to win. It is bad to lose.
        /// If we can win, we want to win fast,
        /// If we must lose or tie, we want to lose slowly
        /// It is always good to hold the mid point
        fn heuristic(&mut self, b: &Board) -> f64 {
            self.n_leafs_evaluated += 1;
            let n_moves_made: f64 = b.0.iter().map(|&q| q.is_some() as u64 as f64).sum();
            match b.winner() {
                None => 0.0 + n_moves_made,
                Some(mark) => {
                    if mark == self.my_marker {
                        100.0 - n_moves_made
                    } else {
                        -100.0 + n_moves_made
                    }
                }
            }
        }

        /// compute the score of a node by use of minimax
        /// Assumes I want to maximize my score, and the opponent makes moves to minimize it
        fn minimax(&mut self, node: &Board, depth: usize, my_move: bool) -> f64 {
            if depth == 0 || node.game_over() {
                let s = self.heuristic(node);
                // println!("Leaf node board\n {node} gets score {s}, at {depth}. Compare with {a} and {b}");
                return s;
            }
            let moves = empty_addresses(node);
            let my_marker = self.my_marker; // take a copy here
            if my_move {
                // In this branch, the AI tries to find a move for itself that would maximize the score
                let mut value = -f64::INFINITY;
                let child_nodes = moves.iter().map(|addr| {
                    let mut child = *node;
                    child.place(*addr, my_marker);
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
                    child.place(*addr, my_marker.other());
                    child
                });
                for child in child_nodes {
                    value = value.min(self.minimax(&child, depth - 1, true));
                }
                value
            }
        }
    }

    /// with the 1-N_SQUARES convention
    fn empty_addresses(b: &Board) -> Vec<usize> {
        b.0.iter()
            .enumerate()
            .filter_map(|(num, &mark)| if mark.is_none() { Some(num + 1) } else { None })
            .collect()
    }

    impl Player for MinMaxAi {
        fn play(&mut self, b: &crate::Board) -> crate::Action {
            let res = empty_addresses(b)
                .iter()
                .map(|addr| {
                    let mut b2 = *b;
                    b2.place(*addr, self.my_marker);
                    let score = self.minimax(&b2, 10, false);
                    (score, addr)
                })
                .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(_, &q)| Action::move_at(q))
                .expect("At least one element");
            println!("Up to {} heuristic evaluations", self.n_leafs_evaluated);
            res
        }
    }

    #[cfg(test)]
    mod test {
        use crate::{Action, Board, Player};

        use super::ABAi;

        #[test]
        fn can_find_winning_move() {
            let b = Board::from_str("   xx    ");
            let mut ai = ABAi::new(crate::PlayerMark::Cross);
            let action = ai.play(&b);
            assert!(action == Action::MoveAt(6))
        }
        #[test]
        fn can_block_winning_move() {
            let b = Board::from_str("oo  x    ");
            let mut ai = ABAi::new(crate::PlayerMark::Cross);
            let action = ai.play(&b);
            assert_eq!(action, Action::MoveAt(3))
        }
    }
}

/// The holder of the game state, and the state of the players
/// This struct is a bit nasty, because it is kind of a world-object
struct TicTacToeGame {
    player1: Box<dyn Player>,
    player2: Box<dyn Player>,
    board: Board,
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

type Marker = Option<PlayerMark>;

impl TicTacToeGame {
    pub fn new(naughts: Box<dyn Player>, crosses: Box<dyn Player>) -> Self {
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
            Action::MoveAt(num) => {
                println!("Player {player_mark:?} placed marker at {num}");
                self.board.place(num, player_mark);
            }
        };
    }

    fn is_running(&self) -> bool {
        !self.board.game_over()
    }

    pub fn run(&mut self) {
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

use clap::{Parser, ValueEnum};
use min_max::MinMaxAi;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum PlayerType {
    Console,
    Random,
    Minimax,
    AlphaBeta,
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
}

fn main() {
    let args = Args::parse();
    let p1: Box<dyn Player> = match args.p1 {
        PlayerType::Console => Box::new(ConsolePlayer::new(PlayerMark::Naught)),
        PlayerType::Random => Box::new(RandomAi::new()),
        PlayerType::Minimax => Box::new(MinMaxAi::new(PlayerMark::Naught)),
        PlayerType::AlphaBeta => Box::new(ABAi::new(PlayerMark::Naught)),
    };
    let p2: Box<dyn Player> = match args.p2 {
        PlayerType::Console => Box::new(ConsolePlayer::new(PlayerMark::Cross)),
        PlayerType::Random => Box::new(RandomAi::new()),
        PlayerType::Minimax => Box::new(MinMaxAi::new(PlayerMark::Cross)),
        PlayerType::AlphaBeta => Box::new(ABAi::new(PlayerMark::Cross)),
    };
    let mut g = TicTacToeGame::new(p1, p2);
    g.run();
}
