# Shortest paths (negative weights)

Most of graphfinder is **GENERAL-SEARCH**: a goal-directed walk over a frontier
([Algorithms](algorithms.md)). Dijkstra and A\* are the optimal members of that
family — but they **assume every edge cost is non-negative**. The moment a
weight can be negative (a refund, an elevation drop, a reward), they can return
the wrong answer.

For that case graphfinder ships the two classic **relaxation / dynamic
programming** algorithms. They are different in kind: instead of stopping at a
goal, they compute *all* the distances from a source (or between *every* pair),
and they handle negative edges and detect **negative cycles**.

| Function          | Computes              | Negative edges | Negative cycle      | Time     |
|-------------------|-----------------------|----------------|---------------------|----------|
| `bellman_ford`    | one source → all      | yes            | yes (from source)   | `O(V·E)` |
| `floyd_warshall`  | all pairs             | yes            | yes (anywhere)      | `O(V³)`  |

Both take an **edge list** over `0..num_nodes` (`(u, v, w)` triples), exactly
like [`search_graph`](api.md). They default to **directed** edges
(`undirected=False`) — an undirected negative edge is itself a trivial negative
cycle, so directed graphs are the meaningful setting.

!!! note "When to reach for these vs. Dijkstra/A\*"
    Use [`ucs`/`dijkstra` or `astar`](algorithms.md) for a single
    start→goal query on a **non-negative** graph — they stop early and (with a
    heuristic) explore far less. Use **Bellman–Ford** when some edge can be
    negative, or when you need to *detect* a negative cycle. Use
    **Floyd–Warshall** when you want the whole distance matrix at once.

## Bellman–Ford — single source

```python
import graphfinder as gf

# 0 →(4)→ 1, 0 →(5)→ 2, 1 →(-3)→ 2
# The cheapest 0→2 path goes through 1: 4 + (-3) = 1, beating the direct 5.
edges = [(0, 1, 4.0), (0, 2, 5.0), (1, 2, -3.0)]
sp = gf.bellman_ford(num_nodes=3, edges=edges, source=0)

sp.dist            # [0.0, 4.0, 1.0]          cheapest cost from the source
sp.path_to(2)      # [0, 1, 2]                rebuilt shortest path
sp.pred            # [None, 0, 1]             predecessor on each shortest path
sp.negative_cycle  # False
```

`dist[v]` is `inf` for unreachable nodes, and `path_to(v)` then returns `None`.

### Detecting a negative cycle

If a cycle of net-negative weight is reachable from the source, no finite
shortest path is well defined. Bellman–Ford reports it:

```python
# 0 → 1 → 2 → 0 sums to -1.
cyc = gf.bellman_ford(3, [(0, 1, 1.0), (1, 2, -3.0), (2, 0, 1.0)], source=0)
cyc.negative_cycle   # True
```

## Floyd–Warshall — all pairs

```python
edges = [(0, 1, 4.0), (0, 2, 5.0), (1, 2, -3.0), (2, 3, 2.0)]
ap = gf.floyd_warshall(num_nodes=4, edges=edges)

ap.distance(0, 3)    # 3.0          cheapest 0→3 cost
ap.path(0, 3)        # [0, 1, 2, 3] rebuilt path
ap.matrix()          # full 4×4 distance matrix (rows of floats; inf = no path)
ap.negative_cycle    # False        True if any vertex can reach itself at < 0
```

`O(V³)` time and `O(V²)` memory: ideal for small/medium or dense graphs where
you want every distance, impractical for very large sparse ones (run
Bellman–Ford or Dijkstra per source instead).

## Rust

The same two functions live in `graphfinder_core::shortest_paths`, operating on a
[`CsrGraph`](domains.md):

```rust
use graphfinder_core::{CsrGraph, bellman_ford, floyd_warshall};

let g = CsrGraph::from_edges(3, &[(0, 1, 4.0), (0, 2, 5.0), (1, 2, -3.0)], false);

let sp = bellman_ford(&g, 0);
assert_eq!(sp.dist[2], 1.0);
assert_eq!(sp.path_to(2), Some(vec![0, 1, 2]));

let ap = floyd_warshall(&g);
assert_eq!(ap.distance(0, 2), 1.0);
```

```bash
cargo run --example shortest_paths -p graphfinder-core
```

See the [API reference](api.md#shortest-paths) for the full `ShortestPaths` and
`AllPairs` surface.
