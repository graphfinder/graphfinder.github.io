//! Tests for the relaxation/DP shortest-path algorithms (Bellman–Ford and
//! Floyd–Warshall): agreement with UCS on non-negative graphs, correct handling
//! of negative edges, negative-cycle detection, and all-pairs consistency.

use graphfinder_core::{bellman_ford, floyd_warshall, search, Algorithm, CsrGraph, Zero};

/// A small non-negative directed graph with several competing routes.
fn nonneg_graph() -> CsrGraph {
    CsrGraph::from_edges(
        5,
        &[
            (0, 1, 2.0),
            (0, 2, 5.0),
            (1, 2, 1.0),
            (1, 3, 7.0),
            (2, 3, 1.0),
            (3, 4, 3.0),
            (2, 4, 8.0),
        ],
        false,
    )
}

/// On a non-negative graph, Bellman–Ford must agree with UCS/Dijkstra on every
/// reachable distance.
#[test]
fn bellman_ford_matches_ucs_when_non_negative() {
    let g = nonneg_graph();
    let sp = bellman_ford(&g, 0);
    assert!(!sp.negative_cycle);
    for target in 0..g.num_nodes() {
        let ucs = search(&g, 0usize, target, Algorithm::ucs(), &Zero, false);
        if ucs.found() {
            assert_eq!(sp.dist[target], ucs.cost, "dist to {target}");
            assert_eq!(sp.path_to(target), ucs.path, "path to {target}");
        } else {
            assert!(
                !sp.dist[target].is_finite(),
                "{target} should be unreachable"
            );
        }
    }
}

/// Bellman–Ford finds the cheapest path through a negative edge, where a greedy
/// non-negative method (Dijkstra) could commit to a worse route.
#[test]
fn bellman_ford_handles_negative_edges() {
    // 0→1 = 4, 0→2 = 5, 1→2 = -3  ⇒ cheapest 0→2 is 0→1→2 = 1, not the direct 5.
    let g = CsrGraph::from_edges(3, &[(0, 1, 4.0), (0, 2, 5.0), (1, 2, -3.0)], false);
    let sp = bellman_ford(&g, 0);
    assert!(!sp.negative_cycle);
    assert_eq!(sp.dist[2], 1.0);
    assert_eq!(sp.path_to(2), Some(vec![0, 1, 2]));
}

/// A reachable negative cycle is detected.
#[test]
fn bellman_ford_detects_negative_cycle() {
    // 0→1→2→0 sums to -1: a negative cycle reachable from 0.
    let g = CsrGraph::from_edges(3, &[(0, 1, 1.0), (1, 2, -3.0), (2, 0, 1.0)], false);
    let sp = bellman_ford(&g, 0);
    assert!(sp.negative_cycle);
}

/// Floyd–Warshall agrees with Bellman–Ford run from every source.
#[test]
fn floyd_warshall_matches_bellman_ford_all_sources() {
    let g = nonneg_graph();
    let ap = floyd_warshall(&g);
    assert!(!ap.negative_cycle);
    for s in 0..g.num_nodes() {
        let sp = bellman_ford(&g, s);
        for t in 0..g.num_nodes() {
            assert_eq!(ap.distance(s, t), sp.dist[t], "dist {s}->{t}");
        }
        assert_eq!(ap.path(s, s), Some(vec![s]));
    }
    // Spot-check a reconstructed multi-hop path.
    assert_eq!(ap.path(0, 4), Some(vec![0, 1, 2, 3, 4])); // 2+1+1+3
    assert_eq!(ap.distance(0, 4), 7.0);
}

/// Floyd–Warshall also flags a negative cycle (a vertex reaching itself < 0).
#[test]
fn floyd_warshall_detects_negative_cycle() {
    let g = CsrGraph::from_edges(3, &[(0, 1, 1.0), (1, 2, -3.0), (2, 0, 1.0)], false);
    let ap = floyd_warshall(&g);
    assert!(ap.negative_cycle);
}

/// Unreachable nodes report `+∞` distance and `None` path.
#[test]
fn unreachable_nodes_are_infinite() {
    // node 2 is isolated.
    let g = CsrGraph::from_edges(3, &[(0, 1, 1.0)], false);
    let sp = bellman_ford(&g, 0);
    assert!(!sp.dist[2].is_finite());
    assert_eq!(sp.path_to(2), None);

    let ap = floyd_warshall(&g);
    assert!(!ap.distance(0, 2).is_finite());
    assert_eq!(ap.path(0, 2), None);
}
