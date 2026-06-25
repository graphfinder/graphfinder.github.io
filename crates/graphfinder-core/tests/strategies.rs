//! Tests for the Phase-2 strategies (DLS, IDDFS, IDA*, beam, bidirectional),
//! the node budget, and the random-graph generators.

use graphfinder_core::domains::{
    barabasi_albert, erdos_renyi, watts_strogatz, Maze, SAMPLE_OPEN, SAMPLE_WALL,
};
use graphfinder_core::heuristic::{Manhattan, Zero};
use graphfinder_core::{
    beam_search, bidirectional, dls, ida_star, iddfs, search, search_with, Algorithm, StopReason,
};

fn optimal_cost(maze: &Maze) -> f64 {
    search(
        &maze.grid,
        maze.start,
        maze.goal,
        Algorithm::bfs(),
        &Zero,
        false,
    )
    .cost
}

/// IDDFS is optimal on a unit-cost grid: it agrees with BFS on cost.
#[test]
fn iddfs_is_optimal_on_grids() {
    for map in [SAMPLE_OPEN, SAMPLE_WALL] {
        let m = Maze::from_ascii(map);
        let r = iddfs(&m.grid, m.start, m.goal, 100, false);
        assert!(r.found(), "IDDFS failed on {map:?}");
        assert_eq!(r.cost, optimal_cost(&m), "IDDFS cost on {map:?}");
    }
}

/// DLS fails below the goal depth and succeeds at or above it.
#[test]
fn dls_respects_the_depth_limit() {
    let m = Maze::from_ascii(SAMPLE_OPEN);
    let depth = optimal_cost(&m) as usize; // unit grid ⇒ depth == cost
    let too_shallow = dls(&m.grid, m.start, m.goal, depth - 1, false);
    assert!(!too_shallow.found());
    assert_eq!(too_shallow.stop_reason, StopReason::FrontierExhausted);
    let deep_enough = dls(&m.grid, m.start, m.goal, depth, false);
    assert!(deep_enough.found());
    assert_eq!(deep_enough.cost, optimal_cost(&m));
}

/// IDA* matches A* on cost (both optimal with an admissible heuristic).
#[test]
fn ida_star_matches_astar() {
    for map in [SAMPLE_OPEN, SAMPLE_WALL] {
        let m = Maze::from_ascii(map);
        let astar = search(
            &m.grid,
            m.start,
            m.goal,
            Algorithm::astar(),
            &Manhattan,
            false,
        );
        let ida = ida_star(&m.grid, m.start, m.goal, Manhattan, false);
        assert!(ida.found());
        assert_eq!(ida.cost, astar.cost, "IDA* vs A* on {map:?}");
    }
}

/// Bidirectional BFS finds a shortest path on a (symmetric) grid.
#[test]
fn bidirectional_is_optimal_on_grids() {
    for map in [SAMPLE_OPEN, SAMPLE_WALL] {
        let m = Maze::from_ascii(map);
        let r = bidirectional(&m.grid, m.start, m.goal, false);
        assert!(r.found(), "bidirectional failed on {map:?}");
        assert_eq!(r.cost, optimal_cost(&m), "bidirectional cost on {map:?}");
    }
}

/// Beam search finds *a* path when the beam is wide enough.
#[test]
fn beam_search_finds_a_path() {
    let m = Maze::from_ascii(SAMPLE_OPEN);
    let r = beam_search(&m.grid, m.start, m.goal, Manhattan, 8, false);
    assert!(r.found());
    assert!(r.cost >= optimal_cost(&m));
}

/// The node budget stops the search early with `NodeLimit` and no path.
#[test]
fn node_budget_stops_early() {
    let m = Maze::from_ascii(SAMPLE_WALL);
    let r = search_with(
        &m.grid,
        m.start,
        m.goal,
        Algorithm::bfs(),
        &Zero,
        false,
        Some(3),
    );
    assert!(!r.found());
    assert_eq!(r.stop_reason, StopReason::NodeLimit);
    assert_eq!(r.nodes_expanded, 3);
}

/// Random-graph generators are reproducible and have the requested node count.
#[test]
fn random_graphs_are_reproducible() {
    let a = erdos_renyi(60, 0.1, 7);
    let b = erdos_renyi(60, 0.1, 7);
    assert_eq!(a.num_nodes(), 60);
    assert_eq!(a.num_edges(), b.num_edges());

    let ba = barabasi_albert(80, 2, 3);
    assert_eq!(ba.num_nodes(), 80);
    assert!(ba.num_edges() > 0);

    let ws = watts_strogatz(40, 4, 0.1, 5);
    assert_eq!(ws.num_nodes(), 40);
}

/// On a connected scale-free graph, bidirectional BFS agrees with BFS on cost.
#[test]
fn strategies_agree_on_a_random_graph() {
    let g = barabasi_albert(120, 3, 11);
    let (start, goal) = (0usize, g.num_nodes() - 1);
    let bfs = search(&g, start, goal, Algorithm::bfs(), &Zero, false);
    let bidi = bidirectional(&g, start, goal, false);
    assert!(bfs.found() && bidi.found());
    assert_eq!(bfs.cost, bidi.cost);
}
