//! The single, algorithm-agnostic search loop.
//!
//! Every traversal in graphfinder is *this one function*. What changes between
//! BFS, DFS, UCS, Greedy and A* is captured by an [`Algorithm`]: which
//! [`Frontier`] to use and how to weight `g(n)` (cost so far) and `h(n)`
//! (heuristic) into the priority. The loop records a [`SearchResult`] with the
//! path plus the instrumentation that drives visualization and comparison.

use std::collections::{HashMap, HashSet};

use crate::frontier::{Fifo, Lifo, PriorityQueue};
use crate::traits::{Frontier, Graph, Heuristic};

/// Which underlying frontier an algorithm uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrontierKind {
    /// FIFO queue → BFS.
    Fifo,
    /// LIFO stack → DFS.
    Lifo,
    /// Min-priority queue → UCS / Greedy / A*.
    Priority,
}

/// A search algorithm = a frontier + an evaluation function
/// `priority = g_coeff·g(n) + h_coeff·h(n)`.
///
/// This is the heart of the library's pedagogy: the named constructors below
/// show that the classic algorithms differ *only* in these three knobs.
#[derive(Debug, Clone, Copy)]
pub struct Algorithm {
    pub frontier: FrontierKind,
    pub g_coeff: f64,
    pub h_coeff: f64,
    pub name: &'static str,
}

impl Algorithm {
    /// Breadth-First Search: FIFO frontier, priority unused. Complete; optimal
    /// only when every edge has the same cost.
    pub fn bfs() -> Self {
        Self {
            frontier: FrontierKind::Fifo,
            g_coeff: 1.0,
            h_coeff: 0.0,
            name: "BFS",
        }
    }

    /// Depth-First Search: LIFO frontier, priority unused. Not optimal; low
    /// memory. Finds *a* path, not necessarily the shortest.
    pub fn dfs() -> Self {
        Self {
            frontier: FrontierKind::Lifo,
            g_coeff: 1.0,
            h_coeff: 0.0,
            name: "DFS",
        }
    }

    /// Uniform-Cost Search (Dijkstra from a single source to a goal): priority
    /// = `g(n)`. Complete and optimal for non-negative edge costs.
    pub fn ucs() -> Self {
        Self {
            frontier: FrontierKind::Priority,
            g_coeff: 1.0,
            h_coeff: 0.0,
            name: "UCS",
        }
    }

    /// Alias of [`Algorithm::ucs`] under the name "Dijkstra".
    pub fn dijkstra() -> Self {
        Self {
            name: "Dijkstra",
            ..Self::ucs()
        }
    }

    /// Greedy Best-First Search: priority = `h(n)`. Fast but neither complete
    /// (on infinite graphs) nor optimal.
    pub fn greedy() -> Self {
        Self {
            frontier: FrontierKind::Priority,
            g_coeff: 0.0,
            h_coeff: 1.0,
            name: "Greedy",
        }
    }

    /// A*: priority = `g(n) + h(n)`. Complete, and optimal with a **consistent
    /// (monotone)** heuristic; expands the fewest nodes among optimal algorithms.
    ///
    /// Note: this is graph search with a closed set and no node reopening (see
    /// [`run`]), so optimality requires *consistency*, not merely admissibility.
    /// The built-in grid heuristics ([`crate::Manhattan`], [`crate::Euclidean`],
    /// [`crate::Octile`]) are all consistent; a custom heuristic that is only
    /// admissible can yield a sub-optimal path here.
    pub fn astar() -> Self {
        Self {
            frontier: FrontierKind::Priority,
            g_coeff: 1.0,
            h_coeff: 1.0,
            name: "A*",
        }
    }

    /// Weighted A*: priority = `g(n) + w·h(n)`, `w ≥ 1`. Trades optimality for
    /// speed; the returned cost is at most `w×` optimal.
    pub fn weighted_astar(w: f64) -> Self {
        Self {
            frontier: FrontierKind::Priority,
            g_coeff: 1.0,
            h_coeff: w,
            name: "Weighted A*",
        }
    }
}

/// Why the search stopped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StopReason {
    /// The goal was expanded — `path` is `Some`.
    GoalReached,
    /// The frontier emptied without reaching the goal — no path exists.
    FrontierExhausted,
    /// The optional node-expansion budget was hit first.
    NodeLimit,
}

/// One expansion event, recorded for visualization and analysis. The sequence
/// of `expanded` cells *is* the animation: replay them to watch the frontier
/// grow, then draw `path` on top.
#[derive(Debug, Clone)]
pub struct TraceStep<N> {
    /// The node taken off the frontier at this step.
    pub expanded: N,
    /// Its best-known cost from the start when expanded.
    pub g: f64,
    /// Frontier size right after expanding it (the memory curve).
    pub frontier_size: usize,
}

/// The outcome of a search, with path and instrumentation.
#[derive(Debug, Clone)]
pub struct SearchResult<N> {
    /// The solution path from start to goal, or `None` if unreachable.
    pub path: Option<Vec<N>>,
    /// Cost of `path` (`+∞` if none).
    pub cost: f64,
    /// Nodes taken off the frontier and expanded.
    pub nodes_expanded: usize,
    /// Nodes ever pushed onto the frontier (≈ work generated).
    pub nodes_generated: usize,
    /// Peak frontier size (a memory proxy).
    pub max_frontier_size: usize,
    /// Why the loop terminated.
    pub stop_reason: StopReason,
    /// Per-expansion trace (empty if `record` was `false`).
    pub trace: Vec<TraceStep<N>>,
    /// Edges `(parent, child)` of the **search tree** — each generated node
    /// linked to its best-known parent. Empty unless `record` was `true`. This
    /// is what `plot_search_tree` draws; for the iterative-deepening and
    /// bidirectional drivers in [`crate::strategies`] it is left empty.
    pub tree: Vec<(N, N)>,
}

impl<N> SearchResult<N> {
    /// Whether a path was found.
    pub fn found(&self) -> bool {
        self.path.is_some()
    }
    /// Number of nodes on the path (edges + 1), if any.
    pub fn path_len(&self) -> Option<usize> {
        self.path.as_ref().map(|p| p.len())
    }
}

/// Run `algorithm` on `graph` from `start` to `goal`, guided by `heuristic`.
///
/// Set `record` to keep the per-step [`TraceStep`] trace (needed for
/// animation); turn it off for raw speed. The same function powers every
/// algorithm — the `algorithm` argument selects the behaviour.
///
/// ```
/// use graphfinder_core::{search, Algorithm, GridGraph, Manhattan};
/// let (grid, start, goal) = GridGraph::from_ascii("S..\n.#.\n..G");
/// let r = search(&grid, start, goal, Algorithm::astar(), &Manhattan, true);
/// assert!(r.found());
/// assert_eq!(r.cost, 4.0); // 4 orthogonal steps around the wall
/// ```
pub fn search<G, H>(
    graph: &G,
    start: G::Node,
    goal: G::Node,
    algorithm: Algorithm,
    heuristic: &H,
    record: bool,
) -> SearchResult<G::Node>
where
    G: Graph,
    H: Heuristic<G::Node>,
{
    search_with(graph, start, goal, algorithm, heuristic, record, None)
}

/// Like [`search`], but with an optional `max_nodes` budget: if the search
/// expands that many nodes without reaching the goal it stops early with
/// [`StopReason::NodeLimit`] and no path. Useful for bounded experiments and
/// for keeping runaway searches in check.
pub fn search_with<G, H>(
    graph: &G,
    start: G::Node,
    goal: G::Node,
    algorithm: Algorithm,
    heuristic: &H,
    record: bool,
    max_nodes: Option<usize>,
) -> SearchResult<G::Node>
where
    G: Graph,
    H: Heuristic<G::Node>,
{
    match algorithm.frontier {
        FrontierKind::Fifo => run(
            graph,
            start,
            goal,
            algorithm,
            heuristic,
            record,
            max_nodes,
            Fifo::new(),
        ),
        FrontierKind::Lifo => run(
            graph,
            start,
            goal,
            algorithm,
            heuristic,
            record,
            max_nodes,
            Lifo::new(),
        ),
        FrontierKind::Priority => run(
            graph,
            start,
            goal,
            algorithm,
            heuristic,
            record,
            max_nodes,
            PriorityQueue::new(),
        ),
    }
}

/// The generic loop, monomorphised once per concrete frontier by [`search`].
#[allow(clippy::too_many_arguments)]
fn run<G, H, F>(
    graph: &G,
    start: G::Node,
    goal: G::Node,
    algo: Algorithm,
    heuristic: &H,
    record: bool,
    max_nodes: Option<usize>,
    mut frontier: F,
) -> SearchResult<G::Node>
where
    G: Graph,
    H: Heuristic<G::Node>,
    F: Frontier<G::Node>,
{
    let mut g_score: HashMap<G::Node, f64> = HashMap::new();
    let mut came_from: HashMap<G::Node, G::Node> = HashMap::new();
    let mut closed: HashSet<G::Node> = HashSet::new();

    g_score.insert(start.clone(), 0.0);
    let h0 = heuristic.estimate(&start, &goal);
    frontier.push(start.clone(), algo.g_coeff * 0.0 + algo.h_coeff * h0);

    let mut nodes_generated = 1usize;
    let mut nodes_expanded = 0usize;
    let mut max_frontier = 1usize;
    let mut trace = Vec::new();

    while let Some(node) = frontier.pop() {
        // The frontier may hold stale duplicates of a node that was reached by a
        // cheaper path after it was first pushed (the relaxation below re-pushes
        // on improvement, for any frontier kind); skip the ones already expanded.
        if closed.contains(&node) {
            continue;
        }
        closed.insert(node.clone());
        let g = *g_score.get(&node).expect("expanded node has a g-score");
        nodes_expanded += 1;
        if record {
            trace.push(TraceStep {
                expanded: node.clone(),
                g,
                frontier_size: frontier.len(),
            });
        }

        if node == goal {
            let path = reconstruct(&came_from, node);
            return SearchResult {
                path: Some(path),
                cost: g,
                nodes_expanded,
                nodes_generated,
                max_frontier_size: max_frontier,
                stop_reason: StopReason::GoalReached,
                tree: tree_edges(&came_from, record),
                trace,
            };
        }

        // Enforce the optional expansion budget (after the goal check, so a goal
        // found exactly at the limit still counts as success).
        if let Some(limit) = max_nodes {
            if nodes_expanded >= limit {
                return SearchResult {
                    path: None,
                    cost: f64::INFINITY,
                    nodes_expanded,
                    nodes_generated,
                    max_frontier_size: max_frontier,
                    stop_reason: StopReason::NodeLimit,
                    tree: tree_edges(&came_from, record),
                    trace,
                };
            }
        }

        for (nbr, w) in graph.neighbors(&node) {
            if closed.contains(&nbr) {
                continue;
            }
            let tentative = g + w;
            let better = match g_score.get(&nbr) {
                Some(&old) => tentative < old,
                None => true,
            };
            if better {
                g_score.insert(nbr.clone(), tentative);
                came_from.insert(nbr.clone(), node.clone());
                let priority =
                    algo.g_coeff * tentative + algo.h_coeff * heuristic.estimate(&nbr, &goal);
                frontier.push(nbr, priority);
                nodes_generated += 1;
                if frontier.len() > max_frontier {
                    max_frontier = frontier.len();
                }
            }
        }
    }

    SearchResult {
        path: None,
        cost: f64::INFINITY,
        nodes_expanded,
        nodes_generated,
        max_frontier_size: max_frontier,
        stop_reason: StopReason::FrontierExhausted,
        tree: tree_edges(&came_from, record),
        trace,
    }
}

/// Flatten the best-parent map into `(parent, child)` edges of the search tree.
/// Empty when `record` is off, so the cost is only paid for visualization.
fn tree_edges<N>(came_from: &HashMap<N, N>, record: bool) -> Vec<(N, N)>
where
    N: Clone + Eq + std::hash::Hash,
{
    if !record {
        return Vec::new();
    }
    came_from
        .iter()
        .map(|(child, parent)| (parent.clone(), child.clone()))
        .collect()
}

/// Walk the parent links from `goal` back to the start, returning the path in
/// start→goal order.
fn reconstruct<N>(came_from: &HashMap<N, N>, goal: N) -> Vec<N>
where
    N: Clone + Eq + std::hash::Hash,
{
    let mut path = vec![goal.clone()];
    let mut current = goal;
    while let Some(prev) = came_from.get(&current) {
        path.push(prev.clone());
        current = prev.clone();
    }
    path.reverse();
    path
}
