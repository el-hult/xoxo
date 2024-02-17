use clap::{Parser, ValueEnum};
use rand::{rngs::StdRng, Rng as _, SeedableRng as _};
use xoxo::{
    core::{run_game_verbose, GameType, Player, PlayerMark},
    game::{connect_four::C4Board, tictactoe::TTTBoard, ultimate_ttt::UTTTBoard},
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
            let p1: Box<dyn Player<TTTBoard>> = match args.p1 {
                PlayerType::Console => Box::new(ConsolePlayer::new(PlayerMark::Naught)),
                PlayerType::Random => Box::new(RandomAi::new(rng.gen())),
                PlayerType::Minimax => Box::new(MinMaxAi::<TTTBoard>::new(
                    PlayerMark::Naught,
                    ttt_heuristic,
                    args.mm_depth,
                )),
                PlayerType::AlphaBeta => Box::new(ABAi::<TTTBoard>::new(
                    PlayerMark::Naught,
                    ttt_heuristic,
                    args.ab_depth,
                )),
                PlayerType::Mcts => Box::new(MctsAi::<TTTBoard>::new(rng.gen(), c)),
            };
            let p2: Box<dyn Player<TTTBoard>> = match args.p2 {
                PlayerType::Console => Box::new(ConsolePlayer::new(PlayerMark::Cross)),
                PlayerType::Random => Box::new(RandomAi::new(rng.gen())),
                PlayerType::Minimax => Box::new(MinMaxAi::<TTTBoard>::new(
                    PlayerMark::Cross,
                    ttt_heuristic,
                    args.mm_depth,
                )),
                PlayerType::AlphaBeta => Box::new(ABAi::<TTTBoard>::new(
                    PlayerMark::Cross,
                    ttt_heuristic,
                    args.ab_depth,
                )),
                PlayerType::Mcts => Box::new(MctsAi::<TTTBoard>::new(rng.gen(), c)),
            };
            run_game_verbose::<TTTBoard>(p1, p2)
        }
        GameType::Uttt => {
            let p1: Box<dyn Player<UTTTBoard>> = match args.p1 {
                PlayerType::Console => Box::new(ConsolePlayer::new(PlayerMark::Naught)),
                PlayerType::Random => Box::new(RandomAi::new(rng.gen())),
                PlayerType::Minimax => Box::new(MinMaxAi::<UTTTBoard>::new(
                    PlayerMark::Naught,
                    uttt_heuristic,
                    args.mm_depth,
                )),
                PlayerType::AlphaBeta => Box::new(ABAi::<UTTTBoard>::new(
                    PlayerMark::Naught,
                    uttt_heuristic,
                    args.ab_depth,
                )),
                PlayerType::Mcts => Box::new(MctsAi::<UTTTBoard>::new(rng.gen(), c)),
            };
            let p2: Box<dyn Player<UTTTBoard>> = match args.p2 {
                PlayerType::Console => Box::new(ConsolePlayer::new(PlayerMark::Cross)),
                PlayerType::Random => Box::new(RandomAi::new(rng.gen())),
                PlayerType::Minimax => Box::new(MinMaxAi::<UTTTBoard>::new(
                    PlayerMark::Cross,
                    uttt_heuristic,
                    args.mm_depth,
                )),
                PlayerType::AlphaBeta => Box::new(ABAi::<UTTTBoard>::new(
                    PlayerMark::Cross,
                    uttt_heuristic,
                    args.ab_depth,
                )),
                PlayerType::Mcts => Box::new(MctsAi::<UTTTBoard>::new(rng.gen(), c)),
            };
            run_game_verbose::<UTTTBoard>(p1, p2)
        }
        GameType::C4 => {
            let p1: Box<dyn Player<C4Board>> = match args.p1 {
                PlayerType::Console => Box::new(ConsolePlayer::new(PlayerMark::Naught)),
                PlayerType::Random => Box::new(RandomAi::new(rng.gen())),
                PlayerType::Minimax => Box::new(MinMaxAi::<C4Board>::new(
                    PlayerMark::Naught,
                    c4_heuristic,
                    args.mm_depth,
                )),
                PlayerType::AlphaBeta => Box::new(ABAi::<C4Board>::new(
                    PlayerMark::Naught,
                    c4_heuristic,
                    args.ab_depth,
                )),
                PlayerType::Mcts => Box::new(MctsAi::<C4Board>::new(rng.gen(), c)),
            };
            let p2: Box<dyn Player<C4Board>> = match args.p2 {
                PlayerType::Console => Box::new(ConsolePlayer::new(PlayerMark::Cross)),
                PlayerType::Random => Box::new(RandomAi::new(rng.gen())),
                PlayerType::Minimax => Box::new(MinMaxAi::<C4Board>::new(
                    PlayerMark::Cross,
                    c4_heuristic,
                    args.mm_depth,
                )),
                PlayerType::AlphaBeta => Box::new(ABAi::<C4Board>::new(
                    PlayerMark::Cross,
                    c4_heuristic,
                    args.ab_depth,
                )),
                PlayerType::Mcts => Box::new(MctsAi::<C4Board>::new(rng.gen(), c)),
            };
            run_game_verbose::<C4Board>(p1, p2)
        }
    };
}
