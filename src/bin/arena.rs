//! A program that lets two players play a game of tic-tac-toe.
//! It records the player names and the score they got to the file "score.csv"
//!

use clap::{Parser, Subcommand, ValueEnum};
use rand::rngs::ThreadRng;
use rand::Rng;
use serde::{Deserialize, Serialize};
use xoxo::player::{c4_heuristic, ABAi};
use std::io::Seek;
use std::path::PathBuf;
use std::time::Duration;
use xoxo::core::{BlitzPlayer, Board, GameStatus, PlayerMark};

use xoxo::{
    core::{GameEndStatus, GameType},
    game::connect_four::C4Board,
    player::{RandomAi,MinMaxAi},
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
    Minimax3,
    AB5,
}
impl PlayerSpec {
    const VARIANTS: usize = 3;
    fn variant_number(&self) -> usize {
        match self {
            PlayerSpec::Random => 0,
            PlayerSpec::Minimax3 => 1,
            PlayerSpec::AB5 => 2,
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



fn main() -> anyhow::Result<()> {
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

fn print_out_report(outfile: &PathBuf, game_to_report: GameType) -> anyhow::Result<()> {
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
    println!("Wins:\n {:?}", n_wins);
    println!("Games:\n {:?}", n_games);
    Ok(())
}

fn record_result(
    outfile: &PathBuf,
    game: GameType,
    player1: PlayerSpec,
    player2: PlayerSpec,
    result: GameEndStatus,
) -> anyhow::Result<()> {
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

fn run_game_silent<B: Board>(mut p1: Box<dyn BlitzPlayer<B>>, mut p2: Box<dyn BlitzPlayer<B>>) -> GameEndStatus {
    let mut current_player = PlayerMark::Naught;
    let mut board = B::default();
    let mut time_remaining_naughts = std::time::Duration::from_millis(1000);
    let mut time_remaining_crosses = std::time::Duration::from_millis(1000);
    while !board.game_is_over() {
        let t0  = std::time::Instant::now();
        let action = match current_player {
            PlayerMark::Naught => p1.blitz(&board,time_remaining_naughts),
            PlayerMark::Cross => p2.blitz(&board,time_remaining_crosses),
        };
        let t1 = std::time::Instant::now();
        match current_player {
            PlayerMark::Naught => {
                time_remaining_naughts = time_remaining_naughts.checked_sub(t1.duration_since(t0)).unwrap_or(Duration::ZERO);
                dbg!(time_remaining_naughts);
                if time_remaining_naughts == Duration::ZERO { return GameEndStatus::Won(PlayerMark::Cross)}},
            PlayerMark::Cross => {
                time_remaining_crosses = time_remaining_crosses.checked_sub(t1.duration_since(t0)).unwrap_or(Duration::ZERO);
                dbg!(time_remaining_crosses);
                if time_remaining_crosses == Duration::ZERO { return GameEndStatus::Won(PlayerMark::Naught)}},
        }
        board.place_mark(action, current_player);
        current_player = current_player.other();
    }
    match board.game_status() {
        GameStatus::Draw => GameEndStatus::Draw,
        GameStatus::Won(p) => GameEndStatus::Won(p),
        GameStatus::Undecided => unreachable!(),
    }
}

fn make_player(p: PlayerSpec, mark: PlayerMark, rng: &mut ThreadRng) -> Box<dyn BlitzPlayer<C4Board>> {
    match p {
        PlayerSpec::Random => Box::new(RandomAi::new(rng.gen())),
        PlayerSpec::Minimax3 => Box::new(MinMaxAi::new(mark, c4_heuristic,  3)),
        PlayerSpec::AB5 => Box::new(ABAi::new(mark, c4_heuristic,  5)),
    }
}

fn run_c4(player1: PlayerSpec, player2: PlayerSpec) -> GameEndStatus {
    let mut rng = rand::thread_rng();
    let p1 = make_player(player1, PlayerMark::Naught, &mut rng);
    let p2 = make_player(player2, PlayerMark::Cross, &mut rng);
    run_game_silent::<C4Board>(p1, p2)
}
