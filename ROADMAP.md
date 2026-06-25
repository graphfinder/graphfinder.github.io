# Roadmap

Check off each task as you progress. Phase 1 is done and verified.

## ✅ Phase 1 — Minimal functional core (DONE)
- [x] Cargo workspace + `graphfinder-core` crate
- [x] Central traits: `Graph`, `Frontier`, `Heuristic`
- [x] Single GENERAL-SEARCH loop (`search`) parameterized by `Algorithm`
- [x] Frontiers: `Fifo` (BFS), `Lifo` (DFS), `PriorityQueue` (UCS/Greedy/A*)
- [x] Algorithms: BFS, DFS, UCS/Dijkstra, Greedy, A*, Weighted A*
- [x] Domains: `GridGraph` (`Cell`, ASCII maps, 4/8-connected), `CsrGraph`
- [x] Heuristics: `Zero`, `Manhattan`, `Euclidean`, `Octile`
- [x] Maze instances + seeded random generator (`domains`)
- [x] `SearchResult` with path, cost, metrics and per-step `trace`
- [x] Examples: `basic` (A* on a maze), `compare` (all algorithms, table)
- [x] Tests: optimality, A*≤UCS, DFS-finds-a-path, no-path, trace, reproducibility

## ✅ Phase 2 — More algorithms & domains (DONE)
- [x] `IDDFS` (iterative deepening) and `DLS` (depth-limited) drivers
- [x] `IDA*` driver (iterative-deepening A*)
- [x] Bidirectional search (BFS, symmetric graphs)
- [x] Beam search (bounded frontier)
- [x] Node-expansion budget (`max_nodes`/`search_with`) → `StopReason::NodeLimit`
- [x] Random graph generators: Erdős–Rényi, Barabási–Albert, Watts–Strogatz
- [x] Per-algorithm comparison tests on shared instances + `strategies` example
- [ ] Bidirectional Dijkstra (weighted) — deferred
- [ ] Landmark / ALT heuristic for `CsrGraph` — deferred

## ✅ Phase 3 — Python API (Rust core via maturin) (DONE)
- [x] `crates/graph-py` PyO3 binding (`graphfinder_native`, abi3-py39)
- [x] `search_grid` / `search_graph` / `search_implicit` (native + Python callable)
- [x] `import graphfinder` + `search(...)` dispatcher; `SearchResult` with
      path, cost, metrics and per-step `trace`
- [x] Random generators + maze helpers exposed to Python
- [x] `examples/quickstart.py` + `tests/test_python.py` (pytest, 10 tests)

## ✅ Phase 4 — Visualization (top priority feature) (DONE)
- [x] `viz.animate_grid(map, result)` — the flagship "watch A* explore" animation
- [x] `viz.plot_grid(map, result)` — static snapshot (visited + path)
- [x] `viz.plot_graph(n, edges, result)` — general-graph layout, nodes by state
- [x] `viz.compare(results)` — bar charts: nodes-expanded (work) vs cost (quality)
- [x] `viz.plot_frontier(result)` — frontier-size-per-step curve
- [x] `examples/demo_viz.py` (writes `assets/`) + `tests/test_viz.py`
- [ ] `viz.plot_search_tree` — deferred (needs the parent map exposed)
- [ ] Notebooks: uninformed, informed, comparison, grid-animation — deferred

## ⬜ Phase 5 — Performance & scale
- [ ] Parallel multi-source BFS and all-pairs (rayon)
- [ ] Radix/bucket-heap Dijkstra for integer weights
- [ ] Implicit state-space domains: 8/15-puzzle, Towers of Hanoi, word-ladder
- [ ] Road-network loaders (DIMACS / OSM) + benchmarks
- [ ] Criterion benchmarks vs networkx / rustworkx baselines

## ⬜ Phase 6 — Advanced & publication
- [ ] Bellman-Ford (negative weights), Floyd-Warshall (all-pairs)
- [ ] D* Lite / LPA* (dynamic replanning)
- [ ] JOSS paper + docs site (mkdocs-material)
