use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::Rng;
use xoxo::core::Player;
use xoxo::game::{connect_four::C4Board, run_blitz_game};
use xoxo::player::{MctsAi, RandomAi};

fn mcts_move() {
    let mut ai: MctsAi<C4Board> = MctsAi::new(123, 2.0, None);
    ai.set_play_steps(2000);
    let board = C4Board::default();
    ai.play(&board);
}

fn mcts_io() {
    const FILENAME1: &str = "bench.mcts.c4.O.data";
    if std::path::Path::new(FILENAME1).exists() {
        std::fs::remove_file(FILENAME1).unwrap();
    }
    let mut rng = rand::thread_rng();

    for _k in 0..10 {
        let ai1: MctsAi<C4Board> = MctsAi::new(rng.gen(), 2.0, Some(FILENAME1.into()));
        let ai2 = RandomAi::new(rng.gen());
        run_blitz_game::<C4Board>(
            Box::new(ai1),
            Box::new(ai2),
            std::time::Duration::from_secs_f64(0.1),
        );
    }

    std::fs::remove_file(FILENAME1).unwrap();
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("mcts-c4");
    group.measurement_time(Duration::from_secs(15));
    group.sampling_mode(criterion::SamplingMode::Flat);
    group.sample_size(10);
    group.bench_function("mcts-c4-io", |b| b.iter(|| {
        mcts_io();
        black_box(())
    }));
    group.sample_size(300);
    group.bench_function("mcts-c4-play", |b| b.iter(|| {
        mcts_move();
        black_box(())
    }));
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
