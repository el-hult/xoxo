set -e
cargo build --release --bin arena
rm -f score.csv
rm -f *.data
# players=("random" "ab4" "ab6" "minimax4" "mcts1" "mcts2" "mcts3");
players=("ab6" "mcts1" "mcts2" "mcts3");
game=c4
for k in {1..10}; do
for p1 in ${players[@]}; do
for p2 in ${players[@]}; do
    start=$(date +%s.%N)
    ./target/release/arena --outfile score.csv $game run  --player2 $p2 --player1 $p1
    end=$(date +%s.%N)
    elapsed=$(echo "$end - $start" | bc)
    ls -lh *.data || true
    echo "Running $game $p1 vs $p2 round $k took $elapsed seconds"
    ./target/release/arena --outfile score.csv $game report
done;
done;
done;