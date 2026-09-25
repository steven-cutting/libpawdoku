# pawdoku

`pawdoku` is the core crate of this workspace: a `no_std` classic-sudoku engine that
solves puzzles, rates their difficulty, models how a human would solve them, and
explains the next step as a hint. It takes randomness through a trait of its own and
reaches for no clock, thread or filesystem, so the same code runs in a browser, in
Python and on the command line. This page is a placeholder that ticket T10 replaces
with the crate's published description and a first example.
