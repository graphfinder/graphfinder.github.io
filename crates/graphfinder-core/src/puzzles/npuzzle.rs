//! The sliding-tile puzzle (8-puzzle on 3×3, 15-puzzle on 4×4, …).

use crate::traits::{Graph, Heuristic};

/// A sliding-tile puzzle on a `width × width` board. A **state** is the tiles in
/// row-major order as a `Vec<u8>`, with `0` marking the blank. The only move is
/// to slide a tile orthogonally into the blank (cost `1`).
#[derive(Clone, Debug)]
pub struct NPuzzle {
    /// Side length of the board (`3` → 8-puzzle, `4` → 15-puzzle).
    pub width: usize,
}

impl NPuzzle {
    /// A puzzle on a `width × width` board.
    pub fn new(width: usize) -> Self {
        Self { width }
    }

    /// The canonical solved state `1, 2, …, n−1, 0` (blank in the last cell).
    pub fn goal(&self) -> Vec<u8> {
        let n = self.width * self.width;
        let mut g: Vec<u8> = (1..n as u8).collect();
        g.push(0);
        g
    }

    /// Whether `state` can be transformed into `goal` by legal slides. A sliding
    /// puzzle is solvable iff the permutation parity equals the parity of the
    /// blank's Manhattan displacement — so half of all arrangements are
    /// unreachable, and searching them would never terminate with a path.
    pub fn is_solvable(&self, state: &[u8], goal: &[u8]) -> bool {
        let n = state.len();
        if goal.len() != n {
            return false;
        }
        // goal position of every tile value.
        let mut goal_pos = vec![0usize; n];
        for (i, &v) in goal.iter().enumerate() {
            goal_pos[v as usize] = i;
        }
        // Permutation that maps the current layout onto the goal; parity via
        // cycle decomposition (blank included).
        let perm: Vec<usize> = state.iter().map(|&v| goal_pos[v as usize]).collect();
        let mut seen = vec![false; n];
        let mut transpositions = 0usize;
        for i in 0..n {
            if seen[i] {
                continue;
            }
            let mut j = i;
            let mut len = 0usize;
            while !seen[j] {
                seen[j] = true;
                j = perm[j];
                len += 1;
            }
            transpositions += len - 1;
        }
        let perm_parity = transpositions % 2;
        // Parity of the blank's Manhattan distance between the two states.
        let w = self.width;
        let bs = state.iter().position(|&x| x == 0).unwrap_or(0);
        let bg = goal.iter().position(|&x| x == 0).unwrap_or(0);
        let dist = (bs / w).abs_diff(bg / w) + (bs % w).abs_diff(bg % w);
        perm_parity == dist % 2
    }
}

impl Graph for NPuzzle {
    type Node = Vec<u8>;

    fn neighbors(&self, state: &Vec<u8>) -> Vec<(Vec<u8>, f64)> {
        let w = self.width as i32;
        let blank = state.iter().position(|&x| x == 0).unwrap_or(0);
        let (r, c) = ((blank / self.width) as i32, (blank % self.width) as i32);
        let mut out = Vec::with_capacity(4);
        for (dr, dc) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
            let (nr, nc) = (r + dr, c + dc);
            if nr >= 0 && nr < w && nc >= 0 && nc < w {
                let ni = (nr as usize) * self.width + (nc as usize);
                let mut next = state.clone();
                next.swap(blank, ni);
                out.push((next, 1.0));
            }
        }
        out
    }
}

/// Sum of the Manhattan distances of every tile from its goal cell (the blank
/// excluded). The standard admissible, consistent heuristic for the puzzle.
#[derive(Clone, Copy, Debug)]
pub struct PuzzleManhattan {
    /// Board side length (needed to map an index to `(row, col)`).
    pub width: usize,
}

impl Heuristic<Vec<u8>> for PuzzleManhattan {
    fn estimate(&self, node: &Vec<u8>, goal: &Vec<u8>) -> f64 {
        let w = self.width;
        let mut total = 0usize;
        for (i, &v) in node.iter().enumerate() {
            if v == 0 {
                continue;
            }
            let gi = goal.iter().position(|&x| x == v).unwrap_or(i);
            total += (i / w).abs_diff(gi / w) + (i % w).abs_diff(gi % w);
        }
        total as f64
    }
}

/// Number of misplaced tiles (Hamming distance, blank excluded). Admissible but
/// weaker than [`PuzzleManhattan`] — useful to *show* why a better heuristic
/// expands fewer nodes.
#[derive(Clone, Copy, Debug, Default)]
pub struct PuzzleMisplaced;

impl Heuristic<Vec<u8>> for PuzzleMisplaced {
    fn estimate(&self, node: &Vec<u8>, goal: &Vec<u8>) -> f64 {
        node.iter()
            .zip(goal)
            .filter(|(&a, &b)| a != 0 && a != b)
            .count() as f64
    }
}
