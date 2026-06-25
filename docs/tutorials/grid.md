# Tutorial: grid pathfinding

A complete, runnable walk-through of pathfinding on a maze — from building the
map to animating A\*.

## 1. Build a maze

```python
import graphfinder as gf

maze = """
S.......#.
.#####.#..
.#...#.#.#
.#.#.#.#.#
.#.#...#.#
...#.####G
""".strip()
```

Or generate a reproducible random one (regenerate until solvable):

```python
maze = gf.random_maze_ascii(20, 20, 0.25, seed=1)
```

## 2. Run A\*

```python
r = gf.search(maze, algorithm="astar", heuristic="manhattan", record=True)
print(r.found, r.cost, r.nodes_expanded)
print(r.path[:5], "...")
```

`record=True` keeps the trace so we can animate it later.

## 3. See how the heuristic matters

Swap the heuristic and watch the work change while the cost stays optimal:

```python
for h in ["zero", "euclidean", "manhattan"]:
    r = gf.search(maze, algorithm="astar", heuristic=h)
    print(f"{h:10} cost={r.cost} expanded={r.nodes_expanded}")
```

`zero` behaves like UCS (floods); `manhattan` focuses the search. See
[Heuristics](../heuristics.md).

## 4. Diagonal movement

Allow 8-connectivity and use the matching `octile` heuristic:

```python
r = gf.search(maze, algorithm="astar", heuristic="octile", diagonal=True)
```

## 5. Animate it

```python
import matplotlib.pyplot as plt

anim = gf.viz.animate_grid(maze, r, interval=40)
anim.save("astar.gif", writer="pillow", fps=25)   # in a notebook: HTML(anim.to_jshtml())

gf.viz.plot_grid(maze, r)   # or a static snapshot
plt.show()
```

<p align="center">
  <img src="/assets/astar.gif" alt="A* on a maze" width="340">
</p>

## 6. Budget the search

Cap the expansions (useful for huge maps or live demos); the result reports why
it stopped:

```python
r = gf.search(maze, algorithm="bfs", heuristic="zero", max_nodes=50)
print(r.found, r.stop_reason)   # False "node_limit"  (if 50 wasn't enough)
```

## Next

- Compare *all* the algorithms on this maze →
  [Comparing algorithms](comparison.md).
- Move off the grid → [random & implicit graphs](graphs.md).
