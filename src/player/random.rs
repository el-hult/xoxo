use rand::{rngs::StdRng, SeedableRng};

use crate::core::{BlitzPlayer, Board, Player};
pub struct RandomAi<R> {
    rng: R,
}

impl<Rng, B> Player<B> for RandomAi<Rng>
where
    Rng: rand::Rng,
    B: Board,
{
    fn play(&mut self, b: &B) -> B::Coordinate {
        let moves: Vec<_> = b.valid_moves();
        let idx = self.rng.next_u32() as usize % moves.len();
        moves[idx]
    }
}

impl<R,B> BlitzPlayer<B> for RandomAi<R>
where
    R: rand::Rng,
    B: Board,
{
    fn blitz(&mut self, b: &B, time_remaining: std::time::Duration) -> B::Coordinate {
        let _ = time_remaining; // unused
        self.play(b)
    }
}

impl RandomAi<rand::prelude::StdRng> {
    pub fn new(seed: Option<u64>) -> Self {
        Self {
            rng: match seed {
                None => StdRng::from_entropy(),
                Some(seed) => StdRng::seed_from_u64(seed),
            },
        }
    }
}
