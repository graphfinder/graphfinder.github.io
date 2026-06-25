"""Using graphfinder from Python (Rust core).

Run after compiling the module:
    maturin develop --release
    python examples/quickstart.py
"""
import logging

import graphfinder as gf

logging.basicConfig(level=logging.INFO, format="%(levelname)s %(name)s: %(message)s")
log = logging.getLogger("quickstart")

# 1) A* on a maze (native domain, runs with the GIL released)
r = gf.search(gf.sample_maze("wall"), algorithm="astar", heuristic="manhattan")
log.info("A* maze: %s", r)
log.info("path has %d cells, trace has %d expansions", len(r.path), len(r.trace))

# 2) Compare algorithms on the same maze
maze = gf.sample_maze("wall")
print(f"\n{'algorithm':<14}{'cost':>6}{'expanded':>10}{'frontier':>10}")
for algo in ["bfs", "ucs", "greedy", "astar", "iddfs", "bidirectional"]:
    h = "manhattan" if algo in ("greedy", "astar") else "zero"
    r = gf.search(maze, algorithm=algo, heuristic=h, record=False)
    cost = r.cost if r.found else float("inf")
    print(f"{algo:<14}{cost:>6}{r.nodes_expanded:>10}{r.max_frontier_size:>10}")

# 3) Explicit weighted graph from a generator
edges = gf.gen_barabasi_albert(300, 3, seed=7)
r = gf.search_graph(300, edges, start=0, goal=299, algorithm="bidirectional")
log.info("\nScale-free graph 0->299: found=%s cost=%s", r.found, r.cost)

# 4) Implicit graph: reach a target integer via +1 and *2 (BFS = fewest ops)
def successors(s):
    return [(s + 1, 1.0), (s * 2, 1.0)] if s < 100 else []

r = gf.search(successors, start=1, goal=27, algorithm="bfs")
log.info("Reach 27 from 1 via +1/*2 in %d ops: %s", int(r.cost), r.path)
