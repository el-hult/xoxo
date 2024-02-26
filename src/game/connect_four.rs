use std::fmt::Display;

use crate::core::{Board, GameStatus, PlayerMark};

type RawBoard = [[Option<PlayerMark>; 6]; 7];

/// A board is a 7x6 grid, where you can place a marker in one of the 7 columns
/// it lands on the top in that column we number the columns left to right and bottom to top
/// Since the board is a nested array the first index is the column and the second index is the row
///
/// [0][5]   [1][5]   [2][5]   [3][5]   [4][5]   [5][5]   [6][5]
/// [0][4]   [1][4]   [2][4]   [3][4]   [4][4]   [5][4]   [6][4]
/// [0][3]   [1][3]   [2][3]   [3][3]   [4][3]   [5][3]   [6][3]
/// [0][2]   [1][2]   [2][2]   [3][2]   [4][2]   [5][2]   [6][2]
/// [0][1]   [1][1]   [2][1]   [3][1]   [4][1]   [5][1]   [6][1]
/// [0][0]   [1][0]   [2][0]   [3][0]   [4][0]   [5][0]   [6][0]
///
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord, Default, serde::Serialize, serde::Deserialize)]
pub struct C4Board {
    /// 7 columns, 6 rows. N.B. it is column major
    board: RawBoard,
    /// The game status must always be valid. I.e. you must always keep it up to date in all &mut self methods
    status: GameStatus,
    /// Current player must always be valid. I.e. you must always keep it up to date in all &mut self methods
    current_player: PlayerMark,
}


impl From<C4Board> for RawBoard {
    fn from(val: C4Board) -> Self {
        val.board
    }
}

impl Board for C4Board {
    type Coordinate = usize;
    fn current_player(&self) -> PlayerMark {
        self.current_player
    }
    fn valid_moves(&self) -> Vec<usize> {
        self.board
            .iter()
            .enumerate()
            .filter_map(|(i, col)| if col[5].is_none() { Some(i) } else { None })
            .collect()
    }
    fn game_status(&self) -> GameStatus {
        self.status
    }

    fn place_mark(&mut self, column: usize, marker: PlayerMark) {
        assert!(column < 7, "Column out of bounds");
        let row = self.board[column].iter().position(|x| x.is_none()).expect("Column is full");
        self.board[column][row] = Some(marker);
        let i_won =Some(self.current_player);
        if (Self::raw_winner_in_column(&self.board,column) == i_won) || (Self::raw_winner_in_row(&self.board,row) == i_won)|| (Self::raw_winner_in_slash_diagonal(&self.board,5+column-row)== i_won) || (Self::raw_winner_in_backslash_diagonal(&self.board,column+row)== i_won) {
            self.status = GameStatus::Won(marker);
        } else if row == 5 && self.board.iter().all(|col| col.iter().all(|x| x.is_some())) {
            self.status = GameStatus::Draw;
        }
        self.current_player = self.current_player.other();
    }
}

#[cfg(test)]
#[doc = "A helpful parser. It takes a string and returns a C4Board. The string should have 6 lines, each with 7 characters. The characters can be 'x' or 'o' or '.'  you can add extra empty lines in start/end and add indentation as you like."]
macro_rules! parse_c4board {
    ($input:expr) => {{
        let rows: Vec<&str> = $input.trim().split('\n').collect();
        if rows.len() != 6 {
            panic!("Invalid number of rows for tic-tac-toe game");
        }

        let mut game_board = [[None; 6]; 7];

        for (i_row, row) in rows.iter().enumerate() {
            if row.trim().len() != 7 {
                panic!("Invalid number of columns for tic-tac-toe game");
            }
            for (j_col, cell) in row.trim().chars().enumerate() {
                game_board[j_col][5 - i_row] = match cell {
                    'x' => Some(PlayerMark::Cross),
                    'o' => Some(PlayerMark::Naught),
                    '.' => None,
                    _ => panic!(
                        "Invalid character in ({},{}). must be 'x' or 'o' or '.'",
                        i_row, j_col
                    ),
                };
            }
        }
        
        let current_player = C4Board::raw_current_player(game_board);
        let status = C4Board::raw_game_status(game_board);
        
        C4Board { board: game_board, current_player, status}

    }};
}
impl C4Board {

    pub fn winner(&self) -> Option<PlayerMark> {
        match self.status {
            GameStatus::Won(m) => Some(m),
            _ => None,
        }
    }

    /// Compute who is next to go, based on the current board
    /// Useful in debugging
    #[cfg(test)]
    fn raw_current_player(game_board: RawBoard) -> PlayerMark {
        let n_crosses = game_board
            .iter()
            .flatten()
            .filter(|x| **x == Some(PlayerMark::Cross))
            .count();
        let n_naughts = game_board.iter().flatten().filter(|x| **x == Some(PlayerMark::Naught)).count();
        
        if n_crosses == n_naughts {
                PlayerMark::Naught 
            } else if n_naughts == n_crosses+1{ 
                PlayerMark::Cross 
            } else {
                panic!("The number of x vs o is not valid for a game of connect four")
            }
    }
    #[cfg(test)]
    fn raw_game_status(board : RawBoard) -> GameStatus {
        match Self::raw_winner(&board) {
            Some(m) => GameStatus::Won(m),
            None if board.iter().flatten().all(|cell| cell.is_some()) => GameStatus::Draw,
            _ => GameStatus::Undecided,
        }
    }

    /// Compute if there is a winner from the board data alone
    /// Useful in debugging
    pub fn raw_winner(board: &RawBoard) -> Option<PlayerMark> {
        for i in 0..7 {
            if let Some(winner) = Self::raw_winner_in_column(board,i) {
                return Some(winner);
            }
        }
        for i in 0..6 {
            if let Some(winner) = Self::raw_winner_in_row(board,i) {
                return Some(winner);
            }
        }
        for i in 3..=8 {
            if let Some(winner) = Self::raw_winner_in_slash_diagonal(board,i) {
                return Some(winner);
            }
        }
        for i in 3..=8 {
            if let Some(winner) = Self::raw_winner_in_backslash_diagonal(board,i) {
                return Some(winner);
            }
        }
        None
    }
    fn raw_winner_in_column(board: &RawBoard, col: usize) -> Option<PlayerMark> {
        let col = &board[col];
        for i in 0..3 {
            if let Some(mark) = col[i] {
                if col[i + 1..i + 4].iter().all(|x| *x == Some(mark)) {
                    return Some(mark);
                }
            }
        }
        None
    }
    fn raw_winner_in_row(board: &RawBoard, row: usize) -> Option<PlayerMark> {
        for i in 0..4 {
            if let Some(mark) = board[i][row] {
                if (1..4).all(|j| board[i + j][row] == Some(mark)) {
                    return Some(mark);
                }
            }
        }
        None
    }

    /// Check if there is a winner in the diagonal that goes from bottom left to top right
    /// diagonal 0 has 1 element, and that is only [0][5]
    /// diagonal 1 has 2 elements, and that is [0][4] and [1][5]
    /// etc
    fn raw_winner_in_slash_diagonal(board: &RawBoard, diag: usize) -> Option<PlayerMark> {
        // if this diagonal has 3 or less elements, it can't have a winner
        if !(3..=8).contains(&diag) { return None; }
        let n_chances = match diag {
            3 | 8 => 1,
            4 | 7 => 2,
            5 | 6 => 3,
            _ => unreachable!(),
        };
        for i in 0..n_chances {
            let (x, y) = match diag {
                3 => (i, 2 + i),
                4 => (i, 1 + i),
                5 => (i, i),
                6 => (1 + i, i),
                7 => (2 + i, i),
                8 => (3 + i, i),
                _ => unreachable!(),
            };
            let candidate = board[x][y];
            candidate?;
            if (1..4).all(|j| board[x + j][y + j] == candidate) {
                return candidate;
            }
        }
        None
    }
    /// Diagonal goes from top left to bottom right
    /// Diagonal 0 has 1 element, and that is only [0][0]
    /// Diagonal 1 has 2 elements, and that is [0][1] and [1][0]
    /// etc
    fn raw_winner_in_backslash_diagonal(board: &RawBoard, diag: usize) -> Option<PlayerMark> {
        // if this diagonal has 3 or less elements, it can't have a winner
        if !(3..=8).contains(&diag) {
            return None;
        }
        let n_chances = match diag {
            3 | 8 => 1,
            4 | 7 => 2,
            5 | 6 => 3,
            _ => unreachable!(),
        };
        for i in 0..n_chances {
            let (x, y) = match diag {
                3 => (i, 3 - i),
                4 => (i, 4 - i),
                5 => (i, 5 - i),
                6 => (1 + i, 5 - i),
                7 => (2 + i, 5 - i),
                8 => (3 + i, 5 - i),
                _ => unreachable!(),
            };
            let candidate = board[x][y];
            if candidate.is_none() {
                continue;
            }
            if (1..4).all(|j| board[x + j][y - j] == candidate) {
                return candidate;
            }
        }
        None
    }
}

impl Display for C4Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in (0..6).rev() {
            for col in 0..7 {
                let cell = match self.board[col][row] {
                    Some(PlayerMark::Cross) => 'x',
                    Some(PlayerMark::Naught) => 'o',
                    None => '.',
                };
                write!(f, "{} ", cell)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::core::{GameStatus, Board};


    use super::*;
    #[test]
    fn test_winner_in_column() {
        let mut board = C4Board::default();
        board.place_mark(0, PlayerMark::Cross);
        board.place_mark(0, PlayerMark::Cross);
        board.place_mark(0, PlayerMark::Cross);
        assert_eq!(board.winner(), None);
        board.place_mark(0, PlayerMark::Cross);
        assert_eq!(board.winner(), Some(PlayerMark::Cross));
    }
    #[test]
    fn test_winner_in_row() {
        let mut board = C4Board::default();
        board.place_mark(0, PlayerMark::Cross);
        board.place_mark(1, PlayerMark::Cross);
        board.place_mark(2, PlayerMark::Cross);
        board.place_mark(3, PlayerMark::Cross);
        assert_eq!(board.winner(), Some(PlayerMark::Cross));
        assert_eq!(C4Board::raw_winner_in_row(&board.board,0), Some(PlayerMark::Cross));
    }
    #[test]
    fn parse_board() {
        let board = parse_c4board!(
            "
        .......
        .......
        ...x...
        ..x....
        .x.....
        xooo.o.
        "
        );
        assert_eq!(board.board[0][1], None);
        assert_eq!(board.board[0][0], Some(PlayerMark::Cross));
        assert_eq!(board.board[1][1], Some(PlayerMark::Cross));
        assert_eq!(board.board[2][2], Some(PlayerMark::Cross));
        assert_eq!(board.board[3][3], Some(PlayerMark::Cross));
        assert_eq!(board.board[4][4], None);
    }
    #[test]
    fn test_winner_in_slash_diagonal() {
        let board = parse_c4board!(
            "
        .......
        .......
        ...x...
        ..x....
        oxo....
        xoo....
        "
        );
        assert_eq!(C4Board::raw_winner_in_slash_diagonal(&board.board,0), None);
        assert_eq!(C4Board::raw_winner_in_slash_diagonal(&board.board,1), None);
        assert_eq!(C4Board::raw_winner_in_slash_diagonal(&board.board,2), None);
        assert_eq!(C4Board::raw_winner_in_slash_diagonal(&board.board,3), None);
        assert_eq!(C4Board::raw_winner_in_slash_diagonal(&board.board,4), None);
        assert_eq!(C4Board::raw_winner_in_slash_diagonal(&board.board,5), Some(PlayerMark::Cross));
        assert_eq!(C4Board::raw_winner_in_slash_diagonal(&board.board,6), None);
        assert_eq!(C4Board::raw_winner_in_slash_diagonal(&board.board,7), None);
        assert_eq!(C4Board::raw_winner_in_slash_diagonal(&board.board,8), None);
        assert_eq!(C4Board::raw_winner_in_slash_diagonal(&board.board,9), None);
        assert_eq!(C4Board::raw_winner_in_slash_diagonal(&board.board,10), None);
        assert_eq!(C4Board::raw_winner_in_slash_diagonal(&board.board,11), None);
    }
    #[test]
    fn test_realizes_game_over() {
        let board: C4Board = parse_c4board!(
            "
        .......
        .......
        .o.....
        .oo....
        .xxo...
        xoxxo.."
        );
        assert_eq!(
            C4Board::raw_winner_in_backslash_diagonal(&board.board,4),
            Some(PlayerMark::Naught)
        );
        assert_eq!(board.winner(), Some(PlayerMark::Naught));
        assert!(matches!(board.game_status(), GameStatus::Won(_)));
    }
}
