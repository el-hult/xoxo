//! Monte Carlo Tree Search -- general implementation
//! This holds a general implementation of the MCTS algorithm, which is a general algorithm for solving MDPs where the state
//! evolves in a tree-like structure. It will not work if you can have cycles in the state space.
//!
//! It also holds a Ai struct, that knows how to play a game using MCTS, assuming the MDP structure of the games is known.

use itertools::Itertools as _;
use rand::prelude::SliceRandom;
use rand::rngs::StdRng;
use rand::seq::IteratorRandom as _;
use rand::SeedableRng;

use std::hash::Hash;
use std::{collections::HashMap, fmt::Debug};

use crate::core::{GameType,Board, GameStatus, Player};

pub(crate) trait Mdp {
    type Action: Clone + Debug + PartialEq + Eq + Hash + Ord;
    type State: Sized + Debug + Clone + PartialEq + Eq + Hash;
    const DISCOUNT_FACTOR: f64; // 1= no discount, 0=only immediate reward
    fn act(s: Self::State, action: &Self::Action) -> (Self::State, f64); // This is sampling from the Sutton&Barto's p(s',r|s,a), equation 3.2
    fn is_terminal(s: &Self::State) -> bool;
    fn allowed_actions(s: &Self::State) -> Vec<Self::Action>;
    /// If the game branch factor is large, this random strategy is bad, since it will explore very inefficiently
    /// What better enginges do is to use some heuristic for the Q-function to do the rollout.
    fn rollout(s: Self::State, rng: &mut StdRng) -> f64 {
        if Self::is_terminal(&s) {
            return 0.0;
        }
        let actions = Self::allowed_actions(&s);
        let action = actions.choose(rng).expect(
            "This function should never have been called on a state with no actions allowed",
        );
        let (state, reward) = Self::act(s, action);
        reward + Self::DISCOUNT_FACTOR * Self::rollout(state, rng)
    }
}

pub(crate) fn mcts_step<M: Mdp>(
    state: &M::State,
    c: f64,
    state_visit_counter: &mut HashMap<M::State, f64>,
    qmap: &mut QMap<M>,
    rng: &mut StdRng,
) -> f64 {
    if M::is_terminal(state) {
        return 0.0;
    }
    let t = *state_visit_counter.get(state).unwrap_or(&0.0);
    let best_action = best_action::<M>(state, c, qmap, state_visit_counter, rng);
    let (new_state, reward) = M::act(state.clone(), &best_action);
    let state_was_new = state_visit_counter.get(state).is_none();
    let g_return = if state_was_new {
        reward + M::rollout(new_state, rng) * M::DISCOUNT_FACTOR
    } else {
        reward + mcts_step::<M>(&new_state, c, state_visit_counter, qmap, rng) * M::DISCOUNT_FACTOR
    };

    // Update the Q-function and the visit counter
    state_visit_counter.insert(state.clone(), t + 1.0);
    let state_action_key = (state.clone(), best_action.clone());
    let (w, v) = qmap.get(&state_action_key).unwrap_or(&(0.0, 0.0));
    qmap.insert(state_action_key, (w + g_return, v + 1.0));

    g_return
}

pub(crate) type QMap<M> = HashMap<(<M as Mdp>::State, <M as Mdp>::Action), (f64, f64)>;

pub(crate) fn best_action<M: Mdp>(
    state: &M::State,
    c: f64,
    qmap: &QMap<M>,
    state_visit_counter: &HashMap<M::State, f64>,
    rng: &mut StdRng,
) -> M::Action {
    let allowed_actions = M::allowed_actions(state);
    let t = *state_visit_counter.get(state).unwrap_or(&0.0);
    let best_action = allowed_actions
        .into_iter()
        .map(|action| {
            let (w, v) = qmap
                .get(&(state.clone(), action.clone()))
                .unwrap_or(&(0.0, 0.0));
            (action, ucb(c, *w, *v, t))
        })
        .max_set_by(|(_, ucb1), (_, ucb2)| ucb1.partial_cmp(ucb2).unwrap())
        .into_iter()
        .map(|(action, _)| action)
        .choose(rng)
        .expect("There must be at least one action");
    best_action
}

pub(crate) fn run_train_steps<M: Mdp>(
    b: &M::State,
    c: f64,
    qmap: &mut QMap<M>,
    state_visit_counter: &mut HashMap<M::State, f64>,
    rng: &mut StdRng,
    n_rounds: usize,
) {
    for _ in 0..n_rounds {
        mcts_step::<M>(b, c, state_visit_counter, qmap, rng);
    }
}

/// The UCB1 formula,
/// the constant c needs to be passed in.
/// by default, c=2.0 is often used
/// if the branching factor is large, c should be smaller
fn ucb(c: f64, tot_g: f64, n_visists: f64, time: f64) -> f64 {
    if n_visists == 0.0 {
        return f64::INFINITY;
    }
    tot_g / n_visists + c * (time.ln() / n_visists).sqrt()
}

#[cfg(test)]
mod test {

    use super::*;
    use rand::{thread_rng, Rng, SeedableRng};

    #[derive(Debug, Clone, PartialEq, Hash, Eq)]
    struct CountGameState(Vec<i8>);
    #[derive(Clone, Debug, PartialEq, Copy, Hash, Eq, Ord, PartialOrd)]
    enum CountGameAction {
        Add,
        Sub,
    }
    struct CountGameMDP {}
    impl Mdp for CountGameMDP {
        type Action = CountGameAction;
        type State = CountGameState;
        const DISCOUNT_FACTOR: f64 = 0.99;
        fn is_terminal(s: &CountGameState) -> bool {
            let total = s.0.iter().sum::<i8>();
            total <= -10 || total >= 10
        }
        fn act(s: CountGameState, action: &Self::Action) -> (CountGameState, f64) {
            let mut s = s;
            match action {
                CountGameAction::Add => s.0.push(thread_rng().gen_range(-1..=3)),
                CountGameAction::Sub => s.0.push(thread_rng().gen_range(-3..=1)),
            };
            let reward = if s.0.iter().sum::<i8>() >= 10 {
                1.0
            } else {
                0.0
            }; // reward is 1.0 for winning
            (s, reward)
        }
        fn allowed_actions(_s: &Self::State) -> Vec<Self::Action> {
            vec![CountGameAction::Add, CountGameAction::Sub]
        }
    }

    // If I take two steps, will both children be visited once?
    #[test]
    fn test_mcts_step() {
        let root: CountGameState = CountGameState(vec![]);
        let mut state_visit_counter = HashMap::new();
        let mut qmap = HashMap::new();
        let mut rng = StdRng::from_entropy();
        let c = 0.75;
        mcts_step::<CountGameMDP>(&root, c, &mut state_visit_counter, &mut qmap, &mut rng);
        mcts_step::<CountGameMDP>(&root, c, &mut state_visit_counter, &mut qmap, &mut rng);
        // The root state should have been visited twice
        assert!(state_visit_counter.contains_key(&root));
        assert_eq!(state_visit_counter[&root], 2.0);
        // check that there are two actions in the qmap associated with the root state
        let visits = qmap
            .iter()
            .filter(|((s, _), (_, _))| s == &root)
            .map(|(_, (_, v))| v)
            .collect::<Vec<_>>();
        dbg!(&qmap);
        assert_eq!(visits.len(), 2);
        assert_eq!(*visits[0], 1.0);
        assert_eq!(*visits[1], 1.0);
    }

    // If I run the game many times, Have I identified the best move?
    #[test]
    fn test_mcts() {
        let root: CountGameState = CountGameState(vec![]);
        let mut state_visit_counter = HashMap::new();
        let mut qmap = HashMap::new();
        let mut rng = StdRng::from_entropy();
        let c = 0.75;
        run_train_steps::<CountGameMDP>(
            &root,
            c,
            &mut qmap,
            &mut state_visit_counter,
            &mut rng,
            1000,
        );
        let best_move =
            best_action::<CountGameMDP>(&root, c, &qmap, &state_visit_counter, &mut rng);
        assert_eq!(
            best_move,
            CountGameAction::Add,
            "Stochastic test that might fail sometimes"
        );
    }
}

pub(crate) struct MctsAi<T: Mdp> {
    qmap: QMap<T>,
    state_visit_counter: HashMap<T::State, f64>,
    rng: StdRng,
    c: f64,
}

impl<T: Mdp> MctsAi<T> {
    /// seed is for the RNG, c is the exploration constant in the UCB1 formula
    pub fn new(seed: u64, c: f64) -> Self {
        MctsAi {
            qmap: QMap::<T>::new(),
            state_visit_counter: HashMap::new(),
            rng: StdRng::seed_from_u64(seed),
            c,
        }
    }
}

impl<T, B> Player<B> for MctsAi<T>
where
    T: Mdp<Action = B::Coordinate, State = B>,
    B: Board,
{
    fn play(&mut self, b: &B) -> B::Coordinate {
        run_train_steps::<T>(
            b,
            self.c,
            &mut self.qmap,
            &mut self.state_visit_counter,
            &mut self.rng,
            10000,
        );
        let a = best_action::<T>(
            b,
            self.c,
            &self.qmap,
            &self.state_visit_counter,
            &mut self.rng,
        );
        println!("MCTS AI played {}", a);
        a
    }
}

pub(crate) fn get_c(game: GameType) -> f64 {
    match game {
        GameType::Ttt => 1.0,
        GameType::Uttt => 0.75,
        GameType::C4 => 1.0,
    }
}

impl<B: Board> Mdp for B
where
    B::Coordinate: Ord + Hash + Debug,
    B: Hash + Eq + Clone + Debug,
{
    type Action = B::Coordinate;

    type State = B;

    const DISCOUNT_FACTOR: f64 = -1.0;

    fn act(mut board: Self::State, action: &Self::Action) -> (Self::State, f64) {
        let player_mark = board.current_player();
        board.place_mark(*action, player_mark);
        let reward: f64 = match board.game_status() {
            GameStatus::Won(mark) => {
                if player_mark == mark {
                    1.0
                } else {
                    -1.0
                }
            }
            _ => 0.0,
        };
        (board, reward)
    }

    fn is_terminal(s: &Self::State) -> bool {
        !matches!(s.game_status(), GameStatus::Undecided)
    }

    fn allowed_actions(s: &Self::State) -> Vec<Self::Action> {
        s.valid_moves()
    }
}
