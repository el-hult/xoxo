//! A program that lets two players play a game of tic-tac-toe.
//! It records the player names and the score they got to the file "score.csv"
//!

use clap::{Parser, Subcommand, ValueEnum};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::io::Seek;
use std::path::PathBuf;
use xoxo::core::PlayerMark;

use xoxo::{
    core::{run_game_silent, GameEndStatus, GameType},
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
    Report {
        ///Which game to report on
        #[arg(short, long)]
        game: GameType,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum, Serialize, Deserialize)]
enum PlayerSpec {
    Random,
}
impl PlayerSpec {
    const VARIANTS: usize = 1;
    fn variant_number(&self) -> usize {
        match self {
            PlayerSpec::Random => 0,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct GameRecord {
    game: GameType,
    player1: PlayerSpec,
    player2: PlayerSpec,
    result: GameEndStatus,
    played_at: chrono::DateTime<chrono::Local>,
}

fn main() -> Result<(), std::io::Error> {
    let args = Cli::parse();
    match args.command {
        Commands::Run {
            player1,
            player2,
            game: GameType::C4,
            ..
        } => {
            let result: GameEndStatus = run_c4(player1, player2);
            record_result(&args.outfile, GameType::C4, player1, player2, result)
        }
        Commands::Report { game } => print_out_report(&args.outfile, game),
        _ => todo!("Implement the rest"),
    }
}

fn print_out_report(outfile: &PathBuf, game_to_report: GameType) -> Result<(), std::io::Error> {
    let mut n_wins = vec![vec![0.0; PlayerSpec::VARIANTS]; PlayerSpec::VARIANTS];
    let mut n_games = vec![vec![0.0; PlayerSpec::VARIANTS]; PlayerSpec::VARIANTS];

    // Iterate the lines in the file, and for each line, update the n_wins and n_games
    let file = std::fs::File::open(outfile).expect(&format!(
        "The report file {} does not exist",
        outfile.to_str().unwrap()
    ));
    let mut rdr = csv::Reader::from_reader(file);
    for line in rdr.deserialize() {
        let GameRecord {
            game,
            player1,
            player2,
            result,
            ..
        } = line?;
        if game_to_report != game {
            continue;
        }
        let p1num = player1.variant_number();
        let p2num = player2.variant_number();
        n_games[p1num][p2num] += 1.0;
        n_games[p2num][p1num] += 1.0;
        match result {
            GameEndStatus::Draw => {
                n_wins[p1num][p2num] += 0.5;
                n_wins[p2num][p1num] += 0.5;
            }
            GameEndStatus::Won(PlayerMark::Naught) => {
                n_wins[p1num][p2num] += 1.0;
            }
            GameEndStatus::Won(PlayerMark::Cross) => {
                n_wins[p2num][p1num] += 1.0;
            }
        }
    }
    println!("Wins: {:?}", n_wins);
    println!("Games: {:?}", n_games);
    Ok(())
}

fn record_result(
    outfile: &PathBuf,
    game: GameType,
    player1: PlayerSpec,
    player2: PlayerSpec,
    result: GameEndStatus,
) -> Result<(), std::io::Error> {
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(outfile)?;
    let needs_headers = file.seek(std::io::SeekFrom::End(0))? == 0;
    let mut wtr = csv::WriterBuilder::new()
        .has_headers(needs_headers)
        .from_writer(file);

    let record = GameRecord {
        game,
        player1,
        player2,
        result,
        played_at: chrono::Local::now(),
    };
    wtr.serialize(record)?;
    wtr.flush()?;
    Ok(())
}

fn run_c4(player1: PlayerSpec, player2: PlayerSpec) -> GameEndStatus {
    let mut rng = rand::thread_rng();
    let p1 = match player1 {
        PlayerSpec::Random => Box::new(RandomAi::new(rng.gen())),
    };
    let p2 = match player2 {
        PlayerSpec::Random => Box::new(RandomAi::new(rng.gen())),
    };
    run_game_silent::<C4Board>(p1, p2)
}
