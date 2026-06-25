# Getting started

## A maze in three lines

```python
import graphfinder as gf

r = gf.search(gf.sample_maze("wall"), algorithm="astar", heuristic="manhattan")
print(r)        # SearchResult(found=True, cost=20, expanded=25, frontier=3, stop=goal)
```

A [`SearchResult`](api.md) carries the `path` (a list of `(row, col)` cells),
`cost`, `found`, `nodes_expanded`, `nodes_generated`, `max_frontier_size`,
`stop_reason`, and a per-step `trace`.

## Watch it explore

```python
import matplotlib.pyplot as plt

maze = gf.random_maze_ascii(25, 25, 0.25, seed=0)
r = gf.search(maze, algorithm="astar", record=True)

anim = gf.viz.animate_grid(maze, r)     # the flagship animation
anim.save("astar.gif", writer="pillow", fps=25)

gf.viz.plot_grid(maze, r)               # or a static snapshot
plt.show()
```

## Compare algorithms

```python
maze = gf.sample_maze("wall")
results = {
    "BFS":    gf.search(maze, algorithm="bfs",    heuristic="zero"),
    "UCS":    gf.search(maze, algorithm="ucs",    heuristic="zero"),
    "Greedy": gf.search(maze, algorithm="greedy", heuristic="manhattan"),
    "A*":     gf.search(maze, algorithm="astar",  heuristic="manhattan"),
}
gf.viz.compare(results)                 # work (expanded) vs quality (cost)
```

## Explicit graphs

```python
edges = gf.gen_barabasi_albert(300, 3, seed=7)        # scale-free graph
r = gf.search_graph(300, edges, start=0, goal=299, algorithm="bidirectional")
gf.viz.plot_graph(300, edges, r)
```

## Implicit graphs (lazy successors)

States are ints or tuples of ints; you supply a successor function.

```python
# Reach 27 from 1 using +1 and *2 — BFS finds the fewest operations.
def successors(s):
    return [(s + 1, 1.0), (s * 2, 1.0)] if s < 100 else []

r = gf.search(successors, start=1, goal=27, algorithm="bfs")
print(r.path)   # [1, 2, 3, 6, 12, 13, 26, 27]
```

See [Algorithms](algorithms.md) for what each `algorithm=` value does, and the
[API reference](api.md) for every function.
