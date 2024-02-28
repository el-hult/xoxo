rm *.data
cargo build --release --bin arena
# run 5 times to generate some data for serialization
for k in {1..10}; do
./target/release/arena c4 run --player2 minimax4 --player1 mcts2
done
# report the runtime on the last one
time ./target/release/arena c4 run --player2 minimax4 --player1 mcts2
# report how much data was generated (approx same as the number of spots explored)
ls -lh *.data
# Then generate a flame graph on yet another run
PERF=/usr/lib/linux-tools/5.4.0-171-generic/perf flamegraph -- ./target/release/arena c4 run --player2 minimax4 --player1 mcts2

# with nested hashmaps, it reports 18k - 42k MCTS steps per game.
# the timed run takes ca 1.5 seconds
# the data file takes ca 16M on disk