


use rand::{rngs::StdRng, SeedableRng};


use crate::core::{Board, Player, PlayerMark};
pub struct RandomAi<R> {
    rng: R,
    pub name: String,
}

impl<Rng, B> Player<B> for RandomAi<Rng>
where
    Rng: rand::Rng,
    B: Board,
{
    fn play(&mut self, b: &B) -> B::Coordinate {
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
