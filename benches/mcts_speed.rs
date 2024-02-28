use criterion::{black_box, criterion_group, criterion_main, Criterion};
use xoxo::core::Player;
use xoxo::player::MctsAi;
use xoxo::game::connect_four::C4Board;

fn mcts_move() {
    let mut ai: MctsAi<C4Board> = MctsAi::new(123, 2.0, None);
    let board = C4Board::default();
    ai.play(&board);
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("mcts-c4-play");
    group.sample_size(20);
    group.bench_function("mcts-c4-play", |b| b.iter(|| black_box(mcts_move())));
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);