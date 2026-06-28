//! Solve the three implicit puzzles with the *same* search loop that runs on
//! mazes and graphs — only the `Graph` impl changes.
//!
//! ```text
//! cargo run --example puzzles -p graphfinder-core
//! ```

use graphfinder_core::puzzles::{
    Hanoi, HanoiMisplaced, LadderHamming, NPuzzle, PuzzleManhattan, WordLadder,
};
use graphfinder_core::{search, Algorithm};

fn main() {
    // --- 8-puzzle: A* with Manhattan ---------------------------------------
    let p = NPuzzle::new(3);
    let goal = p.goal();
    let start = vec![1, 2, 3, 4, 0, 6, 7, 5, 8];
    let r = search(
        &p,
        start,
        goal,
        Algorithm::astar(),
        &PuzzleManhattan { width: 3 },
        false,
    );
    println!(
        "8-puzzle  | A* (Manhattan): {} moves, {} states expanded",
        r.cost, r.nodes_expanded
    );

    // --- Towers of Hanoi: optimal is 2^n - 1 -------------------------------
    for disks in [3, 5, 7] {
        let h = Hanoi::new(disks);
        let r = search(
            &h,
            h.start(),
            h.goal(),
            Algorithm::astar(),
            &HanoiMisplaced,
            false,
        );
        println!(
            "Hanoi({disks})  | {} moves (optimal {}), {} expanded",
            r.cost,
            2i64.pow(disks as u32) - 1,
            r.nodes_expanded
        );
    }

    // --- Word ladder -------------------------------------------------------
    let words = ["hit", "hot", "dot", "dog", "cog", "lot", "log"];
    let ladder = WordLadder::new(words);
    let r = search(
        &ladder,
        "hit".to_string(),
        "cog".to_string(),
        Algorithm::astar(),
        &LadderHamming,
        true,
    );
    println!(
        "word ladder | hit → cog in {} steps: {:?}",
        r.cost,
        r.path.unwrap_or_default()
    );
}
