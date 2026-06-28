//! The Towers of Hanoi as an implicit graph.

use crate::traits::{Graph, Heuristic};

/// Towers of Hanoi with `disks` disks over `pegs` pegs (classically `3`). A
/// **state** is a `Vec<u8>` of length `disks`: `state[d]` is the peg holding
/// disk `d`, where disk `0` is the smallest. A move takes the smallest disk off
/// one peg and onto another whose top disk is larger (cost `1`).
#[derive(Clone, Debug)]
pub struct Hanoi {
    /// Number of disks.
    pub disks: usize,
    /// Number of pegs (`3` for the classic puzzle).
    pub pegs: usize,
}

impl Hanoi {
    /// The classic 3-peg puzzle with `disks` disks.
    pub fn new(disks: usize) -> Self {
        Self { disks, pegs: 3 }
    }

    /// A puzzle with a custom number of pegs.
    pub fn with_pegs(disks: usize, pegs: usize) -> Self {
        Self { disks, pegs }
    }

    /// Start state: every disk stacked on peg `0`.
    pub fn start(&self) -> Vec<u8> {
        vec![0u8; self.disks]
    }

    /// Goal state: every disk stacked on the last peg.
    pub fn goal(&self) -> Vec<u8> {
        vec![(self.pegs.saturating_sub(1)) as u8; self.disks]
    }
}

impl Graph for Hanoi {
    type Node = Vec<u8>;

    fn neighbors(&self, state: &Vec<u8>) -> Vec<(Vec<u8>, f64)> {
        // The top of a peg is its smallest disk. Disks iterate in ascending size,
        // so the first one found on a peg is its top.
        let mut top: Vec<Option<usize>> = vec![None; self.pegs];
        for (disk, &peg) in state.iter().enumerate() {
            let p = peg as usize;
            if p < self.pegs && top[p].is_none() {
                top[p] = Some(disk);
            }
        }
        let mut out = Vec::new();
        for (from, &from_top) in top.iter().enumerate() {
            let Some(disk) = from_top else { continue };
            for (to, &dest_top) in top.iter().enumerate() {
                if to == from {
                    continue;
                }
                // Legal iff the destination is empty or its top disk is larger.
                let legal = match dest_top {
                    None => true,
                    Some(other) => other > disk,
                };
                if legal {
                    let mut next = state.clone();
                    next[disk] = to as u8;
                    out.push((next, 1.0));
                }
            }
        }
        out
    }
}

/// Number of disks not yet on their goal peg. Admissible and consistent (every
/// misplaced disk must move at least once, and a single move changes the count
/// by at most one).
#[derive(Clone, Copy, Debug, Default)]
pub struct HanoiMisplaced;

impl Heuristic<Vec<u8>> for HanoiMisplaced {
    fn estimate(&self, node: &Vec<u8>, goal: &Vec<u8>) -> f64 {
        node.iter().zip(goal).filter(|(a, b)| a != b).count() as f64
    }
}
