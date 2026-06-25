//! Phase-2 search strategies that need their own thin driver around the core
//! primitives: depth-limited / iterative-deepening search, IDA*, beam search and
//! bidirectional search. Each returns the same [`SearchResult`] as [`search`],
//! so they are directly comparable.
//!
//! [`search`]: crate::search

use std::collections::{HashMap, HashSet};

use crate::search::{SearchResult, StopReason, TraceStep};
use crate::traits::{Graph, Heuristic};

/// Cost of a path = sum of the edge weights along it (`0` for a 0/1-node path).
fn path_cost<G: Graph>(graph: &G, path: &[G::Node]) -> f64 {
    let mut total = 0.0;
    for pair in path.windows(2) {
        let (a, b) = (&pair[0], &pair[1]);
        let w = graph
            .neighbors(a)
            .into_iter()
            .find(|(n, _)| n == b)
            .map(|(_, w)| w)
            .unwrap_or(0.0);
        total += w;
    }
    total
}

fn solved<G: Graph>(
    graph: &G,
    path: Vec<G::Node>,
    nodes_expanded: usize,
    nodes_generated: usize,
    max_frontier_size: usize,
    trace: Vec<TraceStep<G::Node>>,
) -> SearchResult<G::Node> {
    let cost = path_cost(graph, &path);
    SearchResult {
        path: Some(path),
        cost,
        nodes_expanded,
        nodes_generated,
        max_frontier_size,
        stop_reason: StopReason::GoalReached,
        trace,
    }
}

fn unsolved<N>(
    stop_reason: StopReason,
    nodes_expanded: usize,
    nodes_generated: usize,
    max_frontier_size: usize,
    trace: Vec<TraceStep<N>>,
) -> SearchResult<N> {
    SearchResult {
        path: None,
        cost: f64::INFINITY,
        nodes_expanded,
        nodes_generated,
        max_frontier_size,
        stop_reason,
        trace,
    }
}

// ---------------------------------------------------------------------------
// Depth-Limited Search (DLS) and Iterative-Deepening DFS (IDDFS)
// ---------------------------------------------------------------------------

struct DlsCtx<G: Graph> {
    expanded: usize,
    generated: usize,
    max_frontier: usize,
    trace: Vec<TraceStep<G::Node>>,
    record: bool,
}

/// Recursive depth-limited visit. `on_path` is the set of nodes on the current
/// root→node branch (cycle avoidance); it is undone on backtrack so DLS behaves
/// as a tree search. Returns the path from `node` to the goal if found.
fn dls_visit<G: Graph>(
    graph: &G,
    node: G::Node,
    goal: &G::Node,
    depth_left: usize,
    g: f64,
    on_path: &mut HashSet<G::Node>,
    ctx: &mut DlsCtx<G>,
) -> Option<Vec<G::Node>> {
    ctx.expanded += 1;
    if ctx.record {
        ctx.trace.push(TraceStep {
            expanded: node.clone(),
            g,
            frontier_size: on_path.len(),
        });
    }
    if node == *goal {
        return Some(vec![node]);
    }
    if depth_left == 0 {
        return None;
    }
    on_path.insert(node.clone());
    ctx.max_frontier = ctx.max_frontier.max(on_path.len());
    for (nbr, w) in graph.neighbors(&node) {
        if on_path.contains(&nbr) {
            continue;
        }
        ctx.generated += 1;
        if let Some(child) = dls_visit(graph, nbr, goal, depth_left - 1, g + w, on_path, ctx) {
            on_path.remove(&node);
            let mut path = vec![node];
            path.extend(child);
            return Some(path);
        }
    }
    on_path.remove(&node);
    None
}

/// Depth-Limited Search: DFS that never descends deeper than `limit` edges.
/// Complete only if the goal is within `limit`; otherwise reports failure.
pub fn dls<G: Graph>(
    graph: &G,
    start: G::Node,
    goal: G::Node,
    limit: usize,
    record: bool,
) -> SearchResult<G::Node> {
    let mut ctx = DlsCtx::<G> {
        expanded: 0,
        generated: 1,
        max_frontier: 1,
        trace: Vec::new(),
        record,
    };
    let mut on_path = HashSet::new();
    match dls_visit(graph, start, &goal, limit, 0.0, &mut on_path, &mut ctx) {
        Some(path) => solved(
            graph,
            path,
            ctx.expanded,
            ctx.generated,
            ctx.max_frontier,
            ctx.trace,
        ),
        None => unsolved(
            StopReason::FrontierExhausted,
            ctx.expanded,
            ctx.generated,
            ctx.max_frontier,
            ctx.trace,
        ),
    }
}

/// Iterative-Deepening DFS: run [`dls`] with limit `0, 1, 2, …, max_depth`
/// until the goal is found. Combines DFS's low memory with BFS's optimality on
/// unit-cost graphs. Metrics accumulate across iterations.
pub fn iddfs<G: Graph>(
    graph: &G,
    start: G::Node,
    goal: G::Node,
    max_depth: usize,
    record: bool,
) -> SearchResult<G::Node>
where
    G::Node: Clone,
{
    let mut expanded = 0;
    let mut generated = 0;
    let mut max_frontier = 0;
    let mut trace = Vec::new();
    for limit in 0..=max_depth {
        let r = dls(graph, start.clone(), goal.clone(), limit, record);
        expanded += r.nodes_expanded;
        generated += r.nodes_generated;
        max_frontier = max_frontier.max(r.max_frontier_size);
        if record {
            trace.extend(r.trace);
        }
        if let Some(path) = r.path {
            return solved(graph, path, expanded, generated, max_frontier, trace);
        }
    }
    unsolved(
        StopReason::FrontierExhausted,
        expanded,
        generated,
        max_frontier,
        trace,
    )
}

// ---------------------------------------------------------------------------
// IDA* — iterative-deepening A*
// ---------------------------------------------------------------------------

struct IdaCtx<G: Graph, H> {
    expanded: usize,
    generated: usize,
    max_frontier: usize,
    trace: Vec<TraceStep<G::Node>>,
    record: bool,
    heuristic: H,
}

/// One bounded DFS pass for IDA*. Returns `Ok(path)` on success, or `Err(next)`
/// with the smallest `f = g + h` that exceeded `bound` (the next threshold), or
/// `Err(+∞)` if the branch is exhausted.
fn ida_visit<G: Graph, H: Heuristic<G::Node>>(
    graph: &G,
    node: G::Node,
    goal: &G::Node,
    g: f64,
    bound: f64,
    on_path: &mut HashSet<G::Node>,
    ctx: &mut IdaCtx<G, H>,
) -> Result<Vec<G::Node>, f64> {
    let f = g + ctx.heuristic.estimate(&node, goal);
    if f > bound {
        return Err(f);
    }
    ctx.expanded += 1;
    if ctx.record {
        ctx.trace.push(TraceStep {
            expanded: node.clone(),
            g,
            frontier_size: on_path.len(),
        });
    }
    if node == *goal {
        return Ok(vec![node]);
    }
    on_path.insert(node.clone());
    ctx.max_frontier = ctx.max_frontier.max(on_path.len());
    let mut next_bound = f64::INFINITY;
    for (nbr, w) in graph.neighbors(&node) {
        if on_path.contains(&nbr) {
            continue;
        }
        ctx.generated += 1;
        match ida_visit(graph, nbr, goal, g + w, bound, on_path, ctx) {
            Ok(child) => {
                on_path.remove(&node);
                let mut path = vec![node];
                path.extend(child);
                return Ok(path);
            }
            Err(exceeded) => next_bound = next_bound.min(exceeded),
        }
    }
    on_path.remove(&node);
    Err(next_bound)
}

/// IDA*: iterative deepening on the `f = g + h` threshold. Optimal with an
/// admissible heuristic, with DFS-level memory (no open list).
pub fn ida_star<G: Graph, H: Heuristic<G::Node>>(
    graph: &G,
    start: G::Node,
    goal: G::Node,
    heuristic: H,
    record: bool,
) -> SearchResult<G::Node> {
    let mut ctx = IdaCtx::<G, H> {
        expanded: 0,
        generated: 1,
        max_frontier: 1,
        trace: Vec::new(),
        record,
        heuristic,
    };
    let mut bound = ctx.heuristic.estimate(&start, &goal);
    loop {
        let mut on_path = HashSet::new();
        match ida_visit(
            graph,
            start.clone(),
            &goal,
            0.0,
            bound,
            &mut on_path,
            &mut ctx,
        ) {
            Ok(path) => {
                return solved(
                    graph,
                    path,
                    ctx.expanded,
                    ctx.generated,
                    ctx.max_frontier,
                    ctx.trace,
                )
            }
            Err(next) if next.is_infinite() => {
                return unsolved(
                    StopReason::FrontierExhausted,
                    ctx.expanded,
                    ctx.generated,
                    ctx.max_frontier,
                    ctx.trace,
                )
            }
            Err(next) => bound = next,
        }
    }
}

// ---------------------------------------------------------------------------
// Beam search — greedy best-first with a bounded frontier
// ---------------------------------------------------------------------------

/// Beam search: at each level keep only the `beam_width` most promising
/// successors (by `h`). Low, bounded memory; neither complete nor optimal —
/// it can prune the only path to the goal. `beam_width = usize::MAX` ≈ greedy.
pub fn beam_search<G: Graph, H: Heuristic<G::Node>>(
    graph: &G,
    start: G::Node,
    goal: G::Node,
    heuristic: H,
    beam_width: usize,
    record: bool,
) -> SearchResult<G::Node> {
    let mut came_from: HashMap<G::Node, G::Node> = HashMap::new();
    let mut visited: HashSet<G::Node> = HashSet::new();
    visited.insert(start.clone());
    let mut level = vec![start];
    let mut expanded = 0;
    let mut generated = 1;
    let mut max_frontier = 1;
    let mut trace = Vec::new();

    while !level.is_empty() {
        max_frontier = max_frontier.max(level.len());
        let mut candidates: Vec<(f64, G::Node)> = Vec::new();
        for node in level.drain(..) {
            expanded += 1;
            if record {
                trace.push(TraceStep {
                    expanded: node.clone(),
                    g: 0.0,
                    frontier_size: candidates.len(),
                });
            }
            if node == goal {
                let path = reconstruct(&came_from, node);
                return solved(graph, path, expanded, generated, max_frontier, trace);
            }
            for (nbr, _w) in graph.neighbors(&node) {
                if visited.insert(nbr.clone()) {
                    came_from.insert(nbr.clone(), node.clone());
                    generated += 1;
                    let h = heuristic.estimate(&nbr, &goal);
                    candidates.push((h, nbr));
                }
            }
        }
        // Keep the best `beam_width` candidates for the next level.
        candidates.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        candidates.truncate(beam_width);
        level = candidates.into_iter().map(|(_, n)| n).collect();
    }
    unsolved(
        StopReason::FrontierExhausted,
        expanded,
        generated,
        max_frontier,
        trace,
    )
}

// ---------------------------------------------------------------------------
// Bidirectional BFS
// ---------------------------------------------------------------------------

/// Bidirectional BFS: grow a frontier from the start and another from the goal,
/// alternating, until they meet. On unit-cost graphs this finds a shortest path
/// while exploring far fewer nodes than one-directional BFS.
///
/// **Assumes the graph is symmetric** (undirected, e.g. a grid): it uses
/// `neighbors` to expand the backward frontier too.
pub fn bidirectional<G: Graph>(
    graph: &G,
    start: G::Node,
    goal: G::Node,
    record: bool,
) -> SearchResult<G::Node> {
    if start == goal {
        return solved(graph, vec![start], 0, 1, 1, Vec::new());
    }
    // parent maps; the root maps to itself as a sentinel.
    let mut fwd: HashMap<G::Node, G::Node> = HashMap::new();
    let mut bwd: HashMap<G::Node, G::Node> = HashMap::new();
    fwd.insert(start.clone(), start.clone());
    bwd.insert(goal.clone(), goal.clone());
    let mut fq = vec![start.clone()];
    let mut bq = vec![goal.clone()];
    let mut expanded = 0;
    let mut generated = 2;
    let mut max_frontier = 2;
    let mut trace = Vec::new();

    while !fq.is_empty() && !bq.is_empty() {
        // Always expand the smaller frontier (the bidirectional speed-up).
        let forward = fq.len() <= bq.len();
        let current = if forward {
            std::mem::take(&mut fq)
        } else {
            std::mem::take(&mut bq)
        };
        let mut next = Vec::new();
        for node in current {
            expanded += 1;
            if record {
                trace.push(TraceStep {
                    expanded: node.clone(),
                    g: 0.0,
                    frontier_size: next.len(),
                });
            }
            for (nbr, _w) in graph.neighbors(&node) {
                // Read/write only the maps for the side we are expanding; check
                // the opposite map for a meeting point. The two maps are
                // distinct, so the borrows never alias.
                let seen_here = if forward {
                    fwd.contains_key(&nbr)
                } else {
                    bwd.contains_key(&nbr)
                };
                if seen_here {
                    continue;
                }
                if forward {
                    fwd.insert(nbr.clone(), node.clone());
                } else {
                    bwd.insert(nbr.clone(), node.clone());
                }
                generated += 1;
                let meets = if forward {
                    bwd.contains_key(&nbr)
                } else {
                    fwd.contains_key(&nbr)
                };
                if meets {
                    let path = join(&fwd, &bwd, &nbr);
                    return solved(graph, path, expanded, generated, max_frontier, trace);
                }
                next.push(nbr);
            }
        }
        max_frontier = max_frontier.max(next.len());
        if forward {
            fq = next;
        } else {
            bq = next;
        }
    }
    unsolved(
        StopReason::FrontierExhausted,
        expanded,
        generated,
        max_frontier,
        trace,
    )
}

/// Build the start→goal path through meeting node `meet`, given both parent maps
/// (each root maps to itself).
fn join<N>(fwd: &HashMap<N, N>, bwd: &HashMap<N, N>, meet: &N) -> Vec<N>
where
    N: Clone + Eq + std::hash::Hash,
{
    // start → meet
    let mut front = Vec::new();
    let mut cur = meet.clone();
    loop {
        front.push(cur.clone());
        let p = fwd.get(&cur).cloned();
        match p {
            Some(parent) if parent != cur => cur = parent,
            _ => break,
        }
    }
    front.reverse();
    // meet → goal (skip `meet`, already in `front`)
    let mut back = Vec::new();
    let mut cur = meet.clone();
    loop {
        let p = bwd.get(&cur).cloned();
        match p {
            Some(parent) if parent != cur => {
                back.push(parent.clone());
                cur = parent;
            }
            _ => break,
        }
    }
    front.extend(back);
    front
}

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
