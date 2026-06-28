# Implicit puzzles

A maze or an edge list is an **explicit** graph: the nodes and edges exist in
memory. A puzzle is an **implicit** graph — astronomically too large to store
(the 15-puzzle has ~10¹³ states), so each state *generates its neighbours on
demand*. graphfinder ships three classic ones, and because they implement the
same [`Graph`](architecture.md) trait as every other domain, **the same search
loop and every algorithm run on them unchanged** — and natively in Rust, with
the GIL released for the built-in heuristics.

| Function              | State (tuple)               | Move (cost 1)                 | Heuristic (admissible)        |
|-----------------------|-----------------------------|-------------------------------|-------------------------------|
| `search_npuzzle`      | tiles row-major, `0`=blank  | slide a tile into the blank   | `manhattan`, `misplaced`      |
| `search_hanoi`        | peg of each disk            | move the smallest top disk    | `misplaced`                   |
| `search_wordladder`   | the word (`str`)            | change one letter             | `hamming`                     |

All three accept the usual knobs: `algorithm=` (`"astar"`, `"bfs"`, `"ida_star"`,
…), `heuristic=` (a built-in name, `"zero"`, or a callable
`h(state, goal) -> float`), `record=`, `max_nodes=`, etc.

## Sliding-tile puzzle (8/15-puzzle)

```python
import graphfinder as gf

# Flat, row-major start state; 0 is the blank. Goal defaults to 1..n, 0.
r = gf.search_npuzzle([1, 2, 3, 4, 0, 6, 7, 5, 8], heuristic="manhattan")
r.cost            # 2.0  — optimal number of slides
r.path[0]         # (1, 2, 3, 4, 0, 6, 7, 5, 8)
r.path[-1]        # (1, 2, 3, 4, 5, 6, 7, 8, 0)  — solved
```

`tiles` must be a permutation of `0..n` whose length is a perfect square (9, 16,
…). Half of all arrangements are **unreachable** from a given goal (a parity
invariant), so an unsolvable start raises `ValueError` rather than searching
forever. Pass a custom `goal=` to solve toward a different target.

**Why the heuristic matters.** Manhattan dominates misplaced-tiles, so A\*
expands fewer states for the same optimal answer — a clean illustration of
heuristic strength:

```python
hard = [8, 6, 7, 2, 5, 4, 3, 0, 1]   # a deep 8-puzzle instance
gf.search_npuzzle(hard, heuristic="misplaced").nodes_expanded   # many
gf.search_npuzzle(hard, heuristic="manhattan").nodes_expanded   # far fewer
# Memory-bound? IDA* solves it with almost no open list:
gf.search_npuzzle(hard, algorithm="ida_star", heuristic="manhattan").cost
```

## Towers of Hanoi

```python
r = gf.search_hanoi(disks=4)        # all disks peg 0 → all disks last peg
r.cost                              # 15.0  == 2**4 - 1, the known optimum
r.path[0], r.path[-1]              # (0,0,0,0)  →  (2,2,2,2)
```

A state is a tuple `peg_of_disk[d]` with disk `0` the smallest. The classic
3-peg optimum is `2**disks − 1`; pass `pegs=4` for the Frame–Stewart variant.
The default heuristic counts misplaced disks (admissible and consistent).

## Word ladder

```python
words = ["hit", "hot", "dot", "dog", "cog", "lot", "log"]
r = gf.search_wordladder("hit", "cog", words)
r.cost     # 4.0
r.path     # ['hit', 'hot', 'dot', 'dog', 'cog']
```

Neighbours are the dictionary words differing in exactly one letter; `start` and
`goal` are added to the dictionary automatically so the endpoints are valid
nodes. Words must share a length. The Hamming heuristic (number of differing
letters) is admissible because each step fixes at most one letter.

## Rust

The same domains live in `graphfinder_core::puzzles`:

```rust
use graphfinder_core::puzzles::{Hanoi, HanoiMisplaced, NPuzzle, PuzzleManhattan};
use graphfinder_core::{search, Algorithm};

let p = NPuzzle::new(3);
let r = search(&p, vec![1,2,3,4,0,6,7,5,8], p.goal(),
               Algorithm::astar(), &PuzzleManhattan { width: 3 }, false);
assert_eq!(r.cost, 2.0);

let h = Hanoi::new(4);
let r = search(&h, h.start(), h.goal(), Algorithm::astar(), &HanoiMisplaced, false);
assert_eq!(r.cost, 15.0); // 2^4 - 1
```

```bash
cargo run --example puzzles -p graphfinder-core
```

See the [API reference](api.md#puzzles) for the full surface, and
[Algorithms](algorithms.md) for the iterative-deepening variants (`ida_star`)
that shine on these large implicit spaces.
