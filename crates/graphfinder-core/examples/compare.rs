//! Comparison example: run every algorithm on the same maze and print a table
//! of cost / optimality / nodes-expanded / peak-frontier. This is the core
//! teaching message — same problem, same loop, very different behaviour.
//!
//! ```text
//! cargo run --example compare -p graphfinder-core
//! ```

use graphfinder_core::domains::{Maze, SAMPLE_WALL};
use graphfinder_core::heuristic::{Manhattan, Zero};
use graphfinder_core::{search, Algorithm, Cell, SearchResult};

fn main() {
    let maze = Maze::from_ascii(SAMPLE_WALL);

    // Optimal cost as a reference (BFS is optimal on a unit-cost grid).
    let reference = search(
        &maze.grid,
        maze.start,
        maze.goal,
        Algorithm::bfs(),
        &Zero,
        false,
    );
    let optimal = reference.cost;

    println!(
        "Maze {}x{} | optimal cost = {optimal}\n",
        maze.grid.width(),
        maze.grid.height()
    );
    println!(
        "{:<14} {:>6} {:>9} {:>9} {:>9}",
        "algorithm", "cost", "optimal?", "expanded", "frontier"
    );
    println!("{}", "-".repeat(52));

    // Uninformed algorithms use the Zero heuristic; informed ones use Manhattan.
    report("BFS", run(&maze, Algorithm::bfs(), &Zero), optimal);
    report("DFS", run(&maze, Algorithm::dfs(), &Zero), optimal);
    report("UCS", run(&maze, Algorithm::ucs(), &Zero), optimal);
    report(
        "Greedy",
        run(&maze, Algorithm::greedy(), &Manhattan),
        optimal,
    );
    report("A*", run(&maze, Algorithm::astar(), &Manhattan), optimal);
    report(
        "Weighted A*",
        run(&maze, Algorithm::weighted_astar(2.0), &Manhattan),
        optimal,
    );
}

fn run<H: graphfinder_core::Heuristic<Cell>>(
    maze: &Maze,
    algo: Algorithm,
    h: &H,
) -> SearchResult<Cell> {
    search(&maze.grid, maze.start, maze.goal, algo, h, false)
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
        "{name:<14} {cost:>6} {opt:>9} {:>9} {:>9}",
        r.nodes_expanded, r.max_frontier_size
    );
}
