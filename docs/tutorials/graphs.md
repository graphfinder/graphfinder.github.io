# Tutorial: random & implicit graphs

Beyond mazes, graphfinder searches **explicit** graphs (an edge list) and
**implicit** graphs (a successor function). This tutorial covers both.

## Explicit weighted graphs

Build a graph from an edge list over nodes `0..n`:

```python
import graphfinder as gf

edges = [(0, 1, 1.0), (1, 2, 2.0), (0, 2, 4.0), (2, 3, 1.0)]
r = gf.search_graph(4, edges, start=0, goal=3, algorithm="dijkstra")
print(r.cost, r.path)   # 4.0 [0, 1, 2, 3]
```

### Random graph families

Each generator returns an edge list ready for `search_graph`:

```python
er = gf.gen_erdos_renyi(100, 0.05, seed=1)      # uniform random
ba = gf.gen_barabasi_albert(100, 2, seed=1)     # scale-free (hubs)
ws = gf.gen_watts_strogatz(100, 4, 0.1, seed=1) # small-world
```

Search and visualize one:

```python
r = gf.search_graph(90, ba, start=0, goal=89, algorithm="bfs", record=True)
gf.viz.plot_graph(90, ba, r)
```

<p align="center">
  <img src="/assets/graph.png" alt="BFS on a scale-free graph" width="560">
</p>

Bidirectional search shines on large graphs — it agrees with BFS on cost while
expanding far fewer nodes:

```python
bfs  = gf.search_graph(90, ba, 0, 89, algorithm="bfs")
bidi = gf.search_graph(90, ba, 0, 89, algorithm="bidirectional")
assert bfs.cost == bidi.cost
print(bfs.nodes_expanded, "vs", bidi.nodes_expanded)
```

## Implicit graphs (state spaces)

When the graph is huge or generated on the fly, give a **successor function**.
States are ints or tuples of ints.

### Arithmetic reachability

```python
# Reach a target integer using +1 and *2 — BFS finds the fewest operations.
def successors(s):
    return [(s + 1, 1.0), (s * 2, 1.0)] if s < 10_000 else []

r = gf.search(successors, start=1, goal=27, algorithm="bfs")
print(r.path)   # [1, 2, 3, 6, 12, 13, 26, 27]   (7 operations)
```

### A sliding-tile puzzle

Encode the board as a tuple; the successor function returns the boards reachable
by sliding the blank (`0`). A custom heuristic (tiles out of place) makes A\*
efficient:

```python
GOAL = (1, 2, 3, 4, 5, 6, 7, 8, 0)

def neighbors(state):
    i = state.index(0)
    r, c = divmod(i, 3)
    out = []
    for dr, dc in ((1,0),(-1,0),(0,1),(0,-1)):
        nr, nc = r + dr, c + dc
        if 0 <= nr < 3 and 0 <= nc < 3:
            j = nr * 3 + nc
            b = list(state)
            b[i], b[j] = b[j], b[i]
            out.append((tuple(b), 1.0))
    return out

def misplaced(state, goal):
    return float(sum(s != g and s != 0 for s, g in zip(state, goal)))

start = (1, 2, 3, 4, 0, 6, 7, 5, 8)
r = gf.search(neighbors, start=start, goal=GOAL, algorithm="astar", heuristic=misplaced)
print(r.found, int(r.cost), "moves")
```

Native domains run with the GIL released; the Python successor callable
reacquires the GIL once per expansion — so you can bring an arbitrary state space
while the search loop stays in Rust.

## When to use which

- **Explicit** when the graph fits in memory and you have all edges (road
  networks, social graphs, dependency graphs).
- **Implicit** when the state space is enormous or infinite (puzzles, planning,
  reachability) — only the reachable part is ever generated.

See [Domains](../domains.md) for the underlying model and
[Algorithms](../algorithms.md) for picking a search.
