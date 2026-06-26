//! Optimality, correctness and reproducibility tests — the analogue of
//! turboswarm's convergence tests. Each algorithm is checked against the
//! property it is supposed to guarantee.

use graphfinder_core::domains::{random_maze, Maze, SAMPLE_OPEN, SAMPLE_WALL};
use graphfinder_core::heuristic::{Manhattan, Zero};
use graphfinder_core::{search, Algorithm, StopReason};

/// BFS, UCS and A* are all optimal on a unit-cost grid: they must agree on cost.
#[test]
fn optimal_algorithms_agree_on_cost() {
    for map in [SAMPLE_OPEN, SAMPLE_WALL] {
        let m = Maze::from_ascii(map);
        let bfs = search(&m.grid, m.start, m.goal, Algorithm::bfs(), &Zero, false);
        let ucs = search(&m.grid, m.start, m.goal, Algorithm::ucs(), &Zero, false);
        let astar = search(
            &m.grid,
            m.start,
            m.goal,
            Algorithm::astar(),
            &Manhattan,
            false,
        );

        assert!(bfs.found() && ucs.found() && astar.found());
        assert_eq!(bfs.cost, ucs.cost, "BFS vs UCS on {map:?}");
        assert_eq!(bfs.cost, astar.cost, "BFS vs A* on {map:?}");
    }
}

/// With an admissible heuristic, A* never expands more nodes than UCS.
#[test]
fn astar_expands_no_more_than_ucs() {
    let m = Maze::from_ascii(SAMPLE_WALL);
    let ucs = search(&m.grid, m.start, m.goal, Algorithm::ucs(), &Zero, false);
    let astar = search(
        &m.grid,
        m.start,
        m.goal,
        Algorithm::astar(),
        &Manhattan,
        false,
    );
    assert!(
        astar.nodes_expanded <= ucs.nodes_expanded,
        "A* expanded {} > UCS {}",
        astar.nodes_expanded,
        ucs.nodes_expanded
    );
}

/// DFS finds *a* path but is not required to be optimal.
#[test]
fn dfs_finds_a_path() {
    let m = Maze::from_ascii(SAMPLE_WALL);
    let dfs = search(&m.grid, m.start, m.goal, Algorithm::dfs(), &Zero, false);
    let bfs = search(&m.grid, m.start, m.goal, Algorithm::bfs(), &Zero, false);
    assert!(dfs.found());
    assert!(dfs.cost >= bfs.cost, "DFS cost cannot beat the optimum");
}

/// A walled-off goal yields no path and an exhausted frontier.
#[test]
fn no_path_when_goal_is_walled_off() {
    // The goal cell is fully enclosed by walls.
    let m = Maze::from_ascii("S....\n....#\n...#G\n....#");
    let r = search(&m.grid, m.start, m.goal, Algorithm::bfs(), &Zero, false);
    assert!(!r.found());
    assert_eq!(r.stop_reason, StopReason::FrontierExhausted);
    assert!(r.cost.is_infinite());
}

/// The trace records exactly one step per expanded node, ending at the goal.
#[test]
fn trace_is_recorded_and_consistent() {
    let m = Maze::from_ascii(SAMPLE_OPEN);
    let r = search(
        &m.grid,
        m.start,
        m.goal,
        Algorithm::astar(),
        &Manhattan,
        true,
    );
    assert_eq!(r.trace.len(), r.nodes_expanded);
    assert_eq!(r.trace.last().unwrap().expanded, m.goal);
}

/// On a weighted grid the least-cost path differs from the fewest-steps path:
/// BFS finds the short-but-expensive route, UCS/A* find the long-but-cheap one.
#[test]
fn weighted_grid_separates_bfs_from_ucs() {
    // Top row is expensive terrain (cost 9); the bottom row is a cheap detour.
    let (grid, start, goal) = graphfinder_core::GridGraph::from_ascii("S99G\n1111");

    let bfs = search(&grid, start, goal, Algorithm::bfs(), &Zero, false);
    let ucs = search(&grid, start, goal, Algorithm::ucs(), &Zero, false);
    let astar = search(&grid, start, goal, Algorithm::astar(), &Manhattan, false);

    // BFS: fewest steps (the 3-step top row) but expensive.
    assert_eq!(bfs.path_len(), Some(4)); // 4 nodes = 3 steps
    assert_eq!(bfs.cost, 19.0); // 9 + 9 + 1

    // UCS/A*: least cost via the longer detour.
    assert_eq!(ucs.cost, 5.0);
    assert_eq!(astar.cost, 5.0);
    assert_eq!(ucs.path_len(), Some(6)); // 6 nodes = 5 steps
    assert!(ucs.path_len() > bfs.path_len()); // cheaper path is longer in steps
    assert!(ucs.cost < bfs.cost);
}

/// `from_costs` turns non-positive / non-finite cells into walls.
#[test]
fn from_costs_marks_walls() {
    use graphfinder_core::{Cell, GridGraph};
    let grid = GridGraph::from_costs(&[vec![1.0, 5.0], vec![0.0, 1.0]]);
    assert_eq!(grid.cost_at(Cell::new(0, 1)), 5.0);
    assert!(grid.is_blocked(Cell::new(1, 0))); // cost 0.0 ⇒ wall
    assert!(grid.is_weighted());
}

/// Same seed ⇒ identical maze ⇒ identical result (reproducibility).
#[test]
fn random_maze_is_reproducible() {
    let a = random_maze(15, 15, 0.3, 42);
    let b = random_maze(15, 15, 0.3, 42);
    let ra = search(
        &a.grid,
        a.start,
        a.goal,
        Algorithm::astar(),
        &Manhattan,
        false,
    );
    let rb = search(
        &b.grid,
        b.start,
        b.goal,
        Algorithm::astar(),
        &Manhattan,
        false,
    );
    assert_eq!(ra.found(), rb.found());
    assert_eq!(ra.cost, rb.cost);
    assert_eq!(ra.nodes_expanded, rb.nodes_expanded);
}
