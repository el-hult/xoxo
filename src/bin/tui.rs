use clap::{Parser, ValueEnum};
use rand::{rngs::StdRng, Rng as _, SeedableRng as _};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::hash::Hash;
use xoxo::{
    core::{run_game, Board, GameType, HeuristicFn, Player, PlayerMark},
    player::{
        alpha_beta::ABAi,
        c4_heuristic,
        console::ConsolePlayer,
        mcts::{get_c, MctsAi},
        min_max::MinMaxAi,
        random::RandomAi,
        ttt_heuristic, uttt_heuristic,
    },
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum PlayerType {
    Console,
    Random,
    Minimax,
    AlphaBeta,
    Mcts,
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

    /// Which game to play
    #[arg(long, default_value = "ttt")]
    game: GameType,

    /// The depth of the minimax algorithm
    /// Only used for minimax ai, if used
    #[arg(long, default_value = "3")]
    mm_depth: usize,

    /// The depth of the alpha-beta algorithm
    /// Only used for alpha-beta ai, if used
    #[arg(long, default_value = "7")]
    ab_depth: usize,

    /// The seed for the random number generator (when used)
    #[arg(long)]
    seed: Option<u64>,

    /// The exploration constant for the MCTS algorithm
    /// Only used for MCTS ai, if used
    /// If None, the value is determined by game-specific deafults
    #[arg(long)]
    c: Option<f64>,
}

fn make_player<T>(
    player_type: PlayerType,
    marker: PlayerMark,
    rng: &mut StdRng,
    mm_depth: usize,
    ab_depth: usize,
    c: f64,
    heuristic: HeuristicFn<T>,
) -> Box<dyn Player<T>>
where
    T: Board + Clone + Hash + Eq + Debug + 'static + Serialize + for <'de> Deserialize<'de>,
    ConsolePlayer: Player<T>,
    <T as Board>::Coordinate: Ord + Hash + Debug,
    for<'de> <T as Board>::Coordinate: Deserialize<'de> + Serialize
{
    match player_type {
        PlayerType::Console => Box::new(ConsolePlayer::new(marker)),
        PlayerType::Random => Box::new(RandomAi::new(rng.gen())),
        PlayerType::Minimax => Box::new(MinMaxAi::<T>::new(marker, heuristic, mm_depth)),
        PlayerType::AlphaBeta => Box::new(ABAi::<T>::new(marker, heuristic, ab_depth)),
        PlayerType::Mcts => Box::new(MctsAi::<T>::new(rng.gen(), c)),
    }
}

fn main() {
    let args = Args::parse();
    let seed = args.seed.unwrap_or(StdRng::from_entropy().gen());
    println!("AI seed: {}", seed); // debug output
    let mut rng = StdRng::seed_from_u64(seed);
    let c = match args.c {
        Some(c) => c,
        None => get_c(args.game),
    };
    match args.game {
        GameType::Ttt => {
            let p1 = make_player(
                args.p1,
                PlayerMark::Naught,
                &mut rng,
                args.mm_depth,
                args.ab_depth,
                c,
                ttt_heuristic,
            );
            let p2 = make_player(
                args.p2,
                PlayerMark::Cross,
                &mut rng,
                args.mm_depth,
                args.ab_depth,
                c,
                ttt_heuristic,
            );
            run_game(p1, p2)
        }
        GameType::Uttt => {
            let p1 = make_player(
                args.p1,
                PlayerMark::Naught,
                &mut rng,
                args.mm_depth,
                args.ab_depth,
                c,
                uttt_heuristic,
            );
            let p2 = make_player(
                args.p2,
                PlayerMark::Cross,
                &mut rng,
                args.mm_depth,
                args.ab_depth,
                c,
                uttt_heuristic,
            );
            run_game(p1, p2)
        }
        GameType::C4 => {
            let p1 = make_player(
                args.p1,
                PlayerMark::Naught,
                &mut rng,
                args.mm_depth,
                args.ab_depth,
                c,
                c4_heuristic,
            );
            let p2 = make_player(
                args.p2,
                PlayerMark::Cross,
                &mut rng,
                args.mm_depth,
                args.ab_depth,
                c,
                c4_heuristic,
            );
            run_game(p1, p2)
        }
    };
}
