TicTacToe and UltimateTicTacToe and Connect Four
================================================

All games can be played in the terminal between two console players, or with AI players.

The minimax and alphabeta AI players for normal TicTacToe are good - they will play perfectly.

For UltimateTicTacToe, the AI players are not as good. They can only plan 5-10 moves ahead, (depending on pruning) since the game tree is too large. And the heuristic is not great.

Connect Four is strange, it has a constant medium large branch factor, so the MCTS AI is not great.

Thre are two binaries:
- tui, which is a terminal user interface to play a single game and see progress etc
- arena, which plays a single bot-vs-bot game and outputs the result to a csv file for later analysis
