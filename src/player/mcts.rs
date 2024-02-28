//! Monte Carlo Tree Search -- general implementation
//! This holds a general implementation of the MCTS algorithm, which is a general algorithm for solving MDPs where the state
//! evolves in a tree-like structure. It will not work if you can have cycles in the state space.
//!
//! It also holds a Ai struct, that knows how to play a game using MCTS, assuming the MDP structure of the games is known.

use log::info;
use rand::prelude::SliceRandom;
use rand::rngs::StdRng;
use rand::seq::IteratorRandom as _;
use rand::SeedableRng;
use serde::{Deserialize, Serialize};

use std::hash::Hash;
use std::io::{Read, Write};
use std::time::Duration;
use std::{collections::HashMap, fmt::Debug};

use crate::core::{BlitzPlayer, Board, GameStatus, Player};

pub trait Mdp {
    type Action: Clone
        + Debug
        + PartialEq
        + Eq
        + Hash
        + Ord
        + Serialize
        + for<'de> serde::Deserialize<'de>;
    type State: Sized
        + Debug
        + Clone
        + PartialEq
        + Eq
        + Hash
        + Serialize
        + for<'de> serde::Deserialize<'de>;
    const DISCOUNT_FACTOR: f64; // 1= no discount, 0=only immediate reward
    /// Sample sample from  p(s',r|s,a)
    /// see Sutton&Barto Equation 3.2
    /// The return is always as percieved by the actor that takes the action
    /// In 2 player games, this means that the reward has to be negated if the action is taken by the 'other' player
    /// One trick is to have a negative discount factor
    /// This can make life tricky if recording the returns and not keeping track on which player is the current player
    fn act(s: Self::State, action: &Self::Action) -> (Self::State, f64);
    fn is_terminal(s: &Self::State) -> bool;
    fn allowed_actions(s: &Self::State) -> Vec<Self::Action>;
    /// Play randomly until end of game, and return the 'return'
    /// The return is the sum of all future rewards, discounted by the discount factor
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

/// Run one step of the MCTS algorithm
/// The algorithm is:
/// 1. Select. Go down the game tree until you find a leaf node. I.e. a node that has not been visited yet.
///    The selection process is by taking the 'best' child at each node, where 'best' is defined by the UCB1 formula (or some other tree planning algo)
/// 2. Expand. If the node is new, expand into all its children. This step is kind of funny, because if you don't keep track of all non-taken actions, it is a noop.
/// 3. Rollout. From a new state, do a random rollout until the end of the game, and return the return.
/// 4. Backup. All the states visited in the selection process are updated with the return of the rollout. Apply discounting if needed.
///
/// N.B. You may accumulate return at every step in the tree.
/// The "Reward" is called G and is the total reward over all future steps.
pub(crate) fn mcts_step<M: Mdp>(
    state: &M::State,
    c: f64,
    qmap: &mut QMap<M::State, M::Action>,
    rng: &mut StdRng,
) -> f64 {
    if M::is_terminal(state) {
        return 0.0;
    }
    let best_action = best_action::<M>(state, c, qmap, rng);
    let (new_state, reward) = M::act(state.clone(), &best_action);
    let n_visits_to_new = qmap.n_state_visits(&new_state);
    // dbg!(state,&new_state,n_visits_to_new);
    let g_return = if n_visits_to_new == 0.0 {
        qmap.increment_state_visits(&new_state);
        reward + M::rollout(new_state, rng) * M::DISCOUNT_FACTOR
    } else {
        reward + mcts_step::<M>(&new_state, c, qmap, rng) * M::DISCOUNT_FACTOR
    };

    // Update the Q-function
    qmap.add_to_state_action_data(state,&best_action, g_return);

    g_return
}

#[derive(Serialize, Deserialize)]
pub(crate) struct QMap<S, A>
where
    S: Hash + Eq,
    A: Hash + Eq,
{
    /// map a state-action pair to a tuple of the total regret obtained, and the number of visits to that state
    /// uses two nested hashmaps. because of the access patterns, this seems more efficient.
    state_action_value: HashMap<S, HashMap<A, (f64, f64)>>,
    /// at 'expansion' we observe a state, but we don't know the value of the actions from that state, since
    /// we only did rollout from that state. This map keeps track of the number of visits to each state
    /// it should always contain one more than if you sum over the second element of the state_action_value
    /// for each action in the same state
    state_visits: HashMap<S, f64>,
}

impl<S, A> QMap<S, A>
where
    S: Hash + Eq + Clone,
    A: Hash + Eq + Clone,
{
    pub fn new() -> Self {
        QMap {
            state_action_value: HashMap::new(),
            state_visits: HashMap::new(),
        }
    }
    /// Peel off the outer layer in the hashmap stack
    pub fn get(&self, s: &S) -> Option<&HashMap<A, (f64, f64)>> {
        self.state_action_value.get(s)
    }
    /// get a two-layer access mutably in the stack
    pub fn get_mut(&mut self, s: &S, a: &A) -> Option<&mut (f64, f64)> {
        self.state_action_value.get_mut(s).map(|m| m.get_mut(a)).flatten()
    }
    pub fn add_to_state_action_data(&mut self, s: &S, a: &A,g_return: f64) {
        self.increment_state_visits(s);
        if let Some((w, v)) = self.get_mut(s,a) {
            *w += g_return;
            *v += 1.0;
        } else {
            if !self.state_action_value.contains_key(s) {
                self.state_action_value.insert(s.clone(), HashMap::new());
            }
            self.state_action_value.get_mut(s).unwrap().insert(a.clone(), (g_return, 1.0));
        }
    }
    pub fn n_state_visits(&self, state: &S) -> f64 {
        *self.state_visits.get(state).unwrap_or(&0.0)
    }
    pub fn increment_state_visits(&mut self, state: &S) {
        if let Some(v) = self.state_visits.get_mut(state) {
            *v += 1.0;
        } else {
            self.state_visits.insert((*state).clone(), 1.0);
        }
    }
}

pub(crate) fn best_action<M: Mdp>(
    state: &M::State,
    c: f64,
    qmap: &QMap<M::State, M::Action>,
    rng: &mut StdRng,
) -> M::Action {
    let allowed_actions = M::allowed_actions(state);
    let t = qmap.n_state_visits(state);
    let best_action = if let Some(m) = qmap.get(&state) {
        allowed_actions
        .into_iter()
        .map(|action| {
            let (w, v) = m
                .get(&action)
                .unwrap_or(&(0.0, 0.0));
            (action, ucb(c, *w, *v, t))
        })
        .fold((-f64::INFINITY, vec![]),|(mut record, mut actions),(action,ucb)| {
            if ucb == record {
                actions.push(action)
            } else if ucb > record {
                record = ucb;
                actions = vec![action];
            }
            (record, actions)
        })
        .1
        .into_iter()
        .choose(rng)
        .expect("There must be at least one action")
    } else {
        allowed_actions.into_iter().choose(rng).expect("There must be at least one action")
    };
    best_action
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

    #[derive(Debug, Clone, PartialEq, Hash, Eq, Serialize, Deserialize)]
    struct CountGameState(Vec<i8>);
    #[derive(Clone, Debug, PartialEq, Copy, Hash, Eq, Ord, PartialOrd, Serialize, Deserialize)]
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
        let mut qmap = QMap::new();
        let mut rng = StdRng::from_entropy();
        let c = 0.75;
        mcts_step::<CountGameMDP>(&root, c, &mut qmap, &mut rng);
        mcts_step::<CountGameMDP>(&root, c, &mut qmap, &mut rng);
        // The root state should have been visited twice
        assert!(qmap.n_state_visits(&root) > 0.0);
        assert_eq!(qmap.n_state_visits(&root), 2.0);
        // check that there are two actions in the qmap associated with the root state
        let visits = qmap
            .state_action_value
            .get(&root)
            .unwrap()
            .into_iter()
            .map(|(_, (_, v))| v)
            .collect::<Vec<_>>();
        assert_eq!(visits.len(), 2);
        assert_eq!(*visits[0], 1.0);
        assert_eq!(*visits[1], 1.0);
    }

    // If I run the game many times, Have I identified the best move?
    #[test]
    fn test_mcts() {
        let root: CountGameState = CountGameState(vec![]);
        let mut qmap = QMap::new();
        let mut rng = StdRng::from_entropy();
        let c = 0.75;
        for _ in 0..10000 {
            mcts_step::<CountGameMDP>(&root, c, &mut qmap, &mut rng);
        }
        let best_move = best_action::<CountGameMDP>(&root, c, &qmap, &mut rng);
        assert_eq!(
            best_move,
            CountGameAction::Add,
            "Stochastic test that might fail sometimes"
        );
    }
}

pub struct MctsAi<T: Mdp> {
    qmap: QMap<T::State, T::Action>,
    rng: StdRng,
    c: f64,
    steps_taken: u32,
    /// The file into which we save any data that helps this AI across runs
    mem_path: Option<String>,
}

impl<M: Mdp> Drop for MctsAi<M> {
    fn drop(&mut self) {
        info!("In my lifetime, I took {} moves", self.steps_taken);
        if let Some(ref mem_path) = self.mem_path {
            if let Ok(mut fd) = std::fs::File::create(mem_path) {
                let mut bytes = bitcode::serialize(&self.qmap).unwrap();
                match fd.write_all(&mut bytes) {
                    Ok(_) => {}
                    Err(e) => {
                        panic!("Failed to serialize the qmap: {}", e);
                    }
                }
            } else {
                panic!("Failed to open the file for serializing the qmap.");
            }
        }
    }
}

impl<T: Mdp> MctsAi<T> {
    /// seed is for the RNG, c is the exploration constant in the UCB1 formula
    pub fn new(seed: u64, c: f64, mem_path: Option<String>) -> Self {
        let mut qmap: QMap<T::State, T::Action> = QMap::<T::State, T::Action>::new();
        if let Some(ref mem_path) = mem_path {
            if let Ok(mut fd) = std::fs::File::open(mem_path) {
                let mut bytes = vec!();
                let _ = fd.read_to_end(&mut bytes);
                if let Ok(a) = bitcode::deserialize(&bytes) {
                    qmap = a;
                }
            }
        }
        MctsAi {
            qmap,
            rng: StdRng::seed_from_u64(seed),
            c,
            steps_taken: 0,
            mem_path,
        }
    }
}

impl<T, B> BlitzPlayer<B> for MctsAi<T>
where
    T: Mdp<Action = B::Coordinate, State = B>,
    B: Board,
{
    fn blitz(&mut self, b: &B, _time_remaining: std::time::Duration) -> <B as Board>::Coordinate {
        let t0 = std::time::Instant::now();
        let mut n_steps = 0;

        loop {
            mcts_step::<T>(b, self.c, &mut self.qmap, &mut self.rng);
            n_steps += 1;
            let duration_per_step = t0.elapsed() / n_steps;
            if t0.elapsed() + duration_per_step + Duration::from_millis(1) > _time_remaining / 8 {
                break;
            }
        }
        // dbg!(n_steps);
        self.steps_taken += n_steps;
        best_action::<T>(b, self.c, &self.qmap, &mut self.rng)
    }
}

impl<T, B> Player<B> for MctsAi<T>
where
    T: Mdp<Action = B::Coordinate, State = B>,
    B: Board,
{
    fn play(&mut self, b: &B) -> B::Coordinate {
        for _ in 0..10000 {
            mcts_step::<T>(b, self.c, &mut self.qmap, &mut self.rng);
            self.steps_taken += 1;
        }
        let a = best_action::<T>(b, self.c, &self.qmap, &mut self.rng);
        a
    }
}

impl<B: Board> Mdp for B
where
    B::Coordinate: Ord + Hash + Debug + for<'de> serde::Deserialize<'de> + Serialize,
    B: Hash + Eq + Clone + Debug + for<'de> serde::Deserialize<'de> + Serialize,
{
    type Action = B::Coordinate;

    type State = B;

    const DISCOUNT_FACTOR: f64 = -0.999;

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
