TicTacToe and UltimateTicTacToe and Connect Four
================================================

Thre are two binaries:
- tui, which is a terminal user interface to play a single game. You can play as a human here.
- arena, which plays a single bot-vs-bot game and outputs the result to a csv file for later analysis

The minimax and alphabeta AI players for normal TicTacToe are good - they will play perfectly. For connect4 they have a weak heuristic (non-admissible?) so they can play wierdly. In UltimateTicTacToe they can struggle. The branching factor is large and the search horizon must be severely limited for minimax. The pruning AI can play quite okay.

The MCTS bots needs to have enough data to perform well. So I keep all data on previous MCTS runs in a file on disk. The deserialization takes a but of time as the file grows, and after ~10 games it starts to be prohibitive. That is not enough for beating the AB6 player. Funnily enough, these bots play better as player2. I suspect this is because the severe limitation on the serach space for their first move. The learning should be ca a factor 81 faster, since the AB6 opponent always plays the same first move, and the part of the game tree explored is much smaller.
To speed this up, I think some custom hashing that is simpler/faster is needed. Or possibly some serialization that stores hashes to disk.


### Profiling

Here is a command on how I profile with flamegraph:
```bash
cargo build --release
PERF=/usr/lib/linux-tools/5.4.0-171-generic/perf flamegraph -- ./target/release/arena c4 run --player2 minimax4 --player1 mcts3
```