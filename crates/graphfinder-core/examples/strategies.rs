//! Compare the Phase-2 strategies (the ones with their own driver) against
//! plain BFS on a single maze. Shows how iterative-deepening and bidirectional
//! search trade memory for re-expansion, and how beam search trades optimality
//! for a tiny, bounded frontier.
//!
//! ```text
//! cargo run --example strategies -p graphfinder-core
//! ```

use graphfinder_core::domains::{Maze, SAMPLE_WALL};
use graphfinder_core::heuristic::{Manhattan, Zero};
use graphfinder_core::{
    beam_search, bidirectional, ida_star, iddfs, search, Algorithm, Cell, SearchResult,
};

fn main() {
    let m = Maze::from_ascii(SAMPLE_WALL);
    let optimal = search(&m.grid, m.start, m.goal, Algorithm::bfs(), &Zero, false).cost;

    println!(
        "Maze {}x{} | optimal cost = {optimal}\n",
        m.grid.width(),
        m.grid.height()
    );
    println!(
        "{:<16} {:>6} {:>9} {:>9} {:>9}",
        "strategy", "cost", "optimal?", "expanded", "frontier"
    );
    println!("{}", "-".repeat(54));

    report(
        "BFS",
        search(&m.grid, m.start, m.goal, Algorithm::bfs(), &Zero, false),
        optimal,
    );
    report(
        "IDDFS",
        iddfs(&m.grid, m.start, m.goal, 100, false),
        optimal,
    );
    report(
        "IDA*",
        ida_star(&m.grid, m.start, m.goal, Manhattan, false),
        optimal,
    );
    report(
        "Bidirectional",
        bidirectional(&m.grid, m.start, m.goal, false),
        optimal,
    );
    report(
        "Beam (w=4)",
        beam_search(&m.grid, m.start, m.goal, Manhattan, 4, false),
        optimal,
    );
}

fn report(name: &str, r: SearchResult<Cell>, optimal: f64) {
    let (cost, opt) = match r.path {
        Some(_) => (
            format!("{}", r.cost),
            if (r.cost - optimal).abs() < 1e-9 {
                "yes"
            } else {
                "no"
            },
        ),
        None => ("—".to_string(), "no path"),
    };
    println!(
        "{name:<16} {cost:>6} {opt:>9} {:>9} {:>9}",
        r.nodes_expanded, r.max_frontier_size
    );
}
