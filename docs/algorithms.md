# Algorithms

All algorithms share one loop; they differ in the **frontier** and the
evaluation function. Select one with `algorithm=` (and, for the priority-based
ones, a `heuristic=`).

## Uninformed

| `algorithm` | Idea | Complete | Optimal | Memory |
|-------------|------|----------|---------|--------|
| `bfs`       | FIFO frontier, expands by depth | yes | unit costs only | high |
| `dfs`       | LIFO frontier, dives deep first | on finite graphs | no | low |
| `ucs` / `dijkstra` | expand by cost-so-far `g(n)` | yes | yes | high |
| `dls`       | DFS cut off at `depth_limit` | within the limit | no | low |
| `iddfs`     | DLS with growing limit | yes | unit costs only | low |
| `bidirectional` | two BFS frontiers meet in the middle | yes (symmetric) | unit costs | medium |

## Informed (need a heuristic)

| `algorithm` | Priority | Complete | Optimal |
|-------------|----------|----------|---------|
| `greedy`        | `h(n)`            | no  | no |
| `astar`         | `g(n) + h(n)`     | yes | yes (admissible `h`) |
| `weighted_astar`| `g(n) + w·h(n)`   | yes | within `w×` optimal |
| `ida_star`      | iterative deepening on `g + h` | yes | yes (admissible `h`) |
| `beam`          | best `beam_width` by `h` per level | no | no |

## Heuristics (grids)

- `zero` — uninformed; turns A\* into UCS and Greedy into a blind expansion.
- `manhattan` — `|Δrow| + |Δcol|`; admissible on a 4-connected unit grid.
- `euclidean` — straight-line distance.
- `octile` — exact distance on an 8-connected grid (use with `diagonal=True`).

## Parameters

- `weight` — the `w` for `weighted_astar` (default 2.0).
- `beam_width` — frontier cap for `beam`.
- `depth_limit` — required by `dls`; caps `iddfs`.
- `max_nodes` — expansion budget; stops early with `stop_reason="node_limit"`.
- `record` — keep the per-step `trace` (needed for visualization).

## What changes between them

The named algorithms are constructors over three knobs — see the table on the
[home page](index.md). Greedy expands fewest nodes but may return a costly path;
BFS/UCS are optimal but explore widely; A\* gets the optimal cost while expanding
far less than UCS. Use `graphfinder.viz.compare` to see this on your own problem.
