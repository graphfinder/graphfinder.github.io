"""Visualization demo. Saves figures and the flagship maze-search GIF.

    pip install graphfinder[viz]    # matplotlib (+ optional networkx)
    maturin develop --release
    python examples/demo_viz.py
"""
import logging

import matplotlib

matplotlib.use("Agg")  # headless: write files instead of opening windows
import matplotlib.pyplot as plt

import graphfinder as gf

logging.basicConfig(level=logging.INFO, format="%(levelname)s %(name)s: %(message)s")

OUT = "assets"
import os

os.makedirs(OUT, exist_ok=True)

# A reproducible maze that is solvable AND forces some real exploration
# (the random generator does not guarantee solvability, so we pick a good seed).
maze = None
for seed in range(200):
    candidate = gf.random_maze_ascii(25, 25, 0.25, seed)
    probe = gf.search(candidate, algorithm="astar", heuristic="manhattan", record=True)
    if probe.found and probe.nodes_expanded >= 60:
        maze = candidate
        logging.info("using maze seed=%d (expanded %d cells)", seed, probe.nodes_expanded)
        break
if maze is None:
    raise SystemExit("no suitable maze found; relax the criteria")

# 1) Flagship animation: watch A* explore, then draw the path.
result = gf.search(maze, algorithm="astar", heuristic="manhattan", record=True)
anim = gf.viz.animate_grid(maze, result, interval=40)
anim.save(f"{OUT}/astar_maze.gif", writer="pillow", fps=25)
logging.info("wrote %s/astar_maze.gif", OUT)

# 2) Static snapshot of the same run.
ax = gf.viz.plot_grid(maze, result)
ax.figure.savefig(f"{OUT}/astar_maze.png", dpi=120, bbox_inches="tight")

# 3) Compare algorithms on the same maze.
algos = {
    "BFS": gf.search(maze, algorithm="bfs", heuristic="zero"),
    "UCS": gf.search(maze, algorithm="ucs", heuristic="zero"),
    "Greedy": gf.search(maze, algorithm="greedy", heuristic="manhattan"),
    "A*": gf.search(maze, algorithm="astar", heuristic="manhattan"),
}
fig = gf.viz.compare(algos)
fig.savefig(f"{OUT}/compare.png", dpi=120, bbox_inches="tight")

# 4) Frontier-size curves overlaid.
ax = None
for name, res in algos.items():
    ax = gf.viz.plot_frontier(res, ax=ax, label=name)
ax.figure.savefig(f"{OUT}/frontier.png", dpi=120, bbox_inches="tight")

# 5) Explicit graph coloured by search state.
edges = gf.gen_barabasi_albert(60, 2, seed=3)
gres = gf.search_graph(60, edges, 0, 59, algorithm="bfs", record=True)
ax = gf.viz.plot_graph(60, edges, gres)
ax.figure.savefig(f"{OUT}/graph.png", dpi=120, bbox_inches="tight")

logging.info("done — see the %s/ directory", OUT)
plt.close("all")
