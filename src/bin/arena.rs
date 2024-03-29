//! A program that lets two players play a game of tic-tac-toe.
//! It records the player names and the score they got to the file "score.csv"
//!

use clap::{Parser, Subcommand, ValueEnum};
use enum_iterator::{all, cardinality, Sequence};
use rand::rngs::ThreadRng;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::io::Seek;
use std::path::PathBuf;
use std::time::Duration;
use xoxo::{
    core::{BlitzPlayer, GameEndStatus, GameType, PlayerMark},
    game::{connect_four::C4Board, run_blitz_game, tictactoe::TTTBoard, ultimate_ttt::UTTTBoard},
    player::{c4_heuristic, ttt_heuristic, uttt_heuristic, ABAi, MctsAi, MinMaxAi, RandomAi},
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Sets a custom score keeping file
    #[arg(short, long, default_value = "score.csv")]
    outfile: PathBuf,

    /// Print out information about what the program is doing
    #[arg(long,short, action)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,

    ///Which game to report on
    game: GameType,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run a game
    Run {
        /// what kind of player is player1? Available options:
        /// "random"
        #[arg(short = 'p', long)]
        player1: PlayerSpec,
        /// what kind of player is player? See alternatives for p1
        #[arg(short = 'q', long)]
        player2: PlayerSpec,
    },
    /// Report on the results of the games in the terminal
    Report {},
}

#[derive(Debug, Clone, Copy, ValueEnum, Serialize, Deserialize, Sequence)]
enum PlayerSpec {
    Random,
    /// Minimax with depth 4, and alpha-beta pruning
    AB4,
    /// Minimax with depth 6, and alpha-beta pruning
    AB6,
    /// Minimax with depth 4
    Minimax4,
    /// MCTS Ai with c=2 in the UCB1 formula
    MCTS2,
    /// MCTS Ai with c=1 in the UCB1 formula
    MCTS1,
    /// MCTS Ai with c=0.5 in the UCB1 formula
    MCTS3,
}

#[derive(Serialize, Deserialize)]
struct GameRecord {
    game: GameType,
    player1: PlayerSpec,
    player2: PlayerSpec,
    result: GameEndStatus,
    played_at: chrono::DateTime<chrono::Local>,
    /// Time remaining for player 1 when game ended, in microseconds
    time1: u128,
    /// Time remaining for player 2 when game ended, in microseconds
    time2: u128,
}

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    let game = args.game;
    let log_level = if args.verbose {
        log::Level::Debug
    } else {
        log::Level::Info
    };
    simple_logger::init_with_level(log_level).unwrap();
    match args.command {
        Commands::Run {
            player1, player2, ..
        } => {
            let (result, time1, time2)  = match game {
                GameType::C4 => run_c4(player1, player2),
                GameType::Ttt => run_ttt(player1, player2),
                GameType::Uttt => run_uttt(player1, player2),
            };
            let record = GameRecord {
                game,
                player1,
                player2,
                result,
                played_at: chrono::Local::now(),
                time1: time1.as_micros(),
                time2: time2.as_micros(),
            };
            record_result(&args.outfile, record)
        }
        Commands::Report {} => print_out_report(&args.outfile, game),
    }
}

fn print_out_report(outfile: &PathBuf, game_to_report: GameType) -> anyhow::Result<()> {
    let mut n_wins = [[0.0; cardinality::<PlayerSpec>()]; cardinality::<PlayerSpec>()];
    let mut n_draws = [[0.0; cardinality::<PlayerSpec>()]; cardinality::<PlayerSpec>()];
    let mut n_losses = [[0.0; cardinality::<PlayerSpec>()]; cardinality::<PlayerSpec>()];

    // Iterate the lines in the file, and for each line, update the n_wins and n_games
    let file = std::fs::File::open(outfile).map_err(|e| anyhow::anyhow!(
        format!("Failed to open the file {:?} for reading scores. {}", outfile, e)))?;
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
        let p1num = player1 as usize;
        let p2num = player2 as usize;
        match result {
            GameEndStatus::Draw => {
                n_draws[p1num][p2num] += 1.0
            }
            GameEndStatus::O => {
                n_wins[p1num][p2num] += 1.0;
            }
            GameEndStatus::X => {
                n_losses[p1num][p2num] += 1.0;
            }
        }
    }
    print_result_matrix(n_wins, n_draws, n_losses);
    Ok(())
}

/// Print the result matrix
fn print_result_matrix<const N: usize>(
    n_wins: [[f64; N]; N],
    n_draws: [[f64; N]; N],
    n_losses: [[f64; N]; N],
) {
    println!("Wins/draws/losses");
    println!("Row=player1, Column=player2");
    print!("{:>10}  ", "");
    let player_labels: Vec<String> = all::<PlayerSpec>()
        .map(|x| {
            serde_json::to_string(&x)
                .expect("This serialization should not fail")
                .strip_prefix('\"')
                .unwrap()
                .strip_suffix('\"')
                .unwrap()
                .chars()
                .take(8)
                .collect()
        })
        .collect::<Vec<_>>();
    if player_labels.len() != N {
        panic!("The number of player labels is not equal to the number of players");
    }
    for j in player_labels.iter() {
        print!("{:>10}  ", j);
    }
    println!();
    for i in 0..N {
        print!("{:>10}  ", player_labels[i]);
        for j in 0..N {
            print!("{:3}/{:2}/{:3}  ", n_wins[i][j], n_draws[i][j], n_losses[i][j]);
        }
        println!();
    }
}

fn record_result(outfile: &PathBuf, record: GameRecord) -> anyhow::Result<()> {
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(outfile)?;
    let needs_headers = file.seek(std::io::SeekFrom::End(0))? == 0;
    let mut wtr = csv::WriterBuilder::new()
        .has_headers(needs_headers)
        .from_writer(file);

    wtr.serialize(record)?;
    wtr.flush()?;
    Ok(())
}



fn make_player_c4(
    p: PlayerSpec,
    mark: PlayerMark,
    rng: &mut ThreadRng,
) -> Box<dyn BlitzPlayer<C4Board>> {
    match p {
        PlayerSpec::Random => Box::new(RandomAi::new(rng.gen())),
        PlayerSpec::Minimax4 => Box::new(MinMaxAi::new(mark, c4_heuristic, 4)),
        PlayerSpec::AB4 => Box::new(ABAi::new(mark, c4_heuristic, 4)),
        PlayerSpec::AB6 => Box::new(ABAi::new(mark, c4_heuristic, 6)),
        PlayerSpec::MCTS1 => Box::new(MctsAi::<C4Board>::new(rng.gen(), 1.0, Some(format!("mcts1.{}.c4.data",mark)))),
        PlayerSpec::MCTS2 => Box::new(MctsAi::<C4Board>::new(rng.gen(), 2.0, Some(format!("mcts2.{}.c4.data",mark)))),
        PlayerSpec::MCTS3 => Box::new(MctsAi::<C4Board>::new(rng.gen(), 0.5, Some(format!("mcts3.{}.c4.data",mark)))),
    }
}

static T0: Duration = Duration::from_secs(1);

fn make_player_ttt(
    p: PlayerSpec,
    mark: PlayerMark,
    rng: &mut ThreadRng,
) -> Box<dyn BlitzPlayer<TTTBoard>> {
    match p {
        PlayerSpec::Random => Box::new(RandomAi::new(rng.gen())),
        PlayerSpec::Minimax4 => Box::new(MinMaxAi::new(mark, ttt_heuristic, 4)),
        PlayerSpec::AB4 => Box::new(ABAi::new(mark, ttt_heuristic, 4)),
        PlayerSpec::AB6 => Box::new(ABAi::new(mark, ttt_heuristic, 6)),
        PlayerSpec::MCTS1 => Box::new(MctsAi::<TTTBoard>::new(rng.gen(), 1.0, Some(format!("mcts1.{}.ttt.data",mark)))),
        PlayerSpec::MCTS2 => Box::new(MctsAi::<TTTBoard>::new(rng.gen(), 2.0, Some(format!("mcts2.{}.ttt.data",mark)))),
        PlayerSpec::MCTS3 => Box::new(MctsAi::<TTTBoard>::new(rng.gen(), 0.5, Some(format!("mcts3.{}.ttt.data",mark)))),
    }
}
fn make_player_uttt(
    p: PlayerSpec,
    mark: PlayerMark,
    rng: &mut ThreadRng,
) -> Box<dyn BlitzPlayer<UTTTBoard>> {
    match p {
        PlayerSpec::Random => Box::new(RandomAi::new(rng.gen())),
        PlayerSpec::Minimax4 => Box::new(MinMaxAi::new(mark, uttt_heuristic, 4)),
        PlayerSpec::AB4 => Box::new(ABAi::new(mark, uttt_heuristic, 4)),
        PlayerSpec::AB6 => Box::new(ABAi::new(mark, uttt_heuristic, 6)),
        PlayerSpec::MCTS1 => Box::new(MctsAi::<UTTTBoard>::new(rng.gen(), 1.0, Some(format!("mcts1.{}.uttt.data",mark)))),
        PlayerSpec::MCTS2 => Box::new(MctsAi::<UTTTBoard>::new(rng.gen(), 2.0, Some(format!("mcts2.{}.uttt.data",mark)))),
        PlayerSpec::MCTS3 => Box::new(MctsAi::<UTTTBoard>::new(rng.gen(), 0.5, Some(format!("mcts3.{}.uttt.data",mark)))),
    }
}

fn run_c4(player1: PlayerSpec, player2: PlayerSpec) -> (GameEndStatus, Duration, Duration) {
    let mut rng = rand::thread_rng();
    let p1 = make_player_c4(player1, PlayerMark::Naught, &mut rng);
    let p2 = make_player_c4(player2, PlayerMark::Cross, &mut rng);
    run_blitz_game::<C4Board>(p1, p2,T0)
}
fn run_ttt(player1: PlayerSpec, player2: PlayerSpec) -> (GameEndStatus, Duration, Duration) {
    let mut rng = rand::thread_rng();
    let p1 = make_player_ttt(player1, PlayerMark::Naught, &mut rng);
    let p2 = make_player_ttt(player2, PlayerMark::Cross, &mut rng);
    run_blitz_game::<TTTBoard>(p1, p2,T0)
}
fn run_uttt(player1: PlayerSpec, player2: PlayerSpec) -> (GameEndStatus, Duration, Duration) {
    let mut rng = rand::thread_rng();
    let p1 = make_player_uttt(player1, PlayerMark::Naught, &mut rng);
    let p2 = make_player_uttt(player2, PlayerMark::Cross, &mut rng);
    run_blitz_game::<UTTTBoard>(p1, p2,T0)
}
