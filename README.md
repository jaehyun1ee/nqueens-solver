Solving the [N-Queens Puzzle](https://en.wikipedia.org/wiki/Eight_queens_puzzle).

The puzzle can be encoded into a propositional logic formula, then solved by the Rust Z3 SAT/SMT solver.

```
$ cargo run -- --count 8 --unique true
```

Outputs the number of fundamental (i.e., counting rotation/reflection as one) solutions on a 8x8 chess board.

`count` defines the width/height of the chess board and `unique` defines whether to count fundamental solutions or all solutions.

The solutions are also available at OEIS, [fundamental](https://oeis.org/A002562) and [all](https://oeis.org/A000170).
