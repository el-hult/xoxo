use std::fmt::Display;

use crate::core::{Board, GameStatus, PlayerMark};

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
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord, Default)]
pub struct C4Board {
    /// 7 columns, 6 rows. N.B. it is column major
    board: [[Option<PlayerMark>; 6]; 7],
}

impl From<C4Board> for [[Option<PlayerMark>; 6]; 7] {
    fn from(val: C4Board) -> Self {
        val.board
    }
}

impl Board for C4Board {
    type Coordinate = usize;
    fn current_player(&self) -> PlayerMark {
        match self.board.iter().flatten().filter(|x| x.is_some()).count() % 2 {
            0 => PlayerMark::Naught,
            _ => PlayerMark::Cross,
        }
    }
    fn valid_moves(&self) -> Vec<usize> {
        self.board
            .iter()
            .enumerate()
            .filter_map(|(i, col)| if col[5].is_none() { Some(i) } else { None })
            .collect()
    }
    fn game_status(&self) -> GameStatus {
        match self.winner() {
            Some(m) => GameStatus::Won(m),
            None if self.board.iter().flatten().all(|cell| cell.is_some()) => GameStatus::Draw,
            _ => GameStatus::Undecided,
        }
    }

    fn place_mark(&mut self, a: usize, marker: PlayerMark) {
        assert!(a < 7, "Column out of bounds");
        let column = &mut self.board[a];
        assert!(column[5].is_none(), "Placed marker in full column");
        let top_of_column = column.iter_mut().find(|x| x.is_none()).unwrap();
        *top_of_column = Some(marker);
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
        C4Board { board: game_board }
    }};
}
impl C4Board {
    pub fn winner(&self) -> Option<PlayerMark> {
        for i in 0..7 {
            if let Some(winner) = self.winner_in_column(i) {
                return Some(winner);
            }
        }
        for i in 0..6 {
            if let Some(winner) = self.winner_in_row(i) {
                return Some(winner);
            }
        }
        for i in 3..=8 {
            if let Some(winner) = self.winner_in_slash_diagonal(i) {
                return Some(winner);
            }
        }
        for i in 3..=8 {
            if let Some(winner) = self.winner_in_backslash_diagonal(i) {
                return Some(winner);
            }
        }
        None
    }
    fn winner_in_column(&self, col: usize) -> Option<PlayerMark> {
        let col = &self.board[col];
        for i in 0..3 {
            if let Some(mark) = col[i] {
                if col[i + 1..i + 4].iter().all(|x| *x == Some(mark)) {
                    return Some(mark);
                }
            }
        }
        None
    }
    fn winner_in_row(&self, row: usize) -> Option<PlayerMark> {
        for i in 0..4 {
            if let Some(mark) = self.board[i][row] {
                if (1..4).all(|j| self.board[i + j][row] == Some(mark)) {
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
    fn winner_in_slash_diagonal(&self, diag: usize) -> Option<PlayerMark> {
        // if this diagonal has 3 or less elements, it can't have a winner
        assert!(
            (3..=8).contains(&diag),
            "Only the diagonals 3..=5 have 4 or more cells. Got diag = {}",
            diag
        );
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
            let candidate = self.board[x][y];
            candidate?;
            if (1..4).all(|j| self.board[x + j][y + j] == candidate) {
                return candidate;
            }
        }
        None
    }
    /// Diagonal goes from top left to bottom right
    /// Diagonal 0 has 1 element, and that is only [0][0]
    /// Diagonal 1 has 2 elements, and that is [0][1] and [1][0]
    /// etc
    fn winner_in_backslash_diagonal(&self, diag: usize) -> Option<PlayerMark> {
        // if this diagonal has 3 or less elements, it can't have a winner
        assert!(
            (3..=8).contains(&diag),
            "Only the diagonals 3..=5 have 4 or more cells"
        );
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
            let candidate = self.board[x][y];
            if candidate.is_none() {
                continue;
            }
            if (1..4).all(|j| self.board[x + j][y - j] == candidate) {
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
                write!(f, "{}", cell)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::core::GameStatus;

    use super::*;
    #[test]
    fn test_winner_in_column() {
        let mut board = C4Board::default();
        board.place_mark(0, PlayerMark::Cross);
        board.place_mark(0, PlayerMark::Cross);
        board.place_mark(0, PlayerMark::Cross);
        assert_eq!(board.winner_in_column(0), None);
        board.place_mark(0, PlayerMark::Cross);
        assert_eq!(board.winner_in_column(0), Some(PlayerMark::Cross));
    }
    #[test]
    fn test_winner_in_row() {
        let mut board = C4Board::default();
        board.place_mark(0, PlayerMark::Cross);
        board.place_mark(1, PlayerMark::Cross);
        board.place_mark(2, PlayerMark::Cross);
        board.place_mark(3, PlayerMark::Cross);
        assert_eq!(board.winner_in_row(0), Some(PlayerMark::Cross));
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
        x......
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
        .x.....
        x......
        "
        );
        assert_eq!(board.winner_in_slash_diagonal(3), None);
        assert_eq!(board.winner_in_slash_diagonal(4), None);
        assert_eq!(board.winner_in_slash_diagonal(5), Some(PlayerMark::Cross));
        assert_eq!(board.winner_in_slash_diagonal(6), None);
        assert_eq!(board.winner_in_slash_diagonal(7), None);
        assert_eq!(board.winner_in_slash_diagonal(8), None);
    }
    #[test]
    fn test_realizes_game_over() {
        let board = parse_c4board!(
            "
        .......
        .......
        .o.....
        .oo....
        .xxo...
        xoxxo.."
        );
        assert_eq!(
            board.winner_in_backslash_diagonal(4),
            Some(PlayerMark::Naught)
        );
        assert_eq!(board.winner(), Some(PlayerMark::Naught));
        assert!(matches!(board.game_status(), GameStatus::Won(_)));
    }
}
