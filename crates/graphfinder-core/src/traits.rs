//! Core traits of the framework.
//!
//! Design philosophy (inherited from the sibling project `turboswarm`): the
//! search loop (in `search.rs`) knows nothing about any concrete algorithm.
//! Everything that distinguishes BFS from DFS from A* lives behind these three
//! traits. Adding a new algorithm = choosing a `Frontier` + an evaluation
//! function; adding a new domain = implementing `Graph`; adding a new informed
//! strategy = implementing `Heuristic`. The loop itself never changes.
//!
//! The unifying idea (Russell & Norvig's GENERAL-SEARCH): every traversal in
//! this library is the *same* loop driven by a priority `g_coeff·g(n) +
//! h_coeff·h(n)` and a frontier ordering. See [`crate::search::Algorithm`].

use std::hash::Hash;

/// The problem domain: a (possibly implicit) graph over nodes of type `Node`.
///
/// `neighbors` returns each successor together with the non-negative cost of
/// the edge that reaches it. Implementations may be *explicit* (an adjacency
/// structure held in memory, e.g. [`crate::CsrGraph`]) or *geometric*
/// (e.g. [`crate::GridGraph`]); implicit state-space graphs (puzzles generated
/// on the fly) arrive in a later phase but fit the same trait.
pub trait Graph {
    /// Identity of a node. Must be cheap to clone, hash and compare: the search
    /// keeps several maps keyed by it.
    type Node: Clone + Eq + Hash;

    /// Successors of `node` as `(neighbor, edge_cost)` pairs. Edge costs must be
    /// finite and non-negative (Dijkstra/A* assume it).
    fn neighbors(&self, node: &Self::Node) -> Vec<(Self::Node, f64)>;
}

/// An admissible-by-convention estimate `h(n)` of the remaining cost from a node
/// to the goal. The uninformed algorithms use [`crate::Zero`]; informed ones
/// (Greedy, A*) plug in a real estimate such as [`crate::Manhattan`].
///
/// Generic over the node type `N` so a single heuristic (e.g. `Zero`) can serve
/// every domain, while geometric ones are implemented only for the node types
/// where they make sense.
pub trait Heuristic<N> {
    /// Estimated cost from `node` to `goal`. Returning `0.0` everywhere turns an
    /// informed algorithm into its uninformed counterpart (A* → UCS).
    fn estimate(&self, node: &N, goal: &N) -> f64;
}

/// The frontier (a.k.a. the open list): the data structure that decides which
/// discovered-but-not-yet-expanded node comes out next. **This single choice is
/// what separates the algorithms:**
///
/// - FIFO queue ([`crate::Fifo`]) → Breadth-First Search,
/// - LIFO stack ([`crate::Lifo`]) → Depth-First Search,
/// - min-priority queue ([`crate::PriorityQueue`]) → UCS / Greedy / A*.
///
/// `push` receives the node and the priority computed by the loop; ordering
/// frontiers honour it, while FIFO/LIFO ignore it and order by insertion.
pub trait Frontier<N> {
    /// Insert `node` with the given `priority` (smaller = expanded sooner for
    /// ordering frontiers).
    fn push(&mut self, node: N, priority: f64);

    /// Remove and return the next node to expand, or `None` if empty.
    fn pop(&mut self) -> Option<N>;

    /// Number of nodes currently waiting in the frontier (used for the memory
    /// metric `max_frontier_size`).
    fn len(&self) -> usize;

    /// Whether the frontier is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
