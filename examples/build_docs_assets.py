"""Generate the visualization assets embedded in the docs site (docs/assets/).

    pip install "graphfinder[viz]"
    maturin develop --release
    python examples/build_docs_assets.py
"""
import logging
import os

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt

import graphfinder as gf

logging.basicConfig(level=logging.INFO, format="%(levelname)s: %(message)s")
log = logging.getLogger("assets")

OUT = "docs/assets"
os.makedirs(OUT, exist_ok=True)


def pick_maze(w, h, density, min_expanded, seeds=range(400)):
    """First reproducible maze that is solvable and forces real exploration."""
    for s in seeds:
        m = gf.random_maze_ascii(w, h, density, s)
        r = gf.search(m, algorithm="astar", heuristic="manhattan", record=True)
        if r.found and r.nodes_expanded >= min_expanded:
            log.info("maze %dx%d seed=%d (A* expanded %d)", w, h, s, r.nodes_expanded)
            return m
    raise SystemExit("no suitable maze found")


# One shared maze for every grid figure, so the panels are directly comparable.
MAZE = pick_maze(28, 28, 0.25, 120)

ALGOS = [
    ("bfs", "zero", "BFS"),
    ("dfs", "zero", "DFS"),
    ("ucs", "zero", "UCS / Dijkstra"),
    ("greedy", "manhattan", "Greedy best-first"),
    ("astar", "manhattan", "A*"),
    ("weighted_astar", "manhattan", "Weighted A* (w=2)"),
]


def save(fig, name):
    path = f"{OUT}/{name}"
    fig.savefig(path, dpi=120, bbox_inches="tight")
    plt.close(fig)
    log.info("wrote %s", path)


# 1) Flagship animations: A* (directed) and BFS (flood) on the same maze.
for algo, h in [("astar", "manhattan"), ("bfs", "zero")]:
    r = gf.search(MAZE, algorithm=algo, heuristic=h, record=True)
    anim = gf.viz.animate_grid(MAZE, r, interval=35)
    anim.save(f"{OUT}/{algo}.gif", writer="pillow", fps=28)
    plt.close("all")
    log.info("wrote %s/%s.gif (%d frames)", OUT, algo, r.nodes_expanded + len(r.path))

# 2) Per-algorithm static snapshots in a 2x3 grid.
fig, axes = plt.subplots(2, 3, figsize=(13, 9))
for ax, (algo, h, title) in zip(axes.ravel(), ALGOS):
    r = gf.search(MAZE, algorithm=algo, heuristic=h, record=True)
    gf.viz.plot_grid(MAZE, r, ax=ax)
    cost = r.cost if r.found else float("inf")
    ax.set_title(f"{title}\nexpanded={r.nodes_expanded}  cost={cost}", fontsize=10)
fig.suptitle("Same maze, same loop — six behaviours", fontsize=14)
fig.tight_layout()
save(fig, "algorithms_grid.png")

# 3) Work-vs-quality comparison bars.
results = {title: gf.search(MAZE, algorithm=a, heuristic=h) for a, h, title in ALGOS}
fig = gf.viz.compare(results)
save(fig, "compare.png")

# 4) Frontier-size curves overlaid.
fig, ax = plt.subplots(figsize=(8, 4.5))
for algo, h, title in ALGOS:
    if algo == "dfs":
        continue
    r = gf.search(MAZE, algorithm=algo, heuristic=h, record=True)
    gf.viz.plot_frontier(r, ax=ax, label=title)
ax.set_title("Frontier size over the search (memory profile)")
save(fig, "frontier.png")

# 5) Heuristic strength: A* with zero / euclidean / manhattan (4-connected).
fig, axes = plt.subplots(1, 3, figsize=(13, 4.6))
for ax, hname in zip(axes, ["zero", "euclidean", "manhattan"]):
    r = gf.search(MAZE, algorithm="astar", heuristic=hname, record=True)
    gf.viz.plot_grid(MAZE, r, ax=ax)
    ax.set_title(f"A* + {hname}\nexpanded={r.nodes_expanded}", fontsize=10)
fig.suptitle("A stronger heuristic focuses the search", fontsize=13)
fig.tight_layout()
save(fig, "heuristics.png")

# 6) Bidirectional vs one-directional BFS.
fig, (a1, a2) = plt.subplots(1, 2, figsize=(9, 4.8))
rb = gf.search(MAZE, algorithm="bfs", heuristic="zero", record=True)
rd = gf.search(MAZE, algorithm="bidirectional", record=True)
gf.viz.plot_grid(MAZE, rb, ax=a1)
a1.set_title(f"BFS\nexpanded={rb.nodes_expanded}", fontsize=10)
gf.viz.plot_grid(MAZE, rd, ax=a2)
a2.set_title(f"Bidirectional\nexpanded={rd.nodes_expanded}", fontsize=10)
fig.suptitle("Two frontiers meet in the middle", fontsize=13)
fig.tight_layout()
save(fig, "bidirectional.png")

# 7) Explicit graph coloured by search state (scale-free).
edges = gf.gen_barabasi_albert(90, 2, seed=3)
gres = gf.search_graph(90, edges, 0, 89, algorithm="bfs", record=True)
fig, ax = plt.subplots(figsize=(7.5, 6))
gf.viz.plot_graph(90, edges, gres, ax=ax)
ax.set_title(f"Scale-free graph (Barabási–Albert) · BFS 0→89 · cost={gres.cost}")
save(fig, "graph.png")

# 8) Weighted terrain: A* routes around an expensive plateau (dark = costly).
H = W = 24
terrain = [[1.0] * W for _ in range(H)]
for r in range(4, 20):
    for c in range(9, 15):
        terrain[r][c] = 9.0  # expensive plateau in the middle
astar_w = gf.search_grid_costs(terrain, (0, 0), (H - 1, W - 1), algorithm="astar", record=True)
fig, (a0, a1) = plt.subplots(1, 2, figsize=(10, 5))
gf.viz.plot_costs(terrain, ax=a0)
gf.viz.plot_grid(terrain, astar_w, ax=a1)
a1.set_title(f"A* skirts the costly region\ncost={astar_w.cost:.1f}", fontsize=10)
fig.suptitle("Weighted terrain — moving into a cell costs its terrain", fontsize=13)
fig.tight_layout()
save(fig, "weighted.png")

# 6) Search tree — the parent links A* explored, on a small maze so it is legible.
tree_maze = pick_maze(7, 7, 0.2, 12)
tree_res = gf.search(tree_maze, algorithm="astar", heuristic="manhattan", record=True)
fig, ax = plt.subplots(figsize=(9, 5))
gf.viz.plot_search_tree(tree_res, ax=ax)
ax.set_title(
    f"A* search tree — {len(tree_res.tree) + 1} nodes; gold is the solution branch",
    fontsize=11,
)
fig.tight_layout()
save(fig, "search_tree.png")

# Keep the top-level hero in sync (used by the GitHub README).
import shutil

shutil.copyfile(f"{OUT}/astar.gif", "assets/astar_maze.gif")
log.info("done — assets in %s/", OUT)
