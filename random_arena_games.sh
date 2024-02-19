set -e
cargo build --release --bin arena
rm -f score.csv

players=("random" "mcts1" "mcts2" "mcts3" "minimax4" "ab6");
for k in {1..1000}; do
    i=$((RANDOM % ${#players[@]}))
    j=$((RANDOM % ${#players[@]}))
    while [ $i -eq $j ]; do
        j=$((RANDOM % ${#players[@]}))
    done
    ./target/release/arena --outfile score.csv --game c4 run  --player2 ${players[$j]} --player1 ${players[$i]} ;
    ./target/release/arena --outfile score.csv --game c4 report
done;