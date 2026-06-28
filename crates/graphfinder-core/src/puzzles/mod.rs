//! **Implicit state-space puzzles** — classic teaching domains where the graph
//! is never materialized: each state generates its neighbours on demand through
//! the same [`Graph`] trait the maze and CSR domains use, so *every* algorithm
//! ([`search`], the [`strategies`] drivers) works on them unchanged.
//!
//! - [`NPuzzle`] — the sliding-tile puzzle (8-puzzle, 15-puzzle, …).
//! - [`Hanoi`] — the Towers of Hanoi.
//! - [`WordLadder`] — transform one word into another, one letter at a time.
//!
//! Each ships an **admissible** heuristic so A\* / IDA\* stay optimal:
//! [`PuzzleManhattan`] / [`PuzzleMisplaced`], [`HanoiMisplaced`] and
//! [`LadderHamming`].
//!
//! [`Graph`]: crate::Graph
//! [`search`]: crate::search
//! [`strategies`]: crate::strategies

mod hanoi;
mod npuzzle;
mod wordladder;

pub use hanoi::{Hanoi, HanoiMisplaced};
pub use npuzzle::{NPuzzle, PuzzleManhattan, PuzzleMisplaced};
pub use wordladder::{LadderHamming, WordLadder};
