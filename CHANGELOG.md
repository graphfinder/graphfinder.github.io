# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/), and the project adheres to
[Semantic Versioning](https://semver.org/).

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

[0.3.0]: https://github.com/graphfinder/graphfinder.github.io/releases/tag/v0.3.0
[0.2.1]: https://github.com/graphfinder/graphfinder.github.io/releases/tag/v0.2.1
[0.2.0]: https://github.com/graphfinder/graphfinder.github.io/releases/tag/v0.2.0
[0.1.0]: https://github.com/graphfinder/graphfinder.github.io/releases/tag/v0.1.0
