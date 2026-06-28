//! Tests for the implicit puzzle domains: each asserts its defining property —
//! optimal cost from A\*/BFS, solvability parity, and known optimal lengths.

use graphfinder_core::puzzles::{
    Hanoi, HanoiMisplaced, LadderHamming, NPuzzle, PuzzleManhattan, PuzzleMisplaced, WordLadder,
};
use graphfinder_core::{search, Algorithm, Zero};

// --- N-puzzle --------------------------------------------------------------

/// On the 8-puzzle, A\* with Manhattan returns the same optimal cost as BFS, and
/// (admissible heuristic) expands no more nodes than UCS.
#[test]
fn npuzzle_astar_optimal_and_efficient() {
    let p = NPuzzle::new(3);
    let goal = p.goal();
    // A scramble three slides from the goal.
    let start = vec![1, 2, 3, 4, 0, 6, 7, 5, 8];
    assert!(p.is_solvable(&start, &goal));

    let bfs = search(
        &p,
        start.clone(),
        goal.clone(),
        Algorithm::bfs(),
        &Zero,
        false,
    );
    let ucs = search(
        &p,
        start.clone(),
        goal.clone(),
        Algorithm::ucs(),
        &Zero,
        false,
    );
    let astar = search(
        &p,
        start.clone(),
        goal.clone(),
        Algorithm::astar(),
        &PuzzleManhattan { width: 3 },
        false,
    );
    assert!(astar.found());
    assert_eq!(astar.cost, bfs.cost, "A* must match BFS optimal cost");
    assert!(
        astar.nodes_expanded <= ucs.nodes_expanded,
        "admissible A* expands ≤ UCS ({} vs {})",
        astar.nodes_expanded,
        ucs.nodes_expanded
    );
}

/// Manhattan is at least as strong as misplaced-tiles: it never expands more.
#[test]
fn npuzzle_manhattan_beats_misplaced() {
    let p = NPuzzle::new(3);
    let goal = p.goal();
    let start = vec![1, 2, 3, 4, 5, 6, 0, 7, 8]; // two slides away
    let manh = search(
        &p,
        start.clone(),
        goal.clone(),
        Algorithm::astar(),
        &PuzzleManhattan { width: 3 },
        false,
    );
    let misp = search(
        &p,
        start.clone(),
        goal.clone(),
        Algorithm::astar(),
        &PuzzleMisplaced,
        false,
    );
    assert_eq!(manh.cost, misp.cost); // both optimal
    assert!(manh.nodes_expanded <= misp.nodes_expanded);
}

/// Half of all arrangements are unreachable; the parity test flags them.
#[test]
fn npuzzle_solvability_parity() {
    let p = NPuzzle::new(3);
    let goal = p.goal();
    // Swapping two non-blank tiles flips parity → unsolvable.
    let unsolvable = vec![2, 1, 3, 4, 5, 6, 7, 8, 0];
    assert!(!p.is_solvable(&unsolvable, &goal));
    assert!(p.is_solvable(&goal, &goal));
}

// --- Towers of Hanoi -------------------------------------------------------

/// The classic 3-peg puzzle has a known optimal of `2^n − 1` moves.
#[test]
fn hanoi_optimal_is_two_pow_n_minus_one() {
    for disks in 1..=6 {
        let h = Hanoi::new(disks);
        let r = search(
            &h,
            h.start(),
            h.goal(),
            Algorithm::astar(),
            &HanoiMisplaced,
            false,
        );
        assert!(r.found(), "Hanoi {disks} should be solvable");
        assert_eq!(
            r.cost,
            (2f64.powi(disks as i32)) - 1.0,
            "Hanoi {disks} optimal moves"
        );
    }
}

/// A* with the admissible misplaced heuristic agrees with BFS on cost.
#[test]
fn hanoi_astar_matches_bfs() {
    let h = Hanoi::new(4);
    let bfs = search(&h, h.start(), h.goal(), Algorithm::bfs(), &Zero, false);
    let astar = search(
        &h,
        h.start(),
        h.goal(),
        Algorithm::astar(),
        &HanoiMisplaced,
        false,
    );
    assert_eq!(astar.cost, bfs.cost);
    assert_eq!(astar.cost, 15.0);
}

// --- Word ladder -----------------------------------------------------------

/// A small ladder with a known shortest transformation.
#[test]
fn wordladder_finds_shortest() {
    let words = ["hit", "hot", "dot", "dog", "cog", "lot", "log"];
    let ladder = WordLadder::new(words);
    let r = search(
        &ladder,
        "hit".to_string(),
        "cog".to_string(),
        Algorithm::astar(),
        &LadderHamming,
        false,
    );
    assert!(r.found());
    // hit → hot → dot → dog → cog  (4 steps)
    assert_eq!(r.cost, 4.0);
    let path = r.path.unwrap();
    assert_eq!(path.first().unwrap(), "hit");
    assert_eq!(path.last().unwrap(), "cog");
}

/// No ladder exists when the target is isolated in the dictionary.
#[test]
fn wordladder_unreachable() {
    let ladder = WordLadder::new(["cat", "cot", "dog"]);
    let r = search(
        &ladder,
        "cat".to_string(),
        "dog".to_string(),
        Algorithm::bfs(),
        &Zero,
        false,
    );
    assert!(!r.found());
}
