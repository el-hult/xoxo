//! A program that lets two players play a game of tic-tac-toe.
//! It records the player names and the score they got to the file "score.csv"
//!

use clap::{Parser, Subcommand, ValueEnum};
use rand::Rng;
use std::io::Write;
use std::path::PathBuf;

use xoxo::{
    core::{run_game, GameEndStatus, GameType, PlayerMark},
    game::connect_four::C4Board,
    player::random::RandomAi,
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Sets a custom config file
    #[arg(short, long, default_value = "score.csv")]
    outfile: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run a game
    Run {
        #[arg(short, long)]
        game: GameType,

        /// what kind of player is player1? Available options:
        /// "random"
        #[arg(short = 'p', long)]
        player1: PlayerSpec,
        /// what kind of player is player? See alternatives for p1
        #[arg(short = 'q', long)]
        player2: PlayerSpec,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum PlayerSpec {
    Random,
}

fn main() -> Result<(), std::io::Error> {
    let args = Cli::parse();
    match args.command {
        Commands::Run {
            player1, player2, game: GameType::C4, ..
        } => {
            let result: GameEndStatus = run_c4(player1, player2);
            record_result(&args.outfile, player1, player2, result)
        },
        _ => todo!("Implement other games"),
    }
}

fn record_result(
    outfile: &PathBuf,
    player1: PlayerSpec,
    player2: PlayerSpec,
    result: GameEndStatus,
) -> Result<(), std::io::Error> {
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(outfile)?;
    writeln!(file, "{:?}, {:?}, {:?}", player1, player2, result)
}

fn run_c4(player1: PlayerSpec, player2: PlayerSpec) -> GameEndStatus {
    println!(
        "Running a game of connect4 with players {:?} and {:?}",
        player1, player2
    );
    let mut rng = rand::thread_rng();
    let p1 = match player1 {
        PlayerSpec::Random => Box::new(RandomAi::new(PlayerMark::Naught, rng.gen())),
    };
    let p2 = match player2 {
        PlayerSpec::Random => Box::new(RandomAi::new(PlayerMark::Cross, rng.gen())),
    };
    run_game::<C4Board>(p1, p2)
}
