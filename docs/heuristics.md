# Heuristics

A heuristic `h(n)` estimates the remaining cost from a node to the goal. It is
what turns blind search into *informed* search: Greedy uses `h` alone, A\* uses
`g + h`. A heuristic is **admissible** if it never overestimates the true
remaining cost — admissibility is what makes A\* and IDA\* optimal.

The `heuristic=` argument accepts either a **built-in name** (`str`) or a
**custom Python callable** `h(node, goal) -> float` — in any domain:

```python
gf.search(maze, algorithm="astar", heuristic="manhattan")              # built-in
gf.search(maze, algorithm="astar", heuristic=lambda n, g: abs(n[0]-g[0]) + abs(n[1]-g[1]))
```

## Built-in grid heuristics

| `heuristic` | Formula | Use when | Admissible on… |
|-------------|---------|----------|----------------|
| `zero`      | `0` | you want uninformed behaviour | always (A\*→UCS) |
| `manhattan` | `|Δrow| + |Δcol|` | 4-connected grids | 4-connected unit grids |
| `euclidean` | `√(Δrow² + Δcol²)` | any-angle / diagonal travel | always (under-estimates on grids) |
| `octile`    | `(L−S) + √2·S` over the long/short legs | 8-connected grids | 8-connected grids with diagonal cost √2 |

`zero` makes any informed algorithm behave like its uninformed counterpart —
handy for A/B comparisons. On an 8-connected grid (`diagonal=True`), use
`octile`: it is the exact diagonal distance and stays admissible, whereas
`manhattan` would overestimate and lose A\*'s optimality guarantee.

!!! warning "Weighted terrain"
    On a [weighted grid](domains.md#weighted-terrain) the geometric heuristics
    count *steps*, so they stay admissible only when **every terrain cost ≥ 1**
    (the usual case). If costs can be below 1, a step-counting heuristic may
    overestimate — use `zero` (→ Dijkstra) or scale the heuristic by the minimum
    terrain cost to keep A\* optimal.

## Stronger heuristic, less work

The closer (but still admissible) the estimate, the more the search focuses on
the goal and the fewer nodes it expands. Same maze, A\* with three heuristics:

<p align="center">
  <img src="/assets/heuristics.png" alt="A* with zero, euclidean, manhattan" width="820">
</p>

`zero` expands like UCS (floods the map); `euclidean` already narrows it;
`manhattan` — the tightest admissible estimate on a 4-connected grid — focuses
the search the most. All three return the **same optimal cost**; they differ only
in *work*.

## Custom heuristics

Pass a callable `h(node, goal) -> float` instead of a name. The node and goal are
handed to your function in the domain's natural form, so the signature differs
per domain:

=== "Grid"

    Nodes are `(row, col)` tuples. Here is a hand-written Chebyshev distance
    (useful on 8-connected grids):

    ```python
    def chebyshev(node, goal):
        return float(max(abs(node[0] - goal[0]), abs(node[1] - goal[1])))

    gf.search(maze, algorithm="astar", heuristic=chebyshev, diagonal=True)
    ```

=== "Explicit graph"

    Nodes are integer ids. With coordinates per node you can use straight-line
    distance to turn `search_graph` into a real A\*:

    ```python
    import math
    xy = {i: (x_i, y_i) for i in range(n)}   # your node coordinates

    def straight_line(u, v):
        (ax, ay), (bx, by) = xy[u], xy[v]
        return math.hypot(ax - bx, ay - by)

    gf.search_graph(n, edges, 0, n - 1, algorithm="astar", heuristic=straight_line)
    ```

=== "Implicit graph"

    States are ints or tuples of ints (whatever your successor function uses):

    ```python
    def successors(s):
        return [(s + 1, 1.0), (s * 2, 1.0)] if s < 100 else []

    # admissible: at least one operation remains while below the goal
    def h(state, goal):
        return 0.0 if state >= goal else 1.0

    gf.search(successors, start=1, goal=27, algorithm="astar", heuristic=h)
    ```

A custom callable works with every priority-based algorithm (`greedy`, `astar`,
`weighted_astar`, `ida_star`, `beam`). Built-in heuristics run in pure Rust with
the GIL released; a Python callable reacquires the GIL once per node it scores.

### Designing one

- **Keep it admissible** for optimal A\*/IDA\*: `h(n) ≤ true remaining cost`.
- **Make it consistent** (`h(n) ≤ cost(n, n') + h(n')`) and A\* never needs to
  re-open a node — the strongest, best-behaved class.
- **Trade optimality for speed** deliberately with `weighted_astar(weight=w)`:
  it inflates `h` by `w`, returning a path at most `w×` the optimum.

In Rust, a heuristic is a tiny trait impl:

```rust
use graphfinder_core::{Heuristic, Cell};

struct Chebyshev;
impl Heuristic<Cell> for Chebyshev {
    fn estimate(&self, n: &Cell, goal: &Cell) -> f64 {
        ((n.row - goal.row).abs().max((n.col - goal.col).abs())) as f64
    }
}
```

See [Algorithms](algorithms.md) for how `h` enters the priority and
[Visualization](visualization.md) to *see* its effect.
