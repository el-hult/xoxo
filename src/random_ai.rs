
use rand::{rngs::StdRng, SeedableRng};

use super::*;
use crate::core::Board;
pub struct RandomAi<R> {
    rng: R,
    pub name: String,
}

impl<Rng, G> Player<G> for RandomAi<Rng>
where
    Rng: rand::Rng,
    G: Game,
{
    fn play(&mut self, b: &G::Board) -> G::Coordinate {
        let moves: Vec<_> = b.valid_moves();
        let idx = self.rng.next_u32() as usize % moves.len();
        println!("Random AI `{}` plays {}", self.name, moves[idx]);
        moves[idx]
    }
}

impl RandomAi<rand::prelude::StdRng> {
    pub fn new(mark: PlayerMark, seed: Option<u64>) -> Self {
        Self {
            name: match mark {
                PlayerMark::Cross => "X".into(),
                PlayerMark::Naught => "O".into(),
            },
            rng: match seed {
                None => StdRng::from_entropy(),
                Some(seed) => StdRng::seed_from_u64(seed)
            }
        }
    }
}
