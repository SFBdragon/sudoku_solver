## Sudoku Solver

A relatively simple sudoku solver implementation in Rust.

* Takes a 81 character string as a program argument of digits from zero to nine, and outputs the solution in the same format.
* If a solution is found, it is returned in the same format. If none could be found, this is reported.
* Has been tested with the 'most difficult' puzzles found, so it should be fairly robust, and does so in 200-300 microseconds on my machine.
* The lib component can be used seperately, and is `no_std` compatible (`alloc` is required).
