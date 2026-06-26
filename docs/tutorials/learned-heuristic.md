# Tutorial: a learned heuristic (PyTorch)

A\* is only as good as its heuristic. Instead of hand-designing one, we can
**learn** it: train a small neural net to predict the remaining cost to the goal,
then plug it into A\* through graphfinder's [custom-heuristic](../heuristics.md)
hook. This is most useful on **weighted terrain**, where the geometric Manhattan
heuristic (which counts steps) badly underestimates the true cost.

```bash
pip install "graphfinder[torch]" matplotlib
python examples/learned_heuristic.py
```

## 1. Generate training data

The "true" heuristic is the actual cost-to-goal — which graphfinder can compute
with UCS. Sample it over the free cells of a weighted maze:

```python
import graphfinder as gf
from graphfinder.viz import _parse_map

maze = gf.random_maze_ascii(30, 30, 0.12, seed=3)
walls, *_ = _parse_map(maze)
H = len(maze.splitlines()); W = max(len(r) for r in maze.splitlines())
goal = (H - 1, W - 1)
# random terrain costs 1–9 (0 = wall)
import random; rng = random.Random(0)
costs = [[0.0 if (r, c) in walls else float(rng.randint(1, 9)) for c in range(W)]
         for r in range(H)]

def feats(node, g):
    (r, c), (gr, gc) = node, g
    return [r/H, c/W, gr/H, gc/W, abs(r-gr)/H, abs(c-gc)/W]

X, Y = [], []
for r in range(H):
    for c in range(W):
        if (r, c) in walls: continue
        res = gf.search_grid_costs(costs, (r, c), goal, algorithm="ucs", heuristic="zero")
        if res.found:
            X.append(feats((r, c), goal)); Y.append(res.cost)
```

## 2. Train a small MLP

```python
import torch, torch.nn as nn
X = torch.tensor(X); Y = torch.tensor(Y).unsqueeze(1)
model = nn.Sequential(nn.Linear(6, 64), nn.ReLU(), nn.Linear(64, 64), nn.ReLU(), nn.Linear(64, 1))
opt = torch.optim.Adam(model.parameters(), lr=1e-3)
for _ in range(800):
    opt.zero_grad(); nn.functional.mse_loss(model(X), Y).backward(); opt.step()
model.eval()
```

## 3. Plug it into A\*

The `graphfinder.integrations.torch` bridge wraps any model + encoder into an
`h(node, goal)` callable:

```python
from graphfinder.integrations import torch as gft

h = gft.as_heuristic(model, lambda node, g: torch.tensor(feats(node, g)))
r = gf.search_grid_costs(costs, start=(0, 0), goal=goal, algorithm="astar", heuristic=h)
```

The bridge is framework-agnostic — `model` can be a PyTorch `nn.Module`, a NumPy
function or a scikit-learn regressor; it just calls `model(encode(node, goal))`
and coerces the result to `float`.

## 4. Result

On random weighted terrain, the learned heuristic expands far fewer nodes than
the admissible Manhattan heuristic, at a near-optimal cost:

<p align="center">
  <img src="/assets/learned_heuristic.png" alt="learned heuristic comparison" width="760">
</p>

```text
UCS (uninformed)   expanded=546   cost/optimal=1.000
A* Manhattan       expanded=421   cost/optimal=1.000
A* learned (NN)    expanded=160   cost/optimal=1.002
```

!!! warning "Admissibility"
    A learned heuristic is generally **not admissible**, so A\* may return a
    slightly sub-optimal path (here ~0.2% above optimal). When you need a
    guaranteed bound, cap it (`weighted_astar`) or verify the returned cost.
