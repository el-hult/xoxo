set -e
cargo build --release --bin arena
rm -f score.csv
rm -f *.bincode
players=("random" "mcts1" "mcts2" "mcts3" "minimax4" "ab6");
players=("mcts1" "mcts2" "mcts3" "ab4" "ab6");
# players=("mcts1" "mcts3" "ab6");
# players=("mcts1" "mcts3");
for k in {1..1000}; do
    i=$((RANDOM % ${#players[@]}))
    j=$((RANDOM % ${#players[@]}))
    while [ $i -eq $j ]; do
        j=$((RANDOM % ${#players[@]}))
    done
    ./target/release/arena --outfile score.csv c4 run  --player2 ${players[$j]} --player1 ${players[$i]} ;
    ./target/release/arena --outfile score.csv c4 report
    # ls -lh *.bincode
done;