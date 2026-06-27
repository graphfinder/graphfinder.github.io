//! Python (PyO3) bindings for the graphfinder Rust core.
//!
//! Three search entry points, all returning a `SearchResult`:
//!   - `search_grid`    — a 2-D maze given as an ASCII map (the teaching path),
//!   - `search_graph`   — an explicit weighted graph given as an edge list,
//!   - `search_implicit`— an *implicit* graph defined by a Python successor
//!     callable (states are ints or tuples of ints), reacquiring the GIL per
//!     expansion. Native domains run with the GIL released.
//!
//! Plus reproducible random-graph generators and maze helpers.

// The `#[pyfunction]` macro (PyO3 0.22) emits a `.into()` on the returned
// `PyErr`, which clippy 1.91 flags as a useless conversion. It is macro-generated
// code we don't control, so silence it crate-wide.
#![allow(clippy::useless_conversion)]

use std::marker::PhantomData;

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyList, PyTuple};

use graphfinder_core::domains::{
    barabasi_albert, erdos_renyi, random_maze, watts_strogatz, SAMPLE_OPEN, SAMPLE_WALL,
};
use graphfinder_core::{
    beam_search, bellman_ford, bidirectional, dls, floyd_warshall, ida_star, iddfs, search_with,
    Algorithm, Cell, CsrGraph, Euclidean, Graph, GridGraph, Heuristic, Manhattan, Octile,
    SearchResult, StopReason, Zero,
};

fn value_err(msg: impl Into<String>) -> PyErr {
    PyValueError::new_err(msg.into())
}

// ---------------------------------------------------------------------------
// Result class
// ---------------------------------------------------------------------------

/// The outcome of a search.
///
/// Attributes:
///     path (list | None): nodes from start to goal, or ``None`` if unreachable.
///         Grid nodes are ``(row, col)`` tuples; graph nodes are ints; implicit
///         nodes are ints or tuples (as your successor function returns them).
///     cost (float): total cost of ``path`` (``inf`` if none).
///     found (bool): whether a path was found.
///     nodes_expanded (int): nodes taken off the frontier and expanded.
///     nodes_generated (int): nodes ever pushed onto the frontier.
///     max_frontier_size (int): peak frontier size (a memory proxy).
///     stop_reason (str): ``"goal"``, ``"exhausted"`` or ``"node_limit"``.
///     trace (list): per-expansion ``(node, g, frontier_size)`` tuples; empty if
///         ``record=False``. Replaying ``node`` reproduces the search order
///         (this drives the visualization in Phase 4).
#[pyclass(name = "SearchResult")]
struct PySearchResult {
    #[pyo3(get)]
    path: Option<Py<PyAny>>,
    #[pyo3(get)]
    cost: f64,
    #[pyo3(get)]
    found: bool,
    #[pyo3(get)]
    nodes_expanded: usize,
    #[pyo3(get)]
    nodes_generated: usize,
    #[pyo3(get)]
    max_frontier_size: usize,
    #[pyo3(get)]
    stop_reason: String,
    #[pyo3(get)]
    trace: Py<PyAny>,
}

#[pymethods]
impl PySearchResult {
    fn __repr__(&self) -> String {
        let cost = if self.found {
            format!("{}", self.cost)
        } else {
            "inf".to_string()
        };
        format!(
            "SearchResult(found={}, cost={}, expanded={}, frontier={}, stop={})",
            self.found, cost, self.nodes_expanded, self.max_frontier_size, self.stop_reason
        )
    }
}

fn stop_str(reason: StopReason) -> String {
    match reason {
        StopReason::GoalReached => "goal",
        StopReason::FrontierExhausted => "exhausted",
        StopReason::NodeLimit => "node_limit",
    }
    .to_string()
}

/// Turn a Rust node into the Python object the user sees.
trait IntoPyNode {
    fn into_py_node(self, py: Python<'_>) -> Py<PyAny>;
}
impl IntoPyNode for Cell {
    fn into_py_node(self, py: Python<'_>) -> Py<PyAny> {
        (self.row, self.col).into_py(py)
    }
}
impl IntoPyNode for usize {
    fn into_py_node(self, py: Python<'_>) -> Py<PyAny> {
        (self as i64).into_py(py)
    }
}
impl IntoPyNode for Vec<i64> {
    fn into_py_node(self, py: Python<'_>) -> Py<PyAny> {
        state_to_py(py, &self)
    }
}

fn to_py_result<N: IntoPyNode>(py: Python<'_>, r: SearchResult<N>) -> PyResult<PySearchResult> {
    let found = r.path.is_some();
    let path = match r.path {
        Some(p) => {
            let items: Vec<Py<PyAny>> = p.into_iter().map(|n| n.into_py_node(py)).collect();
            Some(PyList::new_bound(py, items).into())
        }
        None => None,
    };
    let trace = PyList::empty_bound(py);
    for step in r.trace {
        let item: Py<PyAny> =
            (step.expanded.into_py_node(py), step.g, step.frontier_size).into_py(py);
        trace.append(item)?;
    }
    Ok(PySearchResult {
        path,
        cost: r.cost,
        found,
        nodes_expanded: r.nodes_expanded,
        nodes_generated: r.nodes_generated,
        max_frontier_size: r.max_frontier_size,
        stop_reason: stop_str(r.stop_reason),
        trace: trace.into(),
    })
}

// ---------------------------------------------------------------------------
// Algorithm dispatch (shared by every domain)
// ---------------------------------------------------------------------------

struct RunOpts {
    weight: f64,
    beam_width: usize,
    depth_limit: Option<usize>,
    max_nodes: Option<usize>,
}

/// Run the named algorithm on any `Graph`. Pure Rust — safe to call with the
/// GIL released for native domains.
fn run_algo<G, H>(
    graph: &G,
    start: G::Node,
    goal: G::Node,
    algorithm: &str,
    heuristic: H,
    record: bool,
    opts: &RunOpts,
) -> PyResult<SearchResult<G::Node>>
where
    G: Graph,
    H: Heuristic<G::Node> + Clone,
{
    let max_depth = opts.depth_limit.unwrap_or(1000);
    let r = match algorithm {
        "bfs" => search_with(
            graph,
            start,
            goal,
            Algorithm::bfs(),
            &heuristic,
            record,
            opts.max_nodes,
        ),
        "dfs" => search_with(
            graph,
            start,
            goal,
            Algorithm::dfs(),
            &heuristic,
            record,
            opts.max_nodes,
        ),
        "ucs" => search_with(
            graph,
            start,
            goal,
            Algorithm::ucs(),
            &heuristic,
            record,
            opts.max_nodes,
        ),
        "dijkstra" => search_with(
            graph,
            start,
            goal,
            Algorithm::dijkstra(),
            &heuristic,
            record,
            opts.max_nodes,
        ),
        "greedy" => search_with(
            graph,
            start,
            goal,
            Algorithm::greedy(),
            &heuristic,
            record,
            opts.max_nodes,
        ),
        "astar" | "a*" => search_with(
            graph,
            start,
            goal,
            Algorithm::astar(),
            &heuristic,
            record,
            opts.max_nodes,
        ),
        "weighted_astar" | "wastar" => search_with(
            graph,
            start,
            goal,
            Algorithm::weighted_astar(opts.weight),
            &heuristic,
            record,
            opts.max_nodes,
        ),
        "iddfs" => iddfs(graph, start, goal, max_depth, record),
        "dls" => {
            let limit = opts
                .depth_limit
                .ok_or_else(|| value_err("'dls' requires depth_limit"))?;
            dls(graph, start, goal, limit, record)
        }
        "ida_star" | "idastar" => ida_star(graph, start, goal, heuristic.clone(), record),
        "beam" => beam_search(
            graph,
            start,
            goal,
            heuristic.clone(),
            opts.beam_width,
            record,
        ),
        "bidirectional" | "bidir" => bidirectional(graph, start, goal, record),
        other => {
            return Err(value_err(format!(
                "unknown algorithm: '{other}'. Available: bfs, dfs, ucs, dijkstra, greedy, \
                 astar, weighted_astar, iddfs, dls, ida_star, beam, bidirectional"
            )))
        }
    };
    Ok(r)
}

// ---------------------------------------------------------------------------
// Implicit graph backed by a Python successor callable
// ---------------------------------------------------------------------------

/// Encode a state (int → scalar, tuple/list of ints → tuple) for Python.
fn state_to_py(py: Python<'_>, v: &[i64]) -> Py<PyAny> {
    if v.len() == 1 {
        v[0].into_py(py)
    } else {
        PyTuple::new_bound(py, v).into()
    }
}

/// Decode a Python state into the internal `Vec<i64>` key.
fn py_to_state(obj: &Bound<'_, PyAny>) -> PyResult<Vec<i64>> {
    if let Ok(scalar) = obj.extract::<i64>() {
        return Ok(vec![scalar]);
    }
    obj.extract::<Vec<i64>>()
        .map_err(|_| value_err("implicit-graph states must be an int or a sequence of ints"))
}

struct PyImplicitGraph {
    successors: Py<PyAny>,
}

impl Graph for PyImplicitGraph {
    type Node = Vec<i64>;

    fn neighbors(&self, node: &Vec<i64>) -> Vec<(Vec<i64>, f64)> {
        Python::with_gil(|py| {
            let arg = state_to_py(py, node);
            let result = self
                .successors
                .call1(py, (arg,))
                .expect("the successors callable raised an exception");
            let pairs: Vec<(Py<PyAny>, f64)> = result
                .extract(py)
                .expect("successors must return a list of (state, cost) pairs");
            pairs
                .into_iter()
                .map(|(state, cost)| {
                    let key = py_to_state(state.bind(py)).expect("invalid successor state");
                    (key, cost)
                })
                .collect()
        })
    }
}

/// A heuristic backed by a Python callable `h(node, goal) -> float`, generic
/// over the node type. `None` behaves as the zero heuristic. Works for any node
/// that can be handed to Python (grids → `(row, col)`, graphs → `int`, implicit
/// → `int`/tuple), so the same adapter serves every domain.
struct PyHeuristic<N> {
    func: Option<Py<PyAny>>,
    _marker: PhantomData<N>,
}

impl<N> PyHeuristic<N> {
    fn new(func: Option<Py<PyAny>>) -> Self {
        Self {
            func,
            _marker: PhantomData,
        }
    }
}

impl<N> Clone for PyHeuristic<N> {
    fn clone(&self) -> Self {
        // `Py<PyAny>` clones by bumping the refcount, which needs the GIL.
        Python::with_gil(|py| PyHeuristic::new(self.func.as_ref().map(|f| f.clone_ref(py))))
    }
}

impl<N: IntoPyNode + Clone> Heuristic<N> for PyHeuristic<N> {
    fn estimate(&self, node: &N, goal: &N) -> f64 {
        match &self.func {
            None => 0.0,
            Some(f) => Python::with_gil(|py| {
                let args = (node.clone().into_py_node(py), goal.clone().into_py_node(py));
                f.call1(py, args)
                    .and_then(|v| v.extract::<f64>(py))
                    .expect("the heuristic callable must return a float")
            }),
        }
    }
}

// ---------------------------------------------------------------------------
// Public Python functions
// ---------------------------------------------------------------------------

/// Resolve a `heuristic` argument that may be a built-in name (`str`) or a
/// custom Python callable `h(node, goal) -> float`. `None` falls back to
/// `default_name`.
enum HeuristicArg {
    Named(String),
    Custom(Py<PyAny>),
}

fn resolve_heuristic(
    heuristic: Option<Bound<'_, PyAny>>,
    default_name: &str,
) -> PyResult<HeuristicArg> {
    match heuristic {
        None => Ok(HeuristicArg::Named(default_name.to_string())),
        Some(obj) => {
            if let Ok(name) = obj.extract::<String>() {
                Ok(HeuristicArg::Named(name))
            } else if obj.is_callable() {
                Ok(HeuristicArg::Custom(obj.unbind()))
            } else {
                Err(value_err(
                    "heuristic must be a name (str) or a callable h(node, goal) -> float",
                ))
            }
        }
    }
}

/// Resolve the grid `heuristic` (name or callable) and run `algorithm`. Built-in
/// heuristics run with the GIL released; a custom callable keeps it.
#[allow(clippy::too_many_arguments)]
fn run_on_grid(
    py: Python<'_>,
    grid: GridGraph,
    start: Cell,
    goal: Cell,
    algorithm: &str,
    heuristic: Option<Bound<'_, PyAny>>,
    record: bool,
    opts: &RunOpts,
) -> PyResult<PySearchResult> {
    let r = match resolve_heuristic(heuristic, "manhattan")? {
        HeuristicArg::Named(name) => py.allow_threads(|| -> PyResult<SearchResult<Cell>> {
            match name.as_str() {
                "zero" => run_algo(&grid, start, goal, algorithm, Zero, record, opts),
                "manhattan" => run_algo(&grid, start, goal, algorithm, Manhattan, record, opts),
                "euclidean" => run_algo(&grid, start, goal, algorithm, Euclidean, record, opts),
                "octile" => run_algo(&grid, start, goal, algorithm, Octile, record, opts),
                other => Err(value_err(format!(
                    "unknown heuristic: '{other}'. Available: zero, manhattan, euclidean, \
                     octile, or a callable h(node, goal) -> float"
                ))),
            }
        })?,
        HeuristicArg::Custom(func) => {
            let h: PyHeuristic<Cell> = PyHeuristic::new(Some(func));
            run_algo(&grid, start, goal, algorithm, h, record, opts)?
        }
    };
    to_py_result(py, r)
}

/// Search a 2-D maze given as an ASCII map (`#` wall, `S` start, `G` goal, a
/// digit `1`–`9` = a free cell with that terrain cost).
///
/// `heuristic` is a built-in name (`"zero"`, `"manhattan"`, `"euclidean"`,
/// `"octile"`) or a custom callable `h((row, col), (row, col)) -> float`.
#[pyfunction]
#[pyo3(signature = (
    map, algorithm="astar", heuristic=None, record=true,
    weight=2.0, beam_width=None, depth_limit=None, max_nodes=None, diagonal=false
))]
#[allow(clippy::too_many_arguments)]
fn search_grid(
    py: Python<'_>,
    map: &str,
    algorithm: &str,
    heuristic: Option<Bound<'_, PyAny>>,
    record: bool,
    weight: f64,
    beam_width: Option<usize>,
    depth_limit: Option<usize>,
    max_nodes: Option<usize>,
    diagonal: bool,
) -> PyResult<PySearchResult> {
    if !map.contains('S') || !map.contains('G') {
        return Err(value_err(
            "the ASCII map must contain a start 'S' and a goal 'G'",
        ));
    }
    let (mut grid, start, goal) = GridGraph::from_ascii(map);
    if diagonal {
        grid = grid.with_diagonal(true);
    }
    let opts = RunOpts {
        weight,
        beam_width: beam_width.unwrap_or(usize::MAX),
        depth_limit,
        max_nodes,
    };
    run_on_grid(py, grid, start, goal, algorithm, heuristic, record, &opts)
}

/// Search a grid built from a matrix of **terrain costs**. `costs[r][c]` is the
/// movement cost of entering cell `(r, c)`; a non-positive or non-finite value
/// marks a wall. `start` and `goal` are `(row, col)` tuples. Same `heuristic`
/// options as [`search_grid`].
#[pyfunction]
#[pyo3(signature = (
    costs, start, goal, algorithm="astar", heuristic=None, record=true,
    weight=2.0, beam_width=None, depth_limit=None, max_nodes=None, diagonal=false
))]
#[allow(clippy::too_many_arguments)]
fn search_grid_costs(
    py: Python<'_>,
    costs: Vec<Vec<f64>>,
    start: (i32, i32),
    goal: (i32, i32),
    algorithm: &str,
    heuristic: Option<Bound<'_, PyAny>>,
    record: bool,
    weight: f64,
    beam_width: Option<usize>,
    depth_limit: Option<usize>,
    max_nodes: Option<usize>,
    diagonal: bool,
) -> PyResult<PySearchResult> {
    if costs.is_empty() {
        return Err(value_err("costs must be a non-empty matrix"));
    }
    let mut grid = GridGraph::from_costs(&costs);
    if diagonal {
        grid = grid.with_diagonal(true);
    }
    let start = Cell::new(start.0, start.1);
    let goal = Cell::new(goal.0, goal.1);
    if grid.is_blocked(start) || grid.is_blocked(goal) {
        return Err(value_err(
            "start and goal must be in-bounds, non-wall cells",
        ));
    }
    let opts = RunOpts {
        weight,
        beam_width: beam_width.unwrap_or(usize::MAX),
        depth_limit,
        max_nodes,
    };
    run_on_grid(py, grid, start, goal, algorithm, heuristic, record, &opts)
}

/// Search an explicit weighted graph given as an edge list over `0..num_nodes`.
///
/// `heuristic` defaults to the zero heuristic (so the informed algorithms behave
/// as their uninformed counterparts). Pass a custom callable
/// `h(node, goal) -> float` (nodes are ints) to run a real A\* / Greedy — e.g.
/// straight-line distance when your nodes have coordinates. `weight` sets the
/// `w` for `weighted_astar`.
#[pyfunction]
#[pyo3(signature = (
    num_nodes, edges, start, goal, algorithm="bfs", heuristic=None, undirected=true,
    record=true, weight=2.0, beam_width=None, depth_limit=None, max_nodes=None
))]
#[allow(clippy::too_many_arguments)]
fn search_graph(
    py: Python<'_>,
    num_nodes: usize,
    edges: Vec<(usize, usize, f64)>,
    start: usize,
    goal: usize,
    algorithm: &str,
    heuristic: Option<Bound<'_, PyAny>>,
    undirected: bool,
    record: bool,
    weight: f64,
    beam_width: Option<usize>,
    depth_limit: Option<usize>,
    max_nodes: Option<usize>,
) -> PyResult<PySearchResult> {
    if start >= num_nodes || goal >= num_nodes {
        return Err(value_err("start and goal must be < num_nodes"));
    }
    let graph = CsrGraph::from_edges(num_nodes, &edges, undirected);
    let opts = RunOpts {
        weight,
        beam_width: beam_width.unwrap_or(usize::MAX),
        depth_limit,
        max_nodes,
    };
    let r = match resolve_heuristic(heuristic, "zero")? {
        HeuristicArg::Named(name) => match name.as_str() {
            // Graph nodes are ids with no geometry, so the only built-in is zero.
            "zero" => {
                py.allow_threads(|| run_algo(&graph, start, goal, algorithm, Zero, record, &opts))?
            }
            other => {
                return Err(value_err(format!(
                    "unknown graph heuristic: '{other}'. Use 'zero' or a callable \
                     h(node, goal) -> float"
                )))
            }
        },
        HeuristicArg::Custom(func) => {
            let h: PyHeuristic<usize> = PyHeuristic::new(Some(func));
            run_algo(&graph, start, goal, algorithm, h, record, &opts)?
        }
    };
    to_py_result(py, r)
}

/// Search an implicit graph defined by a Python successor callable
/// `successors(state) -> [(state, cost), ...]`. States are ints or tuples of
/// ints. `heuristic`, if given, is a callable `h(state, goal) -> float`.
#[pyfunction]
#[pyo3(signature = (
    successors, start, goal, algorithm="astar", heuristic=None, record=true,
    weight=2.0, beam_width=None, depth_limit=None, max_nodes=None
))]
#[allow(clippy::too_many_arguments)]
fn search_implicit(
    py: Python<'_>,
    successors: Py<PyAny>,
    start: Bound<'_, PyAny>,
    goal: Bound<'_, PyAny>,
    algorithm: &str,
    heuristic: Option<Py<PyAny>>,
    record: bool,
    weight: f64,
    beam_width: Option<usize>,
    depth_limit: Option<usize>,
    max_nodes: Option<usize>,
) -> PyResult<PySearchResult> {
    let start_v = py_to_state(&start)?;
    let goal_v = py_to_state(&goal)?;
    let graph = PyImplicitGraph { successors };
    let h: PyHeuristic<Vec<i64>> = PyHeuristic::new(heuristic);
    let opts = RunOpts {
        weight,
        beam_width: beam_width.unwrap_or(usize::MAX),
        depth_limit,
        max_nodes,
    };
    // The search calls back into Python on every expansion, so we keep the GIL.
    let r = run_algo(&graph, start_v, goal_v, algorithm, h, record, &opts)?;
    to_py_result(py, r)
}

// --- instance generators ---------------------------------------------------

/// Unique undirected edges `(u, v, 1.0)` with `u < v`.
fn undirected_edges(graph: &CsrGraph) -> Vec<(usize, usize, f64)> {
    graph
        .edges()
        .into_iter()
        .filter(|&(u, v, _)| u < v)
        .collect()
}

/// Erdős–Rényi `G(n, p)` random graph → edge list.
#[pyfunction]
fn gen_erdos_renyi(n: usize, p: f64, seed: u64) -> Vec<(usize, usize, f64)> {
    undirected_edges(&erdos_renyi(n, p, seed))
}

/// Barabási–Albert scale-free graph → edge list.
#[pyfunction]
fn gen_barabasi_albert(n: usize, m: usize, seed: u64) -> Vec<(usize, usize, f64)> {
    undirected_edges(&barabasi_albert(n, m, seed))
}

/// Watts–Strogatz small-world graph → edge list.
#[pyfunction]
fn gen_watts_strogatz(n: usize, k: usize, beta: f64, seed: u64) -> Vec<(usize, usize, f64)> {
    undirected_edges(&watts_strogatz(n, k, beta, seed))
}

/// A named sample maze as an ASCII map. `name` is `"open"` or `"wall"`.
#[pyfunction]
fn sample_maze(name: &str) -> PyResult<String> {
    match name {
        "open" => Ok(SAMPLE_OPEN.to_string()),
        "wall" => Ok(SAMPLE_WALL.to_string()),
        other => Err(value_err(format!(
            "unknown sample maze: '{other}'. Available: open, wall"
        ))),
    }
}

/// A reproducible random maze rendered as an ASCII map (with `S`/`G`).
#[pyfunction]
fn random_maze_ascii(width: i32, height: i32, obstacle_density: f64, seed: u64) -> String {
    let m = random_maze(width, height, obstacle_density, seed);
    let mut out = String::new();
    for row in 0..height {
        for col in 0..width {
            let c = Cell::new(row, col);
            let ch = if c == m.start {
                'S'
            } else if c == m.goal {
                'G'
            } else if m.grid.is_blocked(c) {
                '#'
            } else {
                '.'
            };
            out.push(ch);
        }
        if row + 1 < height {
            out.push('\n');
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Relaxation / DP shortest paths (negative edges allowed)
// ---------------------------------------------------------------------------

/// Single-source shortest paths from [`bellman_ford_py`]. Thin wrapper over the
/// core `ShortestPaths`.
///
/// Attributes:
///     source (int): the node distances are measured from.
///     dist (list[float]): ``dist[v]`` is the cost of the cheapest
///         ``source -> v`` path (``inf`` if unreachable).
///     pred (list[int | None]): predecessor of each node on a shortest path
///         (``None`` for the source and unreachable nodes).
///     negative_cycle (bool): whether a negative-weight cycle is reachable from
///         ``source`` (then the affected distances are not well defined).
#[pyclass(name = "ShortestPaths")]
struct PyShortestPaths {
    inner: graphfinder_core::ShortestPaths,
}

#[pymethods]
impl PyShortestPaths {
    #[getter]
    fn source(&self) -> usize {
        self.inner.source
    }
    #[getter]
    fn dist(&self) -> Vec<f64> {
        self.inner.dist.clone()
    }
    #[getter]
    fn pred(&self) -> Vec<Option<usize>> {
        self.inner.pred.clone()
    }
    #[getter]
    fn negative_cycle(&self) -> bool {
        self.inner.negative_cycle
    }

    /// Rebuild the ``source -> target`` shortest path (inclusive), or ``None``
    /// if ``target`` is unreachable.
    fn path_to(&self, target: usize) -> Option<Vec<usize>> {
        self.inner.path_to(target)
    }

    fn __repr__(&self) -> String {
        format!(
            "ShortestPaths(source={}, nodes={}, negative_cycle={})",
            self.inner.source,
            self.inner.num_nodes(),
            self.inner.negative_cycle
        )
    }
}

/// All-pairs shortest paths from [`floyd_warshall_py`]. Thin wrapper over the
/// core `AllPairs`.
///
/// Attributes:
///     num_nodes (int): number of nodes ``n``.
///     negative_cycle (bool): whether the graph has any negative-weight cycle.
#[pyclass(name = "AllPairs")]
struct PyAllPairs {
    inner: graphfinder_core::AllPairs,
}

#[pymethods]
impl PyAllPairs {
    #[getter]
    fn num_nodes(&self) -> usize {
        self.inner.num_nodes()
    }
    #[getter]
    fn negative_cycle(&self) -> bool {
        self.inner.negative_cycle
    }

    /// Cost of the cheapest ``from_ -> to`` path (``inf`` if none, ``0`` if
    /// ``from_ == to``).
    fn distance(&self, from_: usize, to: usize) -> f64 {
        self.inner.distance(from_, to)
    }

    /// The full ``n x n`` distance matrix as a list of rows.
    fn matrix(&self) -> Vec<Vec<f64>> {
        self.inner.matrix()
    }

    /// Rebuild the cheapest ``from_ -> to`` path (inclusive), or ``None`` if
    /// there is none.
    fn path(&self, from_: usize, to: usize) -> Option<Vec<usize>> {
        self.inner.path(from_, to)
    }

    fn __repr__(&self) -> String {
        format!(
            "AllPairs(num_nodes={}, negative_cycle={})",
            self.inner.num_nodes(),
            self.inner.negative_cycle
        )
    }
}

/// **Bellman–Ford** single-source shortest paths on an explicit weighted graph.
///
/// Unlike Dijkstra/A\*, this tolerates **negative edge weights** and reports a
/// reachable negative cycle. Edges are ``(u, v, w)`` over ``0..num_nodes``.
///
/// ``undirected`` defaults to ``False``: negative weights are the whole point,
/// and an undirected negative edge is itself a trivial negative cycle.
#[pyfunction]
#[pyo3(name = "bellman_ford", signature = (num_nodes, edges, source, undirected=false))]
fn bellman_ford_py(
    py: Python<'_>,
    num_nodes: usize,
    edges: Vec<(usize, usize, f64)>,
    source: usize,
    undirected: bool,
) -> PyResult<PyShortestPaths> {
    if source >= num_nodes {
        return Err(value_err("source must be < num_nodes"));
    }
    let inner = py.allow_threads(|| {
        let graph = CsrGraph::from_edges(num_nodes, &edges, undirected);
        bellman_ford(&graph, source)
    });
    Ok(PyShortestPaths { inner })
}

/// **Floyd–Warshall** all-pairs shortest paths on an explicit weighted graph.
///
/// ``O(V³)`` — for small/medium or dense graphs where you want *every* distance
/// at once. Tolerates negative edges and flags any negative cycle. Edges are
/// ``(u, v, w)`` over ``0..num_nodes``; ``undirected`` defaults to ``False``.
#[pyfunction]
#[pyo3(name = "floyd_warshall", signature = (num_nodes, edges, undirected=false))]
fn floyd_warshall_py(
    py: Python<'_>,
    num_nodes: usize,
    edges: Vec<(usize, usize, f64)>,
    undirected: bool,
) -> PyResult<PyAllPairs> {
    let inner = py.allow_threads(|| {
        let graph = CsrGraph::from_edges(num_nodes, &edges, undirected);
        floyd_warshall(&graph)
    });
    Ok(PyAllPairs { inner })
}

#[pymodule]
fn graphfinder_native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(search_grid, m)?)?;
    m.add_function(wrap_pyfunction!(search_grid_costs, m)?)?;
    m.add_function(wrap_pyfunction!(search_graph, m)?)?;
    m.add_function(wrap_pyfunction!(search_implicit, m)?)?;
    m.add_function(wrap_pyfunction!(gen_erdos_renyi, m)?)?;
    m.add_function(wrap_pyfunction!(gen_barabasi_albert, m)?)?;
    m.add_function(wrap_pyfunction!(gen_watts_strogatz, m)?)?;
    m.add_function(wrap_pyfunction!(sample_maze, m)?)?;
    m.add_function(wrap_pyfunction!(random_maze_ascii, m)?)?;
    m.add_function(wrap_pyfunction!(bellman_ford_py, m)?)?;
    m.add_function(wrap_pyfunction!(floyd_warshall_py, m)?)?;
    m.add_class::<PySearchResult>()?;
    m.add_class::<PyShortestPaths>()?;
    m.add_class::<PyAllPairs>()?;
    Ok(())
}
