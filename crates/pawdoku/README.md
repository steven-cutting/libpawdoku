# pawdoku

`pawdoku` is the classic-sudoku engine behind Pawdoku, a Biscuit Games game: a solver, a
catalogue of human solving techniques, a difficulty rating and hints. It is built one
specified module at a time, and so far holds the randomness boundary the others will use
and, in `sudoku`, the grid's two figures and the rules' value types. It is `no_std` with
`alloc`, so it has no clock, threads, filesystem or network, and runs the same in a
browser, in Python and on the command line.

Randomness is its one effect. The caller supplies a seeded stream of draws through the
`random` module's trait, and the same seed and generator always give the same draws.

Every rule the engine follows is stated first in the Allium specifications under
`docs/specs/` in the repository, and the code is built from them.

Licensed under Apache-2.0.
