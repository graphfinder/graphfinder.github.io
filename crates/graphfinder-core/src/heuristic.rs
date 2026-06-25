//! Heuristics `h(n)`: estimates of remaining cost to the goal.
//!
//! [`Zero`] works for any domain and makes informed algorithms behave as their
//! uninformed counterparts. The geometric heuristics are defined only for grid
//! [`Cell`]s, where they have a clear meaning.

use crate::graph::Cell;
use crate::traits::Heuristic;

/// `h(n) = 0` for every node. Admissible (never overestimates) trivially.
/// Turns A* into UCS and Greedy into a blind expansion. The uninformed default.
#[derive(Debug, Default, Clone, Copy)]
pub struct Zero;

impl<N> Heuristic<N> for Zero {
    fn estimate(&self, _node: &N, _goal: &N) -> f64 {
        0.0
    }
}

/// Manhattan distance `|Δrow| + |Δcol|`. Admissible on a 4-connected unit grid;
/// the natural heuristic for orthogonal-only movement.
#[derive(Debug, Default, Clone, Copy)]
pub struct Manhattan;

impl Heuristic<Cell> for Manhattan {
    fn estimate(&self, node: &Cell, goal: &Cell) -> f64 {
        ((node.row - goal.row).abs() + (node.col - goal.col).abs()) as f64
    }
}

/// Straight-line (Euclidean) distance. Admissible whenever diagonal-ish travel
/// is allowed; underestimates on a 4-connected grid (still admissible there).
#[derive(Debug, Default, Clone, Copy)]
pub struct Euclidean;

impl Heuristic<Cell> for Euclidean {
    fn estimate(&self, node: &Cell, goal: &Cell) -> f64 {
        let dr = (node.row - goal.row) as f64;
        let dc = (node.col - goal.col) as f64;
        (dr * dr + dc * dc).sqrt()
    }
}

/// Octile distance: the exact shortest path on an 8-connected grid with
/// diagonal cost `√2`. Use this with `GridGraph::with_diagonal(true)`.
#[derive(Debug, Default, Clone, Copy)]
pub struct Octile;

impl Heuristic<Cell> for Octile {
    fn estimate(&self, node: &Cell, goal: &Cell) -> f64 {
        let dx = (node.col - goal.col).abs() as f64;
        let dy = (node.row - goal.row).abs() as f64;
        let (long, short) = if dx > dy { (dx, dy) } else { (dy, dx) };
        (long - short) + std::f64::consts::SQRT_2 * short
    }
}
