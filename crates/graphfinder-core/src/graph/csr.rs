//! Explicit weighted graph in Compressed-Sparse-Row (CSR) form.
//!
//! CSR stores all neighbour lists back-to-back in one array, with an `offsets`
//! array marking where each node's slice begins. It is cache-friendly and the
//! standard layout for large graphs; here it backs uninformed search over
//! arbitrary weighted graphs (road networks, random graphs, …). Nodes are
//! plain `usize` indices, so only [`crate::Zero`] applies as a heuristic.

use crate::traits::Graph;

/// A static directed weighted graph in CSR layout. Build it once with
/// [`CsrGraph::from_edges`]; it is then immutable and fast to traverse.
#[derive(Clone, Debug)]
pub struct CsrGraph {
    offsets: Vec<usize>,
    targets: Vec<usize>,
    weights: Vec<f64>,
}

impl CsrGraph {
    /// Build from an edge list over `n` nodes (`0..n`). Each edge is
    /// `(from, to, weight)`. If `undirected`, every edge is inserted both ways.
    ///
    /// # Panics
    /// If any edge endpoint is `>= n`. Callers taking untrusted input (e.g. the
    /// Python binding) should validate endpoints first and surface a clean error.
    pub fn from_edges(n: usize, edges: &[(usize, usize, f64)], undirected: bool) -> Self {
        // Count out-degree per node (counting both directions if undirected).
        let mut degree = vec![0usize; n];
        for &(u, v, _) in edges {
            assert!(
                u < n && v < n,
                "edge ({u}, {v}) is out of range for a {n}-node graph"
            );
            degree[u] += 1;
            if undirected {
                degree[v] += 1;
            }
        }
        // Prefix-sum the degrees into offsets.
        let mut offsets = vec![0usize; n + 1];
        for i in 0..n {
            offsets[i + 1] = offsets[i] + degree[i];
        }
        let total = offsets[n];
        let mut targets = vec![0usize; total];
        let mut weights = vec![0.0f64; total];
        let mut cursor = offsets[..n].to_vec();
        let mut place = |u: usize, v: usize, w: f64| {
            let p = cursor[u];
            targets[p] = v;
            weights[p] = w;
            cursor[u] += 1;
        };
        for &(u, v, w) in edges {
            place(u, v, w);
            if undirected {
                place(v, u, w);
            }
        }
        Self {
            offsets,
            targets,
            weights,
        }
    }

    /// Number of nodes.
    pub fn num_nodes(&self) -> usize {
        self.offsets.len().saturating_sub(1)
    }

    /// Number of stored (directed) edges.
    pub fn num_edges(&self) -> usize {
        self.targets.len()
    }

    /// All stored directed edges as `(from, to, weight)`. For a graph built with
    /// `undirected = true` each undirected edge appears in both directions.
    pub fn edges(&self) -> Vec<(usize, usize, f64)> {
        let mut out = Vec::with_capacity(self.num_edges());
        for u in 0..self.num_nodes() {
            for i in self.offsets[u]..self.offsets[u + 1] {
                out.push((u, self.targets[i], self.weights[i]));
            }
        }
        out
    }
}

impl Graph for CsrGraph {
    type Node = usize;

    fn neighbors(&self, node: &usize) -> Vec<(usize, f64)> {
        let (lo, hi) = (self.offsets[*node], self.offsets[*node + 1]);
        (lo..hi)
            .map(|i| (self.targets[i], self.weights[i]))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "out of range")]
    fn from_edges_rejects_out_of_range_endpoint() {
        // Node 5 does not exist in a 3-node graph.
        CsrGraph::from_edges(3, &[(0, 5, 1.0)], false);
    }
}
