use std::{io::BufRead, ops::Sub};

use crate::{
    core::{Player, PlayerMark},
    game::{
        connect_four::C4Board,
        tictactoe::{self, TTTBoard},
        ultimate_ttt::{self},
    },
};

pub struct ConsolePlayer {
    pub name: String,
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
impl Player<TTTBoard> for ConsolePlayer {
    fn play(&mut self, b: &TTTBoard) -> tictactoe::TTTAddr {
        println!("Time for {} to make a move", self.name);
        print!("{}", b);
        println!("Input a number 1-9 to make a move 1 = top left, 9 = bottom right");
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
        if !(1..=9).contains(&num) {
            eprintln!("Number not in range 1-N_SQUARES");
        }
        println!("Got {}", num);
        tictactoe::TTTAddr(num)
    }
}
impl Player<ultimate_ttt::Board> for ConsolePlayer {
    fn play(&mut self, b: &ultimate_ttt::Board) -> ultimate_ttt::Action {
        println!("Time for {} to make a move", self.name);
        print!("{}", b);
        println!("Input four numbers 1-3 to make a move 1 = top left, 3 = bottom right");
        if let Some(target_board) = b.target_board() {
            println!(
                "You must play in board {},{}",
                target_board.0 + 1,
                target_board.1 + 1
            );
        } else {
            println!("You can play in any board");
        }
        println!("board-row board-col pos-row pos-col");
        let mut line = String::new();
        std::io::stdin()
            .lock()
            .read_line(&mut line)
            .expect("Could not read line");
        let nums = line
            .split_ascii_whitespace()
            .map(|x| x.parse::<usize>().expect("Must input number") - 1)
            .collect::<Vec<_>>();
        if nums.len() != 4 {
            eprintln!("Must input 4 numbers, 1-3 with space in between. i j k l represents board on row i, column j, and in that board, play position row k col l");
        }
        ultimate_ttt::Action::try_from((nums[0], nums[1], nums[2], nums[3])).unwrap()
    }
}

impl Player<C4Board> for ConsolePlayer {
    fn play(&mut self, b: &C4Board) -> usize {
        println!("Time for {} to make a move", self.name);
        print!("{}", b);
        println!("Input a number 1-7 to make a move 1 = leftmost, 7 = rightmost");
        let mut line = String::new();
        std::io::stdin()
            .lock()
            .read_line(&mut line)
            .expect("Could not read line");
        let num = line
            .chars()
            .next()
            .expect("At least one character must be input");
        let num: usize = num.to_string().parse::<_>().expect("Must input number");
        if !(1..=7).contains(&num) {
            eprintln!("Number not in range 1-7");
        }
        println!("Got {}", num);
        num.sub(1)
    }
}
