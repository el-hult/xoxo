use clap::Parser as _;
use rand::seq::SliceRandom as _;

use crate::mcts::{best_action, mcts_step};

mod mcts;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Card {
    value: u8,
    suit: Suit,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Suit {
    Spades,
    Hearts,
    Diamonds,
    Clubs,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum BjAction {
    PlaceBet(i64),
    Hit,
    Stand,
    Surrender,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum BjStatus {
    Wagering,
    PlayerTurn,
    Ended,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// must be compatible with https://github.com/endform/blackjack_state_machine/blob/master/BlackjackStateMachine_state.png
/// but adjusted so that only the player takes actions
struct BjState {
    dealer: Vec<Card>,
    player: Vec<Card>,
    bet: i64,
    status: BjStatus,
}
fn all_cards() -> Vec<Card> {
    let mut cards = vec![];
    for value in 1..=13 {
        for suit in [Suit::Spades, Suit::Hearts, Suit::Diamonds, Suit::Clubs] {
            cards.push(Card { value, suit });
        }
    }
    cards
}

fn make_talon(state: &BjState) -> Vec<Card> {
    let mut rng = rand::thread_rng();
    let mut deck = all_cards()
        .iter()
        .filter(|c| !state.dealer.contains(c) && !state.player.contains(c))
        .cloned()
        .collect::<Vec<Card>>();
    deck.shuffle(&mut rng);
    deck
}

impl BjState {
    fn init() -> Self {
        BjState {
            dealer: vec![],
            player: vec![],
            bet: 0,
            status: BjStatus::Wagering,
        }
    }

    fn player_score(&self) -> u8 {
        Self::score_hand(&self.player)
    }
    fn dealer_score(&self) -> u8 {
        Self::score_hand(&self.dealer)
    }
    /// In blackjack, the value of the hand is the sum of the values of the cards.
    /// 10, Jack, Queen, and King are all worth 10.
    /// The Ace is worth 1 or 11.
    /// this function returns the highest score that is not over 21 (trying both 1 and 11 for each aces)
    fn score_hand(hand: &Vec<Card>) -> u8 {
        let mut score = 0;
        let mut aces = 0;
        for card in hand {
            if card.value == 1 {
                aces += 1;
                score += 11;
            } else if card.value >= 10 {
                score += 10;
            } else {
                score += card.value;
            }
        }
        while score > 21 && aces > 0 {
            score -= 10;
            aces -= 1;
        }
        score
    }

    fn act(self, action: &BjAction) -> (BjState, f64) {
        let status = self.status.clone();
        let mut s = self;
        match status {
            BjStatus::Wagering => match action {
                BjAction::PlaceBet(bet) => {
                    s.bet = *bet;
                    s.status = BjStatus::PlayerTurn;
                    let mut talon = make_talon(&s);
                    s.player.push(talon.pop().unwrap());
                    s.player.push(talon.pop().unwrap());
                    s.dealer.push(talon.pop().unwrap());
                    if s.player_score() == 21 {
                        s.status = BjStatus::Ended;
                        let r = 1.5 * s.bet as f64;
                        (s, r)
                    } else {
                        (s, 0.0)
                    }
                }
                _ => panic!("Invalid action"),
            },
            BjStatus::Ended => unreachable!(),
            BjStatus::PlayerTurn => match action {
                BjAction::Hit => {
                    let mut talon = make_talon(&s);
                    s.player.push(talon.pop().unwrap());
                    if s.player_score() > 21 {
                        s.status = BjStatus::Ended;
                        let r = -s.bet as f64;
                        (s, r)
                    } else {
                        (s, 0.0)
                    }
                }
                BjAction::Stand => {
                    let mut talon = make_talon(&s);
                    while s.dealer_score() < 17 {
                        s.dealer.push(talon.pop().unwrap());
                    }
                    if s.dealer_score() > 21 || s.dealer_score() < s.player_score() {
                        s.status = BjStatus::Ended;
                        let r = s.bet as f64;
                        (s, r)
                    } else if s.dealer_score() == s.player_score() {
                        s.status = BjStatus::Ended;
                        return (s, 0.0);
                    } else {
                        s.status = BjStatus::Ended;
                        let r = -s.bet as f64;
                        return (s, r);
                    }
                }
                BjAction::Surrender => {
                    s.status = BjStatus::Ended;
                    let r = -0.5 * s.bet as f64;
                    (s, r)
                }
                _ => panic!("Invalid action"),
            },
        }
    }
}

/// https://sharpneat.sourceforge.io/research/cart-pole/cart-pole-equations.html
struct BlackJack {}
impl mcts::Mdp for BlackJack {
    type Action = BjAction;
    type State = BjState;
    const DISCOUNT_FACTOR: f64 = 1.0;
    fn is_terminal(s: &Self::State) -> bool {
        s.status == BjStatus::Ended
    }
    fn act(s: Self::State, action: &Self::Action) -> (Self::State, f64) {
        s.act(action)
    }
    fn allowed_actions(s: &Self::State) -> Vec<Self::Action> {
        match s.status {
            BjStatus::Wagering => vec![
                BjAction::PlaceBet(1),
                BjAction::PlaceBet(10),
                BjAction::PlaceBet(100),
            ],
            BjStatus::PlayerTurn => vec![BjAction::Hit, BjAction::Stand, BjAction::Surrender],
            _ => vec![],
        }
    }
}

/// A simplified game of blackjack where the dealer has a fixed strategy.
#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The seed for the random number generator (when used)
    #[arg(long)]
    seed: Option<u64>,
}
fn main() {
    let args = Args::parse();
    dbg!(args);
    let mut total_wins = 0.0;
    for _ in 0..200 {
        let mut state = BjState::init();
        let mut qmap = std::collections::HashMap::new();
        let mut state_visit_counter = std::collections::HashMap::new();
        for _ in 0..10 {
            for _ in 0..20000 {
                mcts_step::<BlackJack>(&state, &mut state_visit_counter, &mut qmap);
            }
            let best_move = best_action::<BlackJack>(&state, &qmap, &state_visit_counter);
            let (new_state, reward) = state.act(&best_move);
            if new_state.status == BjStatus::Ended {
                println!(
                    "Game ended with reward: {:+6.1} and scores {:2} vs {:2}. Total wins: {:+6.1}",
                    reward,
                    new_state.player_score(),
                    new_state.dealer_score(),
                    total_wins
                );
                total_wins += reward;
                break;
            } else {
                state = new_state;
            }
        }
    }
}
