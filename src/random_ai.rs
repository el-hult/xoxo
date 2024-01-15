
use super::*;
use crate::core::Board;
pub struct RandomAi<Rng> {
    rng: Rng,
    pub name: String,
}

impl<Rng, G> Player<G> for RandomAi<Rng>
where
    Rng: rand::Rng,
    G: Game,
{
    fn play(&mut self, b: &G::B) -> <G::B as Board>::A {
        let moves: Vec<_> = b.valid_moves();
        let idx = self.rng.next_u32() as usize % moves.len();
        println!("Random AI `{}` plays {}", self.name, moves[idx]);
        moves[idx]
    }
}

impl RandomAi<rand::prelude::ThreadRng> {
    pub fn new(mark: PlayerMark) -> Self {
        Self {
            name: match mark {
                PlayerMark::Cross => "X".into(),
                PlayerMark::Naught => "O".into(),
            },
            rng: rand::thread_rng(),
        }
    }
}
