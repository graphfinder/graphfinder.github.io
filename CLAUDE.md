# CLAUDE.md

Guide for working in this repository with Claude Code.

## What this project is

A general-purpose **graph traversal & pathfinding** library with a **compute
core in Rust** and (planned) a **Python API**, sibling to `turboswarm`. It
covers both **uninformed** search (BFS, DFS, UCS/Dijkstra) and **informed**
search (Greedy, A*, Weighted A*). It is built for **teaching**, so worked
examples and clear, comparable behaviour are first-class.

Priorities, in order: 1) **visualization/animation**, 2) **algorithm
comparison**, 3) **code clarity**, 4) **performance**.

It is designed as an **extensible framework**: a new algorithm = a `Frontier`
choice + an evaluation function; a new domain = an impl of `Graph`; a new
informed strategy = an impl of `Heuristic`. The search loop never changes.

## The central design (read before touching the core)

The search loop (`crates/graphfinder-core/src/search.rs`) implements Russell &
Norvig's GENERAL-SEARCH **once** and knows nothing about any concrete
algorithm. Everything that distinguishes the algorithms lives behind three
traits in `crates/graphfinder-core/src/traits.rs`:

- `Graph` — the domain. `neighbors(node) -> Vec<(node, cost)>`. Implementations:
  `GridGraph` (2-D maze world, geometric, node = `Cell`), `CsrGraph` (explicit
  weighted graph, node = `usize`). Implicit state-space graphs (puzzles) come
  in a later phase but fit the same trait.
- `Frontier` — the open list. **The single choice that names the algorithm:**
  `Fifo` → BFS, `Lifo` → DFS, `PriorityQueue` → UCS/Greedy/A*. Template:
  `frontier/mod.rs`.
- `Heuristic` — `h(n)`. `Zero` (uninformed, any domain); `Manhattan`,
  `Euclidean`, `Octile` (grid). Template: `heuristic.rs`.

An [`Algorithm`] bundles `(frontier_kind, g_coeff, h_coeff, name)`. The priority
pushed is `g_coeff·g(n) + h_coeff·h(n)`. That table **is** the pedagogy:

| Algorithm | Frontier | g_coeff | h_coeff |
|-----------|----------|---------|---------|
| BFS       | FIFO     | 1 | 0 (priority unused) |
| DFS       | LIFO     | 1 | 0 (priority unused) |
| UCS       | Priority | 1 | 0 |
| Greedy    | Priority | 0 | 1 |
| A*        | Priority | 1 | 1 |
| Weighted A* | Priority | 1 | w |

**Invariant:** edge costs are finite and non-negative (Dijkstra/A* assume it).

## Architecture

```
crates/graphfinder-core/   Rust core. No FFI deps. Traits + loop + domains.
  src/traits.rs            Graph, Frontier, Heuristic.
  src/search.rs            The single GENERAL-SEARCH loop + Algorithm + SearchResult.
  src/frontier/            Fifo, Lifo, PriorityQueue.
  src/graph/               GridGraph (Cell), CsrGraph.
  src/heuristic.rs         Zero, Manhattan, Euclidean, Octile.
  src/strategies.rs        Drivers: dls, iddfs, ida_star, beam_search, bidirectional.
  src/domains/             Maze + random-graph generators (benchmarks).
  examples/                basic.rs, compare.rs, strategies.rs.
  tests/                   optimality.rs, strategies.rs.
crates/graph-py/           PyO3 binding (native module `graphfinder_native`).
python/graphfinder/        Python API: __init__ (dispatcher), viz (matplotlib).
examples/quickstart.py     Python quickstart.
examples/demo_viz.py       Generates assets/ (GIF + PNGs).
tests/                     test_python.py (binding), test_viz.py (viz, Agg).
```

## Commands (verified)

### Rust
```bash
cargo test -p graphfinder-core                 # tests + doctests
cargo run --example basic      -p graphfinder-core
cargo run --example compare    -p graphfinder-core
cargo run --example strategies -p graphfinder-core
cargo clippy -p graphfinder-core --all-targets
```

### Python (Rust core via maturin)
```bash
python -m venv .venv && source .venv/bin/activate
pip install maturin pytest
maturin develop --release        # compiles the Rust core and installs it
python examples/quickstart.py
pip install matplotlib pillow networkx   # viz extras
python examples/demo_viz.py      # writes assets/astar_maze.gif + PNGs
pytest -q                        # binding + viz tests
```

`cargo build -p graph-py` fails to link by design (the `extension-module`
cdylib resolves Python symbols at load time) — always build the binding via
`maturin develop`. After ANY change in `graphfinder-core` or `graph-py`, re-run
`maturin develop` so Python sees it.

## Conventions

- **Language:** all comments, docs and identifiers are in **English** (published
  internationally). Prose in issues/PRs may be Spanish.
- **Teaching first:** every algorithm and domain ships a runnable example and a
  test that asserts its defining property (BFS/UCS/A* optimal, A* expands ≤ UCS,
  DFS finds *a* path, …). Add both when you add a feature.
- **Reproducibility:** every randomized instance takes a `seed` (ChaCha8). The
  priority queue breaks ties deterministically (FIFO on insertion order).
- **Instrumentation:** `record=true` keeps the per-step `trace` that drives
  visualization; results always carry `nodes_expanded`, `nodes_generated`,
  `max_frontier_size`, `stop_reason`.

## Releasing

Tag-driven. Bump the version in `Cargo.toml`, `pyproject.toml` and
`CITATION.cff`, commit, then `git tag vX.Y.Z && git push origin main vX.Y.Z`.
That fires `release-pypi`, `release-crates` and `release-github`. Both package
publishes use **Trusted Publishing (OIDC)** — no tokens stored in the repo. See
the "Releasing a new version" section in `CONTRIBUTING.md` for the full
checklist and the one-time publisher setup.

## How to extend (typical tasks)

### Add an algorithm
Usually no new code in the loop — add a constructor on `Algorithm` (e.g. beam,
weighted variants). If it needs a new frontier discipline, add a type in
`frontier/` implementing `Frontier`, plus a `FrontierKind` arm in `search.rs`.
IDDFS/IDA*/bidirectional need their own thin driver around `search` — keep the
inner loop intact. Add an example + a test.

### Add a domain
Implement `Graph` for the new type in `graph/`, export it, add a domain/instance
in `domains/` with known structure, and a test.

### Add a heuristic
Implement `Heuristic<N>` in `heuristic.rs` for the relevant node type; add a
test that it stays admissible on a known instance.

## Status by phase

- ✅ **Phase 1** — Rust core: `Graph`/`Frontier`/`Heuristic` traits; the single
  GENERAL-SEARCH loop; BFS, DFS, UCS/Dijkstra, Greedy, A*, Weighted A*;
  `GridGraph` + `CsrGraph`; `Zero`/`Manhattan`/`Euclidean`/`Octile`; maze
  domains + seeded generator; `SearchResult` with trace + metrics; examples
  (`basic`, `compare`) and optimality/reproducibility tests.
- ✅ **Phase 2** — extra strategies with their own driver (`strategies.rs`):
  `dls`, `iddfs`, `ida_star`, `beam_search`, `bidirectional`; node budget
  (`search_with`/`max_nodes` → `StopReason::NodeLimit`); random-graph generators
  (`erdos_renyi`, `barabasi_albert`, `watts_strogatz`); `strategies` example +
  tests. Deferred: bidirectional Dijkstra, landmark/ALT heuristic.
- ✅ **Phase 3** — PyO3 binding (`graph-py` → `graphfinder_native`) + Python API
  (`import graphfinder`). Three entry points: `search_grid` (ASCII maze),
  `search_graph` (edge list), `search_implicit` (Python successor callable,
  states = int/tuple-of-int, GIL-reacquired per expansion; native domains run
  with the GIL released). `SearchResult` exposes path/cost/metrics/trace; random
  generators + maze helpers exposed; `examples/quickstart.py` + pytest suite.
- ✅ **Phase 4** — `graphfinder.viz` (matplotlib, lazy imports): `animate_grid`
  (flagship GIF), `plot_grid`, `plot_frontier`, `compare`, `plot_graph`
  (networkx layout if available, else circular). `examples/demo_viz.py` writes
  `assets/`; `tests/test_viz.py` (Agg). Deferred: `plot_search_tree` (needs the
  core to expose the parent map, not just the path) and notebooks.
- ⬜ **Phase 5** — Performance/scale: parallel multi-source BFS & all-pairs
  (rayon), bidirectional/radix-heap Dijkstra, implicit puzzles (8/15-puzzle,
  Hanoi, word-ladder), road networks (DIMACS/OSM).
- ⬜ **Phase 6** — Bellman-Ford, Floyd-Warshall, D* Lite/LPA* (replanning);
  JOSS paper.

See `ROADMAP.md` for the checkable task breakdown.

## Known pitfalls

- The priority queue may hold stale duplicates; the loop skips already-closed
  nodes on pop. Don't "optimise" that check away.
- DFS is intentionally non-optimal — don't assert optimal cost for it.
- A* ≤ UCS in expansions holds only for **admissible** heuristics (use `Octile`,
  not `Manhattan`, on a diagonal grid).
- Keep the doctests in `lib.rs`, `search.rs` and `grid.rs` valid (`cargo test`
  runs them).
