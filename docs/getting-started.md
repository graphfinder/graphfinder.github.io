# Getting started

This page walks through the three ways graphfinder describes a problem — a
**grid/maze**, an **explicit graph**, and an **implicit graph** — and how to read
the result. For deeper walk-throughs see the tutorials:
[grid pathfinding](tutorials/grid.md), [comparing algorithms](tutorials/comparison.md)
and [random & implicit graphs](tutorials/graphs.md).

## The result object

Every search returns a [`SearchResult`](api.md):

```python
import graphfinder as gf

r = gf.search(gf.sample_maze("wall"), algorithm="astar", heuristic="manhattan")

r.found              # True
r.path               # [(0,0), (1,0), ...] list of (row, col) cells
r.cost               # 20.0
r.nodes_expanded     # 25   — taken off the frontier
r.nodes_generated    # 45   — ever pushed
r.max_frontier_size  # 3    — peak memory
r.stop_reason        # "goal" | "exhausted" | "node_limit"
r.trace              # [(node, g, frontier_size), ...] per expansion (if record=True)
```

The `trace` is what powers the visualizations: replay `node` in order to see the
frontier grow.

## 1. Grids / mazes

A maze is an ASCII map: `#` is a wall, `S` the start, `G` the goal, anything else
is free.

```python
maze = """
S.........
.####.###.
.#..#.#.#.
.#.##.#.#.
.#....#..G
""".strip()

r = gf.search(maze, algorithm="astar", heuristic="manhattan", record=True)
```

Helpers give you ready-made mazes:

```python
gf.sample_maze("open")                       # a small open room
gf.sample_maze("wall")                        # a corridor maze
gf.random_maze_ascii(25, 25, 0.25, seed=0)    # reproducible random maze
```

Grids can be 8-connected (diagonal moves) — pair that with the `octile`
heuristic:

```python
gf.search(maze, algorithm="astar", heuristic="octile", diagonal=True)
```

## 2. Explicit graphs

Pass an edge list over nodes `0..n`. The generators return exactly this format.

```python
edges = gf.gen_barabasi_albert(300, 3, seed=7)        # scale-free graph
r = gf.search_graph(300, edges, start=0, goal=299, algorithm="bidirectional")
```

Graphs use integer node ids, so the heuristic is `zero` (informed names degrade
to their uninformed behaviour); for weighted shortest paths use `ucs`/`dijkstra`.

## 3. Implicit graphs (lazy successors)

When the graph is too big to materialize — or generated on the fly — supply a
**successor function**. States are ints or tuples of ints.

```python
# Reach 27 from 1 using +1 and *2 — BFS finds the fewest operations.
def successors(s):
    return [(s + 1, 1.0), (s * 2, 1.0)] if s < 100 else []

r = gf.search(successors, start=1, goal=27, algorithm="bfs")
print(r.path)   # [1, 2, 3, 6, 12, 13, 26, 27]
```

Add a heuristic callable `h(state, goal) -> float` to run A\* or IDA\* on it:

```python
gf.search(successors, start=1, goal=27, algorithm="astar",
          heuristic=lambda s, goal: 0.0 if s >= goal else 1.0)
```

Native domains run with the GIL **released**; the Python callable reacquires the
GIL once per expansion.

## Choosing an algorithm

```python
for algo in ["bfs", "ucs", "greedy", "astar", "iddfs", "bidirectional"]:
    h = "manhattan" if algo in ("greedy", "astar") else "zero"
    r = gf.search(maze, algorithm=algo, heuristic=h)
    print(f"{algo:14} cost={r.cost} expanded={r.nodes_expanded}")
```

See [Algorithms](algorithms.md) for the full menu and the knobs
(`weight`, `beam_width`, `depth_limit`, `max_nodes`).

## Visualize it

```python
import matplotlib.pyplot as plt

anim = gf.viz.animate_grid(maze, r)          # the flagship animation
anim.save("astar.gif", writer="pillow", fps=25)

gf.viz.plot_grid(maze, r)                     # static snapshot
gf.viz.plot_frontier(r)                       # memory profile
plt.show()
```

The [visualization gallery](visualization.md) shows every plotting helper.
