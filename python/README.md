# graphfinder (Python)

Graph traversal & pathfinding (informed and uninformed search) with a Rust core.

```python
import graphfinder as gf

# A* on a maze
r = gf.search(gf.sample_maze("wall"), algorithm="astar", heuristic="manhattan")
print(r)                     # SearchResult(found=True, cost=20, expanded=25, ...)
print(r.cost, r.nodes_expanded, len(r.trace))

# Explicit weighted graph from a generator
edges = gf.gen_barabasi_albert(200, 3, seed=1)
r = gf.search_graph(200, edges, start=0, goal=199, algorithm="bidirectional")

# Implicit graph (lazy successors; states are ints or tuples of ints)
def successors(s):
    return [(s + 1, 1.0), (s * 2, 1.0)] if s < 1000 else []
r = gf.search(successors, start=1, goal=27, algorithm="bfs")
```

Build from source (Rust core via maturin):

```bash
pip install maturin
maturin develop --release
python examples/quickstart.py
```

See the top-level README and `ROADMAP.md` for the full feature list. A
visualization layer (`graphfinder.viz`) is planned for Phase 4.
