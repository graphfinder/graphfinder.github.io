# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/), and the project adheres to
[Semantic Versioning](https://semver.org/).

## [0.11.0] — 2026-06-28

### Added
- **Search-tree visualization** (`graphfinder.viz.plot_search_tree`): draws the
  tree of best-parent links discovered during the search, rooted at the start
  with the solution path in gold — for grids, graphs *and* puzzles. A tidy
  layered layout, matplotlib-only (no extra dependency). New
  [Visualization](https://graphfinder.github.io/visualization/) section + asset.
- **`SearchResult.tree`** — the core now records the search tree as
  `(parent, child)` edges (Rust `SearchResult.tree`, Python `result.tree`),
  gated by `record` like `trace`. Populated by the main algorithms (`bfs`,
  `dfs`, `ucs`, `greedy`, `astar`, `weighted_astar`); empty for the
  iterative-deepening and bidirectional drivers, which keep no parent map.

## [0.10.0] — 2026-06-28

### Added
- **Implicit puzzle domains** — classic state-space problems generated on demand
  through the same `Graph` trait, so every algorithm runs on them unchanged and
  natively (GIL released for the built-in heuristics):
  - **Sliding-tile N-puzzle** (`search_npuzzle`; Rust `puzzles::NPuzzle`):
    8-puzzle, 15-puzzle, … with the `manhattan`/`misplaced` admissible
    heuristics and an `is_solvable` parity check (unsolvable starts raise).
  - **Towers of Hanoi** (`search_hanoi`; `puzzles::Hanoi`): 3+ pegs, with the
    `misplaced`-disks heuristic; the 3-peg optimum is `2^disks − 1`.
  - **Word ladder** (`search_wordladder`; `puzzles::WordLadder`): one-letter
    steps over a dictionary, with the `hamming` heuristic.
  - All accept the usual `algorithm=` / `heuristic=` (built-in name, `"zero"`,
    or a custom callable) / `max_nodes=` knobs. Rust example `puzzles` + tests;
    Python tests; new [Implicit puzzles](https://graphfinder.github.io/puzzles/)
    docs page.

## [0.9.0] — 2026-06-27

### Added
- **Negative-weight shortest paths** — two relaxation/DP algorithms that work
  where Dijkstra/A\* cannot (Rust `graphfinder_core::shortest_paths`, Python
  `graphfinder`):
  - **Bellman–Ford** (`bellman_ford`): single-source shortest paths over an
    explicit weighted graph, tolerant of **negative edge weights** and able to
    **detect a reachable negative cycle**. Returns a `ShortestPaths` with
    `dist`, `pred`, `negative_cycle` and a `path_to(target)` reconstructor.
  - **Floyd–Warshall** (`floyd_warshall`): all-pairs shortest paths in `O(V³)`,
    also negative-edge-aware. Returns an `AllPairs` with `distance(i, j)`,
    `path(i, j)`, `matrix()` and `negative_cycle`.
  - Rust example `shortest_paths` + tests; Python tests; new
    [Shortest paths](https://graphfinder.github.io/shortest-paths/) docs page.

### Notes
- These two relax the library-wide non-negative-edge invariant **on purpose**;
  the frontier algorithms (Dijkstra/A\*) still assume non-negative costs. Both
  default to **directed** edges (`undirected=False`), since an undirected
  negative edge is itself a trivial negative cycle.

## [0.8.0] — 2026-06-27

### Added
- **Gymnasium** (`graphfinder.integrations.gym`): `GridWorldEnv`, a Gymnasium RL
  environment over a graphfinder grid (walls, terrain costs, 4/8 actions), plus
  an **A\* oracle** (`optimal_path` / `optimal_action`) for imitation learning,
  reward shaping or scoring an agent. `[gym]` extra.
- **Graphviz** (`graphfinder.integrations.graphviz`): `to_dot` exports an
  edge-list graph and a found path to DOT (no dependency); `source` returns a
  renderable `graphviz.Source`. `[graphviz]` extra.

## [0.7.0] — 2026-06-26

### Removed
- The **PyTorch learned-heuristic** integration (`graphfinder.integrations.torch`,
  the `[torch]` extra, the tutorial and example) shipped in 0.6.0. A learned
  heuristic is inadmissible — it forfeits A\*'s optimality guarantee — and the
  bridge was a trivial wrapper pulling in a heavy dependency, so it did not earn
  a place as a first-class integration. The core **custom-heuristic hook**
  (`heuristic=<callable>`) is unchanged: you can still pass any estimator,
  learned or not — see [Heuristics](https://graphfinder.github.io/heuristics/).

## [0.6.0] — 2026-06-26

### Added
- **PyTorch / learned heuristics** (`graphfinder.integrations.torch`):
  `as_heuristic(model, encode)` wraps any model (PyTorch, NumPy, scikit-learn)
  into a custom A\* heuristic. New tutorial + example training an MLP that beats
  Manhattan on weighted terrain (`examples/learned_heuristic.py`).
- **Agents / LangChain** (`graphfinder.integrations.agents`): `make_router`
  builds a safe, dependency-free bound router (validates input, caps `max_nodes`,
  allow-listed algorithms, never raises); `as_langchain_tool` wraps it as a
  LangChain `StructuredTool` (`[agents]` extra).

## [0.5.0] — 2026-06-26

### Added
- **OSMnx / geographic routing** (`graphfinder.integrations.osm`):
  - `search` runs A\* between nodes of a geographic networkx graph (nodes with
    `x`/`y` lon-lat) using a great-circle **haversine** heuristic — needs only
    `networkx`, and is admissible when edge `length` is in metres.
  - `route` (snap two lat/lon points to nearest nodes) and `plot_route` are
    OSMnx convenience wrappers (`[osm]` extra).
  - `haversine(lat1, lon1, lat2, lon2)` helper (metres).

## [0.4.0] — 2026-06-26

### Added
- **Ecosystem integrations** under `graphfinder.integrations` (lazy imports,
  optional extras), each returning a `LabeledResult` that maps the path back to
  your node labels and keeps the native result in `.raw`:
  - **NetworkX** (`[networkx]`): `search` over `nx.Graph`/`DiGraph` — a drop-in
    alternative to `nx.shortest_path`/`astar_path` with search instrumentation.
  - **SciPy** (`[scipy]`): `search` over a `scipy.sparse` adjacency matrix,
    matching `scipy.sparse.csgraph`'s `directed` convention.
  - **pandas** (`[pandas]`): `search` from an edge-list `DataFrame`, plus
    `trace_dataframe` and `compare_dataframe` result tables.
- CI now installs the integration extras so their tests run (not skip).

## [0.3.0] — 2026-06-26

### Added
- **Weighted grids / terrain costs.** Every free `GridGraph` cell now carries a
  movement cost (default `1.0`); entering a cell costs its terrain (× `√2`
  diagonally). On a weighted grid Dijkstra/A\* genuinely differ from BFS — the
  cheapest path is no longer the one with the fewest steps.
  - ASCII maps accept digits `1`–`9` as per-cell terrain costs.
  - New `search_grid_costs(costs, start, goal, …)` builds a grid from a matrix of
    costs (a cell `≤ 0` or non-finite is a wall).
  - Rust: `GridGraph::from_costs`, `set_cost`, `cost_at`, `is_weighted`.
- **Visualization of terrain.** `viz.plot_costs` draws a terrain heatmap;
  `plot_grid`/`animate_grid` now shade the terrain underneath the search and
  accept either an ASCII map or a cost matrix.

### Notes
- The geometric heuristics stay admissible when terrain costs ≥ 1 (the usual
  case); below 1, use `zero` or a min-cost-scaled heuristic.

## [0.2.1] — 2026-06-26

### Changed
- CI/release workflows moved to the Node 24 action majors (`cache@v6`,
  `upload-artifact@v7`, `download-artifact@v8`, `deploy-pages@v5`,
  `upload-pages-artifact@v5`), removing the Node 20 deprecation warnings.
  Maintenance only — no library or API changes.

## [0.2.0] — 2026-06-26

### Added
- **Custom heuristics.** `heuristic=` now accepts a Python callable
  `h(node, goal) -> float` in addition to the built-in names, across **all**
  domains: grids (`(row, col)` nodes), explicit graphs (integer nodes) and
  implicit graphs. Works with every priority-based algorithm (`greedy`,
  `astar`, `weighted_astar`, `ida_star`, `beam`).
- `search_graph` gained `heuristic=` and `weight=` parameters, so A\* / Greedy /
  Weighted A\* can run on explicit graphs with a user-supplied estimate.

### Changed
- The PyO3 binding now shares one generic `PyHeuristic<N>` adapter for every
  domain. Built-in heuristics still run with the GIL released; a custom callable
  reacquires the GIL once per node it scores.
- Documentation site expanded with per-domain custom-heuristic examples.

## [0.1.0] — 2026-06-25

### Added
- Rust core (`graphfinder-core`): a single GENERAL-SEARCH loop parameterized by a
  `Frontier` and an evaluation function. Algorithms: BFS, DFS, UCS/Dijkstra,
  Greedy, A\*, Weighted A\*, plus DLS, IDDFS, IDA\*, beam and bidirectional
  drivers.
- Domains: grid/maze worlds, explicit CSR graphs, and seeded random-graph
  generators (Erdős–Rényi, Barabási–Albert, Watts–Strogatz). Heuristics: zero,
  Manhattan, Euclidean, octile.
- Python API (`graphfinder`) via PyO3/maturin: `search_grid`, `search_graph`,
  `search_implicit` (Python successor callable), with a `search` dispatcher.
- Visualization (`graphfinder.viz`): `animate_grid`, `plot_grid`,
  `plot_frontier`, `compare`, `plot_graph`.
- Published to PyPI (`graphfinder`) and crates.io (`graphfinder-core`); docs at
  <https://graphfinder.github.io>.

[0.11.0]: https://github.com/graphfinder/graphfinder.github.io/releases/tag/v0.11.0
[0.10.0]: https://github.com/graphfinder/graphfinder.github.io/releases/tag/v0.10.0
[0.9.0]: https://github.com/graphfinder/graphfinder.github.io/releases/tag/v0.9.0
[0.8.0]: https://github.com/graphfinder/graphfinder.github.io/releases/tag/v0.8.0
[0.7.0]: https://github.com/graphfinder/graphfinder.github.io/releases/tag/v0.7.0
[0.6.0]: https://github.com/graphfinder/graphfinder.github.io/releases/tag/v0.6.0
[0.5.0]: https://github.com/graphfinder/graphfinder.github.io/releases/tag/v0.5.0
[0.4.0]: https://github.com/graphfinder/graphfinder.github.io/releases/tag/v0.4.0
[0.3.0]: https://github.com/graphfinder/graphfinder.github.io/releases/tag/v0.3.0
[0.2.1]: https://github.com/graphfinder/graphfinder.github.io/releases/tag/v0.2.1
[0.2.0]: https://github.com/graphfinder/graphfinder.github.io/releases/tag/v0.2.0
[0.1.0]: https://github.com/graphfinder/graphfinder.github.io/releases/tag/v0.1.0
