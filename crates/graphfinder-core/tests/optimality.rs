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
