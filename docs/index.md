# graphfinder

**Graph traversal & pathfinding** with a compute core in **Rust** and an API in
**Python**. Covers both **uninformed** search (BFS, DFS, UCS/Dijkstra) and
**informed** search (Greedy, A\*, Weighted A\*), with a focus on
**visualization, algorithm comparison and code clarity**. Built for teaching.

<p align="center">
  <img src="https://raw.githubusercontent.com/graphfinder/graphfinder.github.io/main/assets/astar_maze.gif" alt="A* exploring a maze" width="360">
</p>

## The one idea

Every algorithm here is the **same loop** (Russell & Norvig's GENERAL-SEARCH),
differing only in the **frontier** and the evaluation function
`priority = g_coeff·g(n) + h_coeff·h(n)`:

| Algorithm    | Frontier      | `g_coeff` | `h_coeff` | Optimal?            |
|--------------|---------------|-----------|-----------|---------------------|
| BFS          | FIFO queue    | 1 | 0 | only on unit costs  |
| DFS          | LIFO stack    | 1 | 0 | no                  |
| UCS/Dijkstra | min-priority  | 1 | 0 | yes                 |
| Greedy       | min-priority  | 0 | 1 | no                  |
| **A\***      | min-priority  | 1 | 1 | yes (admissible h)  |
| Weighted A\* | min-priority  | 1 | w | w-bounded           |

## Highlights

- **Uninformed:** BFS, DFS, UCS/Dijkstra, depth-limited, iterative deepening,
  bidirectional BFS.
- **Informed:** Greedy, A\*, Weighted A\*, IDA\*, beam search.
- **Domains:** grid/maze worlds, explicit weighted graphs (CSR), implicit
  graphs via a Python successor callable, and random-graph generators
  (Erdős–Rényi, Barabási–Albert, Watts–Strogatz).
- **Heuristics:** zero, Manhattan, Euclidean, octile.
- **Visualization** (`graphfinder.viz`): maze-search animation, static grid,
  frontier-size curve, work-vs-quality comparison, general-graph plots.
- **Instrumentation:** every run reports path, cost, nodes expanded/generated,
  peak frontier, stop reason and a per-step trace.
- **Reproducibility:** seeded generators and deterministic tie-breaking.

Start with the [installation](installation.md) and
[getting started](getting-started.md) pages.
