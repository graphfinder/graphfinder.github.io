# Design & internals

graphfinder is a small Rust core with a thin Python binding. Its guiding
principle: **the search loop knows nothing about any concrete algorithm.**
Everything that distinguishes BFS from A\* lives behind three traits.

## Layout

```
crates/graphfinder-core/   Rust core — traits, the loop, domains, strategies
  src/traits.rs            Graph, Frontier, Heuristic
  src/search.rs            the single GENERAL-SEARCH loop + Algorithm + SearchResult
  src/frontier/            Fifo (BFS), Lifo (DFS), PriorityQueue (UCS/Greedy/A*)
  src/graph/               GridGraph (Cell), CsrGraph
  src/heuristic.rs         Zero, Manhattan, Euclidean, Octile
  src/strategies.rs        dls, iddfs, ida_star, beam_search, bidirectional
  src/domains/             maze + random-graph generators
crates/graph-py/           PyO3 binding → the native module graphfinder_native
python/graphfinder/        Python API (dispatcher) + viz (matplotlib)
```

## The three traits

```rust
pub trait Graph {
    type Node: Clone + Eq + Hash;
    fn neighbors(&self, node: &Self::Node) -> Vec<(Self::Node, f64)>;
}

pub trait Heuristic<N> {
    fn estimate(&self, node: &N, goal: &N) -> f64;
}

pub trait Frontier<N> {
    fn push(&mut self, node: N, priority: f64);
    fn pop(&mut self) -> Option<N>;
    fn len(&self) -> usize;
}
```

- **`Graph`** is the domain. Implement `neighbors` and every algorithm works on
  your type — grids, CSR graphs, implicit state spaces.
- **`Heuristic`** is `h(n)`. `Zero` works for any node type; the geometric ones
  are defined for grid `Cell`s.
- **`Frontier`** is the open list, and *the single choice that names the
  algorithm*: FIFO → BFS, LIFO → DFS, min-priority → UCS/Greedy/A\*.

## One loop to rule them all

An [`Algorithm`](algorithms.md) bundles `(frontier_kind, g_coeff, h_coeff)`. The
loop pushes `priority = g_coeff·g(n) + h_coeff·h(n)` and lets the frontier decide
order. That table *is* the library:

| Algorithm | Frontier | g | h |
|-----------|----------|---|---|
| BFS | FIFO | 1 | 0 |
| DFS | LIFO | 1 | 0 |
| UCS | Priority | 1 | 0 |
| Greedy | Priority | 0 | 1 |
| A\* | Priority | 1 | 1 |
| Weighted A\* | Priority | 1 | w |

Iterative-deepening and bidirectional search need their own thin driver around
the same primitives — see `src/strategies.rs` — but never touch the inner loop.

## The Rust ↔ Python boundary

The binding (`crates/graph-py`) exposes three entry points:

- `search_grid` / `search_graph` run **native** domains with the GIL **released**
  (`py.allow_threads`) for full speed.
- `search_implicit` wraps a Python successor callable in a `Graph` impl that
  reacquires the GIL once per expansion — the same pattern lets you bring an
  arbitrary state space while the loop stays in Rust.

Results carry the path, metrics and the per-step trace back across the boundary
as plain Python objects.

## Instrumentation by default

Every run records `nodes_expanded`, `nodes_generated`, `max_frontier_size`,
`stop_reason`, and (with `record=True`) a per-step `trace`. Visualization is a
first-class goal, so the trace is the contract the `viz` layer builds on.

## Reproducibility

Random instances take a `seed` (ChaCha8 RNG). The priority queue breaks ties
deterministically (FIFO on insertion order), so the same inputs always produce
the same expansions — the tests depend on it.

## Extending

- **New algorithm** → usually just a constructor on `Algorithm`; a genuinely new
  discipline (e.g. iterative deepening) gets a driver in `strategies.rs`.
- **New domain** → `impl Graph`.
- **New heuristic** → `impl Heuristic<N>`.

Each addition ships a runnable example and a test asserting its defining
property. See `CONTRIBUTING.md` and the [API reference](api.md).
