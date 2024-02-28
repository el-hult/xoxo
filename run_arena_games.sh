set -e
cargo build --release --bin arena
rm -f score.csv
rm -f *.data
# players=("random" "ab4" "ab6" "minimax4" "mcts1" "mcts2" "mcts3");
players=("ab6" "mcts1" "mcts2" "mcts3");
game=uttt
for k in {1..10}; do
for p1 in ${players[@]}; do
for p2 in ${players[@]}; do
    ./target/release/arena --outfile score.csv $game run  --player2 $p2 --player1 $p1
    ls -lh *.data || true
    ./target/release/arena --outfile score.csv $game report
done;
done;
done;