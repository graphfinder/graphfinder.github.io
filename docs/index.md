# graphfinder

**Graph traversal & pathfinding** with a compute core in **Rust** and an API in
**Python**. It covers both **uninformed** search (BFS, DFS, UCS/Dijkstra) and
**informed** search (Greedy, A\*, Weighted A\*, IDA\*, beam), and it is built for
**teaching**: every algorithm is the *same loop*, and you can *watch* it run.

<p align="center">
  <img src="/assets/astar.gif" alt="A* exploring a maze" width="380">
  <br>
  <em>A* exploring a maze — blue is the expanded frontier, gold is the final
  path. One line of Python: <code>gf.viz.animate_grid(maze, result)</code>.</em>
</p>

## Install

```bash
pip install graphfinder          # Python (prebuilt wheels)
cargo add graphfinder-core       # Rust crate
```

## Thirty seconds

```python
import graphfinder as gf

r = gf.search(gf.sample_maze("wall"), algorithm="astar", heuristic="manhattan")
print(r)        # SearchResult(found=True, cost=20, expanded=25, frontier=3, stop=goal)
```

## The one idea

Every algorithm here is the **same loop** — Russell & Norvig's GENERAL-SEARCH —
differing only in the **frontier** (which node comes out next) and the
evaluation function `priority = g_coeff·g(n) + h_coeff·h(n)`:

| Algorithm    | Frontier      | `g_coeff` | `h_coeff` | Optimal?            |
|--------------|---------------|-----------|-----------|---------------------|
| BFS          | FIFO queue    | 1 | 0 | only on unit costs  |
| DFS          | LIFO stack    | 1 | 0 | no                  |
| UCS/Dijkstra | min-priority  | 1 | 0 | yes                 |
| Greedy       | min-priority  | 0 | 1 | no                  |
| **A\***      | min-priority  | 1 | 1 | yes (admissible h)  |
| Weighted A\* | min-priority  | 1 | w | within `w×` optimal |

Run the **same maze** through each and the difference is immediate:

<p align="center">
  <img src="/assets/algorithms_grid.png" alt="Six algorithms on the same maze" width="820">
</p>

BFS and UCS flood the whole map; DFS wanders to a long path; Greedy beelines but
overshoots the optimum; **A\* gets the optimal cost while exploring a fraction of
what UCS does**; Weighted A\* trades a little optimality for even less work.

## What's inside

- **Uninformed:** BFS, DFS, UCS/Dijkstra, depth-limited (DLS), iterative
  deepening (IDDFS), bidirectional BFS.
- **Informed:** Greedy best-first, A\*, Weighted A\*, IDA\*, beam search.
- **[Negative weights](shortest-paths.md):** Bellman–Ford (single-source, with
  negative-cycle detection) and Floyd–Warshall (all-pairs) — for the graphs
  Dijkstra/A\* can't handle.
- **Domains:** [grid/maze worlds](domains.md), explicit weighted graphs (CSR),
  implicit graphs via a Python successor callable, and random-graph generators
  (Erdős–Rényi, Barabási–Albert, Watts–Strogatz).
- **Heuristics:** [zero, Manhattan, Euclidean, octile](heuristics.md).
- **[Visualization](visualization.md):** maze-search animation, static grids,
  frontier-size curves, work-vs-quality comparison, and general-graph plots.
- **Instrumentation:** every run reports the path, cost, nodes expanded/
  generated, peak frontier, stop reason, and a per-step trace.
- **[Integrations](integrations.md):** **NetworkX**, **SciPy** sparse adjacency,
  **pandas** DataFrames, **OSMnx** road networks, a safe **LangChain** routing
  tool, a **Gymnasium** GridWorld env with an A\* oracle, and **Graphviz** export.
- **Reproducibility:** seeded generators and deterministic tie-breaking.

## Where to next

<div class="grid cards" markdown>

- 🚀 **[Installation](installation.md)** — pip, cargo, or build from source.
- 🧭 **[Getting started](getting-started.md)** — mazes, graphs, implicit search.
- 📚 **Tutorials** — [grids](tutorials/grid.md) ·
  [comparison](tutorials/comparison.md) · [graphs](tutorials/graphs.md).
- 🧠 **Concepts** — [algorithms](algorithms.md) ·
  [shortest paths](shortest-paths.md) · [heuristics](heuristics.md) ·
  [domains](domains.md).
- 🎬 **[Visualization gallery](visualization.md)**.
- 🔌 **[Integrations](integrations.md)** — NetworkX · SciPy · pandas.
- 🛠️ **[Design & internals](architecture.md)** and the
  **[API reference](api.md)**.

</div>

## How is this different from networkx / rustworkx?

Those are excellent general graph libraries. graphfinder's niche is **search
pedagogy, step-by-step visualization and algorithm comparison**, with a fast
Rust core and GIL-free implicit state-space search (puzzles, large grids) as
supporting features rather than the headline.

## Author

graphfinder is created and maintained by **[Jose L. Salmeron](https://salmeron.cunef.edu)**
([ORCID](https://orcid.org/0000-0001-7811-3716)), CUNEF Universidad.

## Citing

If you use graphfinder in academic work, please cite it — see the `CITATION.cff`
in the repository. Licensed under MIT © Jose L. Salmeron.
