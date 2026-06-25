//! Seeded random-graph generators producing [`CsrGraph`] instances — the
//! non-grid "benchmarks" for uninformed search and the strategy drivers. All
//! edges have unit weight; same `seed` ⇒ same graph.

use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::graph::CsrGraph;

/// Erdős–Rényi `G(n, p)`: each of the `n·(n-1)/2` possible undirected edges is
/// present independently with probability `p`.
pub fn erdos_renyi(n: usize, p: f64, seed: u64) -> CsrGraph {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut edges = Vec::new();
    for u in 0..n {
        for v in (u + 1)..n {
            if rng.gen::<f64>() < p {
                edges.push((u, v, 1.0));
            }
        }
    }
    CsrGraph::from_edges(n, &edges, true)
}

/// Barabási–Albert preferential-attachment graph: start from a small clique of
/// `m` nodes and add the rest, each connecting to `m` existing nodes chosen with
/// probability proportional to their degree (yields a scale-free degree
/// distribution).
pub fn barabasi_albert(n: usize, m: usize, seed: u64) -> CsrGraph {
    assert!(m >= 1 && n > m, "need n > m >= 1");
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut edges: Vec<(usize, usize, f64)> = Vec::new();
    // `targets` is a multiset of node ids; sampling from it is degree-weighted.
    let mut targets: Vec<usize> = Vec::new();
    // Initial clique of the first m nodes.
    for u in 0..m {
        for v in (u + 1)..m {
            edges.push((u, v, 1.0));
            targets.push(u);
            targets.push(v);
        }
    }
    for new_node in m..n {
        let mut chosen = std::collections::HashSet::new();
        while chosen.len() < m {
            let pick = *targets.choose(&mut rng).unwrap_or(&0);
            chosen.insert(pick);
        }
        for &t in &chosen {
            edges.push((new_node, t, 1.0));
            targets.push(new_node);
            targets.push(t);
        }
    }
    CsrGraph::from_edges(n, &edges, true)
}

/// Watts–Strogatz small-world graph: a ring lattice where each node links to its
/// `k` nearest neighbours, then each edge is rewired with probability `beta`.
pub fn watts_strogatz(n: usize, k: usize, beta: f64, seed: u64) -> CsrGraph {
    assert!(k.is_multiple_of(2) && k < n, "k must be even and < n");
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut edge_set: std::collections::HashSet<(usize, usize)> = std::collections::HashSet::new();
    let norm = |a: usize, b: usize| if a < b { (a, b) } else { (b, a) };
    // Ring lattice: connect each node to its k/2 neighbours on each side.
    for u in 0..n {
        for j in 1..=(k / 2) {
            let v = (u + j) % n;
            edge_set.insert(norm(u, v));
        }
    }
    // Rewire.
    let lattice: Vec<(usize, usize)> = edge_set.iter().copied().collect();
    for (u, v) in lattice {
        if rng.gen::<f64>() < beta {
            edge_set.remove(&norm(u, v));
            let mut w = rng.gen_range(0..n);
            while w == u || edge_set.contains(&norm(u, w)) {
                w = rng.gen_range(0..n);
            }
            edge_set.insert(norm(u, w));
        }
    }
    let edges: Vec<(usize, usize, f64)> = edge_set.into_iter().map(|(a, b)| (a, b, 1.0)).collect();
    CsrGraph::from_edges(n, &edges, true)
}
