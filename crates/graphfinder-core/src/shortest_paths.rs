//! Classic **shortest-path** algorithms that are *not* frontier traversals.
//!
//! [`search`] (and the [`strategies`] drivers) implement Russell & Norvig's
//! GENERAL-SEARCH: a goal-directed walk over a frontier. Bellman–Ford and
//! Floyd–Warshall are different in kind — they are **relaxation / dynamic
//! programming** methods that compute *all* the distances from a source (or
//! between *every* pair of nodes) at once, and they handle **negative edge
//! weights** that Dijkstra/A\* cannot. So they live here, on the explicit
//! [`CsrGraph`], rather than behind the `Frontier` trait.
//!
//! | Algorithm        | Computes            | Negative edges | Detects neg. cycle | Time      |
//! |------------------|---------------------|----------------|--------------------|-----------|
//! | [`bellman_ford`] | one source → all    | yes            | yes (from source)  | `O(V·E)`  |
//! | [`floyd_warshall`] | all pairs         | yes            | yes (any)          | `O(V³)`   |
//!
//! The library-wide invariant that edge costs are non-negative applies to the
//! *frontier* algorithms (Dijkstra/A\* assume it). These two relax it on
//! purpose — that is the whole point of including them.
//!
//! [`search`]: crate::search
//! [`strategies`]: crate::strategies
//! [`CsrGraph`]: crate::CsrGraph

use crate::graph::CsrGraph;

/// Single-source shortest paths, as returned by [`bellman_ford`].
#[derive(Debug, Clone)]
pub struct ShortestPaths {
    /// The source node every distance is measured from.
    pub source: usize,
    /// `dist[v]` = cost of the cheapest `source → v` path (`+∞` if `v` is
    /// unreachable). When a negative cycle is reachable these are the values
    /// after `V−1` relaxation rounds and are **not** the true minima.
    pub dist: Vec<f64>,
    /// `pred[v]` = the predecessor of `v` on a shortest path, or `None` for the
    /// source itself and for unreachable nodes. Walk it back to rebuild a path.
    pub pred: Vec<Option<usize>>,
    /// `true` if a negative-weight cycle is reachable from `source`, in which
    /// case no finite shortest path is well defined for the affected nodes.
    pub negative_cycle: bool,
}

impl ShortestPaths {
    /// Number of nodes.
    pub fn num_nodes(&self) -> usize {
        self.dist.len()
    }

    /// Rebuild the `source → target` shortest path (inclusive), or `None` if
    /// `target` is unreachable. Returns `[source]` when `target == source`.
    pub fn path_to(&self, target: usize) -> Option<Vec<usize>> {
        if target >= self.dist.len() || !self.dist[target].is_finite() {
            return None;
        }
        let mut path = vec![target];
        let mut cur = target;
        while let Some(p) = self.pred[cur] {
            path.push(p);
            cur = p;
            // Guard against a malformed predecessor chain.
            if path.len() > self.dist.len() {
                return None;
            }
        }
        path.reverse();
        Some(path)
    }
}

/// **Bellman–Ford**: single-source shortest paths that tolerate negative edge
/// weights and report a reachable negative cycle.
///
/// Relax every edge `V−1` times; if a `V`-th pass can still relax an edge, a
/// negative cycle is reachable and [`ShortestPaths::negative_cycle`] is set.
/// Use this instead of Dijkstra/[`crate::Algorithm::ucs`] exactly when some edge
/// weight can be negative.
///
/// ```
/// use graphfinder_core::{CsrGraph, bellman_ford};
/// // 0 →(4)→ 1, 0 →(5)→ 2, 1 →(-3)→ 2  : the cheapest 0→2 path is 0→1→2 = 1.
/// let g = CsrGraph::from_edges(3, &[(0, 1, 4.0), (0, 2, 5.0), (1, 2, -3.0)], false);
/// let sp = bellman_ford(&g, 0);
/// assert!(!sp.negative_cycle);
/// assert_eq!(sp.dist[2], 1.0);
/// assert_eq!(sp.path_to(2), Some(vec![0, 1, 2]));
/// ```
pub fn bellman_ford(graph: &CsrGraph, source: usize) -> ShortestPaths {
    let n = graph.num_nodes();
    let mut dist = vec![f64::INFINITY; n];
    let mut pred = vec![None; n];
    if source >= n {
        return ShortestPaths {
            source,
            dist,
            pred,
            negative_cycle: false,
        };
    }
    dist[source] = 0.0;
    let edges = graph.edges();

    // V−1 rounds of relaxation suffice for any shortest path (≤ V−1 edges).
    for _ in 1..n {
        let mut changed = false;
        for &(u, v, w) in &edges {
            if dist[u].is_finite() && dist[u] + w < dist[v] {
                dist[v] = dist[u] + w;
                pred[v] = Some(u);
                changed = true;
            }
        }
        if !changed {
            break; // settled early
        }
    }

    // One more pass: any further relaxation proves a reachable negative cycle.
    let mut negative_cycle = false;
    for &(u, v, w) in &edges {
        if dist[u].is_finite() && dist[u] + w < dist[v] {
            negative_cycle = true;
            break;
        }
    }

    ShortestPaths {
        source,
        dist,
        pred,
        negative_cycle,
    }
}

/// All-pairs shortest paths, as returned by [`floyd_warshall`].
#[derive(Debug, Clone)]
pub struct AllPairs {
    n: usize,
    /// Row-major `n×n` distance matrix; `dist[i*n + j]` is `i → j`.
    dist: Vec<f64>,
    /// Row-major `n×n` "next hop" matrix for path reconstruction.
    next: Vec<Option<usize>>,
    /// `true` if the graph contains any negative-weight cycle.
    pub negative_cycle: bool,
}

impl AllPairs {
    /// Number of nodes.
    pub fn num_nodes(&self) -> usize {
        self.n
    }

    /// Cost of the cheapest `from → to` path (`+∞` if none, `0` for `from==to`).
    pub fn distance(&self, from: usize, to: usize) -> f64 {
        if from >= self.n || to >= self.n {
            return f64::INFINITY;
        }
        self.dist[from * self.n + to]
    }

    /// The full `n×n` distance matrix as rows (unreachable entries are `+∞`).
    pub fn matrix(&self) -> Vec<Vec<f64>> {
        (0..self.n)
            .map(|i| self.dist[i * self.n..(i + 1) * self.n].to_vec())
            .collect()
    }

    /// Rebuild the cheapest `from → to` path (inclusive), or `None` if there is
    /// none. Returns `[from]` when `from == to`.
    pub fn path(&self, from: usize, to: usize) -> Option<Vec<usize>> {
        if from >= self.n || to >= self.n || !self.distance(from, to).is_finite() {
            return None;
        }
        if from == to {
            return Some(vec![from]);
        }
        let mut path = vec![from];
        let mut cur = from;
        while cur != to {
            match self.next[cur * self.n + to] {
                Some(nxt) => {
                    cur = nxt;
                    path.push(cur);
                }
                None => return None,
            }
            if path.len() > self.n {
                return None; // malformed (e.g. a negative cycle on the route)
            }
        }
        Some(path)
    }
}

/// **Floyd–Warshall**: all-pairs shortest paths by dynamic programming over an
/// allowed-intermediate-vertex set. `O(V³)` time and `O(V²)` memory, so it fits
/// dense or small/medium graphs where you want *every* distance at once.
///
/// Tolerates negative edges and sets [`AllPairs::negative_cycle`] if any vertex
/// ends up with a negative distance to itself.
///
/// ```
/// use graphfinder_core::{CsrGraph, floyd_warshall};
/// let g = CsrGraph::from_edges(3, &[(0, 1, 4.0), (0, 2, 5.0), (1, 2, -3.0)], false);
/// let ap = floyd_warshall(&g);
/// assert!(!ap.negative_cycle);
/// assert_eq!(ap.distance(0, 2), 1.0);
/// assert_eq!(ap.path(0, 2), Some(vec![0, 1, 2]));
/// ```
pub fn floyd_warshall(graph: &CsrGraph) -> AllPairs {
    let n = graph.num_nodes();
    let mut dist = vec![f64::INFINITY; n * n];
    let mut next = vec![None; n * n];

    for i in 0..n {
        dist[i * n + i] = 0.0;
    }
    // Seed with edges; keep the cheapest when parallel edges exist.
    for (u, v, w) in graph.edges() {
        let idx = u * n + v;
        if w < dist[idx] {
            dist[idx] = w;
            next[idx] = Some(v);
        }
    }

    for k in 0..n {
        for i in 0..n {
            let dik = dist[i * n + k];
            if !dik.is_finite() {
                continue;
            }
            for j in 0..n {
                let through = dik + dist[k * n + j];
                if through < dist[i * n + j] {
                    dist[i * n + j] = through;
                    next[i * n + j] = next[i * n + k];
                }
            }
        }
    }

    let negative_cycle = (0..n).any(|i| dist[i * n + i] < 0.0);

    AllPairs {
        n,
        dist,
        next,
        negative_cycle,
    }
}
