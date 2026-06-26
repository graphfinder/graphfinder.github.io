"""Learned heuristic (PyTorch): train a small MLP to estimate the remaining cost
to the goal, then plug it into A* via graphfinder's custom-heuristic hook.

    pip install "graphfinder[torch]" matplotlib
    python examples/learned_heuristic.py

A learned heuristic is generally NOT admissible, so A* may return a slightly
sub-optimal path — but it expands far fewer nodes than an admissible one. The
figure compares UCS (uninformed), A*+Manhattan (admissible) and A*+learned.
"""
import logging
import os
import random
import statistics as st

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt
import torch
import torch.nn as nn

import graphfinder as gf
from graphfinder.integrations import torch as gft
from graphfinder.viz import _parse_map

logging.basicConfig(level=logging.INFO, format="%(levelname)s: %(message)s")
log = logging.getLogger("learned")
torch.manual_seed(0)

# 1) A solvable maze with *random terrain costs* (1–9). On weighted terrain the
#    Manhattan heuristic (which counts steps) badly underestimates the true cost,
#    so a learned heuristic that predicts cost has a clear edge. Fixed goal at the
#    bottom-right corner.
maze = gf.random_maze_ascii(30, 30, 0.12, seed=3)
walls, _s, _g, _costs = _parse_map(maze)
H = len(maze.splitlines())
W = max(len(row) for row in maze.splitlines())
goal = (H - 1, W - 1)
rng = random.Random(0)
costs = [
    [0.0 if (r, c) in walls else float(rng.randint(1, 9)) for c in range(W)]
    for r in range(H)
]
costs[goal[0]][goal[1]] = 1.0
free = [(r, c) for r in range(H) for c in range(W) if (r, c) not in walls]


def feats(node, g):
    (r, c), (gr, gc) = node, g
    return [r / H, c / W, gr / H, gc / W, abs(r - gr) / H, abs(c - gc) / W]


# 2) Training targets: the true cost-to-goal of each reachable free cell (UCS).
xs, ys = [], []
for cell in free:
    res = gf.search_grid_costs(costs, start=cell, goal=goal, algorithm="ucs", heuristic="zero")
    if res.found:
        xs.append(feats(cell, goal))
        ys.append(res.cost)

X = torch.tensor(xs, dtype=torch.float32)
Y = torch.tensor(ys, dtype=torch.float32).unsqueeze(1)
perm = torch.randperm(len(X))
n_train = int(0.8 * len(X))
tr, te = perm[:n_train], perm[n_train:]
log.info("dataset: %d reachable cells (%d train / %d test)", len(X), len(tr), len(te))

# 3) Train a small MLP to predict the remaining cost.
model = nn.Sequential(nn.Linear(6, 64), nn.ReLU(), nn.Linear(64, 64), nn.ReLU(), nn.Linear(64, 1))
opt = torch.optim.Adam(model.parameters(), lr=1e-3)
loss_fn = nn.MSELoss()
for _ in range(800):
    opt.zero_grad()
    loss_fn(model(X[tr]), Y[tr]).backward()
    opt.step()
model.eval()
with torch.no_grad():
    mae = (model(X[te]) - Y[te]).abs().mean().item()
log.info("trained — test MAE %.2f cells", mae)

# 4) Use the model as an A* heuristic.
h_learned = gft.as_heuristic(model, lambda node, g: torch.tensor(feats(node, g)))

# 5) Compare on held-out start cells: work (expanded) vs quality (cost / optimal).
test_starts = [free[i] for i in te.tolist()[:60]]


def evaluate(algorithm, heuristic):
    expanded, ratio = [], []
    for s in test_starts:
        r = gf.search_grid_costs(costs, start=s, goal=goal, algorithm=algorithm, heuristic=heuristic)
        opt = gf.search_grid_costs(costs, start=s, goal=goal, algorithm="ucs", heuristic="zero")
        if r.found and opt.found:
            expanded.append(r.nodes_expanded)
            ratio.append(r.cost / opt.cost)
    return st.mean(expanded), st.mean(ratio)


runs = {
    "UCS\n(uninformed)": evaluate("ucs", "zero"),
    "A*\nManhattan": evaluate("astar", "manhattan"),
    "A*\nlearned (NN)": evaluate("astar", h_learned),
}
for name, (exp, ratio) in runs.items():
    log.info("%-16s expanded=%.0f  cost/optimal=%.3f", name.replace(chr(10), " "), exp, ratio)

# 6) Figure.
names = list(runs)
fig, (a1, a2) = plt.subplots(1, 2, figsize=(10, 4))
a1.bar(names, [runs[n][0] for n in names], color="#90caf9", edgecolor="#37474f")
a1.set_ylabel("mean nodes expanded")
a1.set_title("Work (lower is better)")
ratios = [runs[n][1] for n in names]
a2.bar(names, ratios, color="#fbc02d", edgecolor="#37474f")
a2.axhline(1.0, color="#43a047", linestyle="--", linewidth=1, label="optimal")
a2.set_ylim(0.95, max(1.05, max(ratios) * 1.05))
a2.set_ylabel("mean cost / optimal")
a2.set_title("Quality (1.0 = optimal)")
a2.legend()
fig.suptitle("Learned heuristic: fewer expansions, near-optimal paths", fontsize=13)
fig.tight_layout()
os.makedirs("docs/assets", exist_ok=True)
fig.savefig("docs/assets/learned_heuristic.png", dpi=120, bbox_inches="tight")
log.info("wrote docs/assets/learned_heuristic.png")
