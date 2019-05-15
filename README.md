# Sudoku Solver
A Sudoku Solver written in Rust

## Usage
Compile and run the program using cargo:

```
cargo run [csv_file]
```

`csv_file` is a path to a csv file containing the puzzles to be solved. It should have two columns, one for the puzzles and one for their solutions (the solutions are used to make sure the algorithm is correct, this column should be made optional in the future). Boards should be serialized as a string of numbers, where each number represents the value of a cell, ordered from left to right and top to bottom on the board. A `0` or a `.` can be used to indicate a blank cell for unsolved puzzles.
