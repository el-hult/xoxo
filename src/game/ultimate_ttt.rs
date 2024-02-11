use std::fmt::Display;

use crate::{
    core::{Board as BoardTrait, Game, GameStatus, Player, PlayerMark},
    player::mcts::Mdp,
};

pub struct UltimateTicTacToe {
    board: Board,
    turn: PlayerMark,
    player_naught: Box<dyn Player<UltimateTicTacToe>>,
    player_cross: Box<dyn Player<UltimateTicTacToe>>,
}

impl UltimateTicTacToe {
    pub fn new(
        naughts: Box<dyn Player<UltimateTicTacToe>>,
        crosses: Box<dyn Player<UltimateTicTacToe>>,
    ) -> Self {
        Self {
            board: Board::new(),
            turn: PlayerMark::Naught,
            player_naught: naughts,
            player_cross: crosses,
        }
    }

    /// Update the game state board with the given action
    /// Remember that the win condition must be updated at the end..
    fn update(&mut self, action: Action) {
        self.board.validate(action);
        self.board.place_mark(action, self.turn);
        self.turn = self.turn.other();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
pub struct Board {
    /// The board is a 3x3 grid of 3x3 grids
    /// board[i][j] is the sub-board at position (i, j)
    /// so board[0][0] is the top left sub-board
    /// and board[i][k][k][l] is the mark at position (k, l) in the sub-board at position (i, j)
    /// all indices go from 0 to 2
    /// 0 = top or left
    /// 2 = right or bottom
    board: [[[[Option<PlayerMark>; 3]; 3]; 3]; 3],
    /// The winner of the game
    /// None if the game is still running
    /// Some(None) if the game is a draw
    /// Some(Some(PlayerMark)) if the game is won by that player
    sup_board_status: GameStatus,
    /// The status of the super-board
    /// The super-board is a normal tictactoe board
    /// While being computable from the board, it is easier to keep track of it separately
    sup_board: [[GameStatus; 3]; 3],
    /// The last action taken decides the next board to play in
    /// In the first move, this is None
    last_action: Option<Action>,
}

impl Board {
    fn new() -> Self {
        Self {
            sup_board: [[GameStatus::Undecided; 3]; 3],
            board: [[[[None; 3]; 3]; 3]; 3],
            sup_board_status: GameStatus::Undecided,
            last_action: None,
        }
    }

    pub fn get_sup_board(&self) -> &[[GameStatus; 3]; 3] {
        &self.sup_board
    }

    /// The next move must be placed in this sub-board
    /// indexed 0-2
    pub fn target_board(&self) -> Option<(usize, usize)> {
        self.last_action.and_then(|a| {
            if self.sup_board[a.position.0][a.position.1] == GameStatus::Undecided {
                Some(a.position)
            } else {
                None
            }
        })
    }

    /// Mark the given position with the given player mark in the sub-board
    /// and if someone won that sub-board, mark the position in the sup-board
    /// and if someone won the sup-board, mark the winner
    /// and if the sup-board is full, mark the draw
    fn place_mark(&mut self, action: Action, mark: PlayerMark) {
        let sub_row = action.position.0;
        let sub_col = action.position.1;
        self.board[action.board.0][action.board.1][action.position.0][action.position.1] =
            Some(mark);

        // check if this player won the sub-board
        let sub_board = &self.board[action.board.0][action.board.1];
        let won_row = sub_board[sub_row][0] == Some(mark)
            && sub_board[sub_row][1] == Some(mark)
            && sub_board[sub_row][2] == Some(mark);
        let won_col = sub_board[0][sub_col] == Some(mark)
            && sub_board[1][sub_col] == Some(mark)
            && sub_board[2][sub_col] == Some(mark);
        let won_diag_se = sub_col == sub_row
            && sub_board[0][0] == Some(mark)
            && sub_board[1][1] == Some(mark)
            && sub_board[2][2] == Some(mark);
        let won_diag_ne = sub_col == 2 - sub_row
            && sub_board[0][2] == Some(mark)
            && sub_board[1][1] == Some(mark)
            && sub_board[2][0] == Some(mark);
        let won_sub_board = won_row || won_col || won_diag_se || won_diag_ne;
        if won_sub_board {
            self.sup_board[action.board.0][action.board.1] = GameStatus::Won(mark);
        }
        // check if the sub-board is a draw
        let sub_board_full = sub_board.iter().all(|row| row.iter().all(|&x| x.is_some()));
        if sub_board_full && !won_sub_board {
            self.sup_board[action.board.0][action.board.1] = GameStatus::Draw;
        }

        // check if the super-board is won
        let won_row = self.sup_board[sub_row][0] == GameStatus::Won(mark)
            && self.sup_board[sub_row][1] == GameStatus::Won(mark)
            && self.sup_board[sub_row][2] == GameStatus::Won(mark);
        let won_col = self.sup_board[0][sub_col] == GameStatus::Won(mark)
            && self.sup_board[1][sub_col] == GameStatus::Won(mark)
            && self.sup_board[2][sub_col] == GameStatus::Won(mark);
        let won_diag_se = sub_col == sub_row
            && self.sup_board[0][0] == GameStatus::Won(mark)
            && self.sup_board[1][1] == GameStatus::Won(mark)
            && self.sup_board[2][2] == GameStatus::Won(mark);
        let won_diag_ne = sub_col == 2 - sub_row
            && self.sup_board[0][2] == GameStatus::Won(mark)
            && self.sup_board[1][1] == GameStatus::Won(mark)
            && self.sup_board[2][0] == GameStatus::Won(mark);
        let won_super_board = won_row || won_col || won_diag_se || won_diag_ne;
        if won_super_board {
            self.sup_board_status = GameStatus::Won(mark);
        }

        // check if the super-board is a draw
        let super_board_full = self
            .sup_board
            .iter()
            .all(|row| row.iter().all(|&x| x != GameStatus::Undecided));
        if super_board_full && !won_super_board {
            self.sup_board_status = GameStatus::Draw;
        }

        self.last_action = Some(action);
    }

    /// Return `true`` if the move is a valid move
    fn validate(&self, action: Action) -> bool {
        if self.last_action.is_none() {
            return true;
        }

        let desired_target_board = self.last_action.unwrap().position;

        // if the target board is running, verify that the new move is in that board
        if self.sup_board[desired_target_board.0][desired_target_board.1] == GameStatus::Undecided
            && action.board != desired_target_board
        {
            return false;
        }

        // is the target position playable in the target board?
        if self.board[action.board.0][action.board.1][action.position.0][action.position.1]
            .is_some()
        {
            return false;
        }

        true
    }
    pub fn get_winner(&self) -> GameStatus {
        self.sup_board_status
    }
    pub fn n_moves_made(&self) -> usize {
        self.board
            .iter()
            .flatten()
            .flatten()
            .flatten()
            .filter(|&x| x.is_some())
            .count()
    }

    pub(crate) fn get_board(&self) -> &[[[[Option<PlayerMark>; 3]; 3]; 3]; 3] {
        &self.board
    }

    fn player_to_go(&self) -> PlayerMark {
        // count the number of noughts and crosses
        // if they are equal, it is naughts turn
        let noughts = self
            .board
            .iter()
            .flatten()
            .flatten()
            .flatten()
            .filter(|&x| *x == Some(PlayerMark::Naught))
            .count();
        let crosses = self
            .board
            .iter()
            .flatten()
            .flatten()
            .flatten()
            .filter(|&x| *x == Some(PlayerMark::Cross))
            .count();
        if noughts == crosses {
            PlayerMark::Naught
        } else {
            PlayerMark::Cross
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct Action {
    board: (usize, usize),
    position: (usize, usize),
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "(board: {:?}, position: {:?})",
            self.board, self.position
        )
    }
}

impl TryFrom<(usize, usize, usize, usize)> for Action {
    fn try_from(
        (board_x, board_y, pos_x, pos_y): (usize, usize, usize, usize),
    ) -> Result<Self, Self::Error> {
        if board_x > 2 || board_y > 2 || pos_x > 2 || pos_y > 2 {
            Err("Some index >2. Invalid action".to_string())
        } else {
            Ok(Self {
                board: (board_x, board_y),
                position: (pos_x, pos_y),
            })
        }
    }

    type Error = String;
}

impl Game for UltimateTicTacToe {
    type Board = Board;
    type Coordinate = Action;
    fn run(&mut self) {
        while self.board.sup_board_status == GameStatus::Undecided {
            let action = if self.turn == PlayerMark::Naught {
                self.player_naught.play(&self.board)
            } else {
                self.player_cross.play(&self.board)
            };
            self.update(action);
        }
        println!("{}", &self.board);
        if let GameStatus::Won(p) = self.board.sup_board_status {
            println!("Player {:?} won", p);
        }
        println!(
            "Game over. The game lasted {} moves. Thanks for playing!",
            self.board.n_moves_made()
        );
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut board = String::new();
        board.push_str(" --- --- --- \n");
        for i in 0..3 {
            for k in 0..3 {
                board.push('|');
                for j in 0..3 {
                    for l in 0..3 {
                        let mark = self.board[i][j][k][l];
                        let mark = match mark {
                            Some(PlayerMark::Naught) => "O",
                            Some(PlayerMark::Cross) => "X",
                            None => " ",
                        };
                        board.push_str(mark);
                    }
                    board.push('|');
                }
                board.push('\n');
            }
            board.push_str(" --- --- --- \n");
        }
        write!(f, "{}", board)
    }
}

impl BoardTrait<Action> for Board {
    fn valid_moves(&self) -> Vec<Action> {
        let mut moves = vec![];
        for i in 0..3 {
            for j in 0..3 {
                if self.sup_board[i][j] != GameStatus::Undecided {
                    continue;
                }
                for k in 0..3 {
                    for l in 0..3 {
                        let action = Action::try_from((i, j, k, l)).unwrap();
                        if self.validate(action) {
                            moves.push(action);
                        }
                    }
                }
            }
        }
        moves
    }
    fn place_mark(&mut self, a: Action, marker: PlayerMark) {
        self.place_mark(a, marker);
    }
    fn game_is_over(&self) -> bool {
        !matches!(self.sup_board_status, GameStatus::Undecided)
    }
}

impl Mdp for UltimateTicTacToe {
    type Action = Action;
    type State = Board;
    const DISCOUNT_FACTOR: f64 = -1.0;
    fn is_terminal(s: &Board) -> bool {
        s.game_is_over()
    }
    fn allowed_actions(s: &Self::State) -> Vec<Self::Action> {
        s.valid_moves()
    }
    fn act(s: Board, action: &Self::Action) -> (Board, f64) {
        let mut s = s;
        let player_mark = s.player_to_go();
        s.place_mark(*action, player_mark);
        let reward: f64 = match s.sup_board_status {
            GameStatus::Won(mark) => {
                if player_mark == mark {
                    1.0
                } else {
                    -1.0
                }
            }
            GameStatus::Draw => 0.0,
            _ => 0.0,
        };
        (s, reward)
    }
}
