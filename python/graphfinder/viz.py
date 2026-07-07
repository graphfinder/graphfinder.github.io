"""Visualization helpers. Consume the `SearchResult` from the Rust core.

Requires matplotlib (``pip install graphfinder[viz]``). Functions:
  - animate_grid(map, result)          # the flagship: watch the search explore
  - plot_grid(map, result, ax=None)    # static snapshot (terrain + visited + path)
  - plot_costs(map, ax=None)           # terrain-cost heatmap with a colourbar
  - plot_frontier(result, ax=None)     # frontier size per expansion (memory)
  - compare(results)                   # bar charts across algorithms
  - plot_graph(n, edges, result)       # general graph, nodes coloured by state
  - plot_search_tree(result)           # the tree of best-parent links explored

Grid helpers accept either an ASCII map (digits ``1``–``9`` are terrain costs) or
the terrain-cost matrix passed to ``search_grid_costs``.

matplotlib is imported lazily inside each function, so importing this module is
cheap and never required for the core search API.
"""
from __future__ import annotations  # keep the matplotlib type hints lazy

import logging
from collections import deque

logger = logging.getLogger(__name__)

# Overlay colours (RGB 0..1) drawn on top of the terrain background.
_WALL_RGB = (0.216, 0.278, 0.310)  # slate
_VISITED_RGB = (0.565, 0.792, 0.976)  # light blue
_PATH_RGB = (0.984, 0.753, 0.141)  # gold
_START_RGB = (0.263, 0.627, 0.278)  # green
_GOAL_RGB = (0.898, 0.224, 0.208)  # red
_FREE_RGB = (0.961, 0.961, 0.961)  # near-white (unweighted free cell)


def _parse_map(map_str):
    """Parse an ASCII map into ``(walls, start, goal, costs)``.

    ``#`` is a wall, ``S``/``G`` the endpoints, a digit ``1``–``9`` sets that
    cell's terrain cost, everything else is a free cell of cost ``1.0``.
    """
    lines = [ln for ln in map_str.splitlines() if ln]
    height = len(lines)
    width = max((len(ln) for ln in lines), default=0)
    walls, start, goal = set(), None, None
    costs = [[1.0] * width for _ in range(height)]
    for r, line in enumerate(lines):
        for c, ch in enumerate(line):
            if ch == "#":
                walls.add((r, c))
            elif ch == "S":
                start = (r, c)
            elif ch == "G":
                goal = (r, c)
            elif ch.isdigit() and ch != "0":
                costs[r][c] = float(ch)
    return walls, start, goal, costs


def _grid_from_arg(map, result=None):
    """Accept either an ASCII map (str) or a matrix of terrain costs (a cell
    ≤ 0 or non-finite is a wall). Returns ``(walls, start, goal, costs)``; for a
    cost matrix the endpoints are taken from ``result.path``."""
    if isinstance(map, str):
        return _parse_map(map)
    import math

    rows = [list(row) for row in map]
    height = len(rows)
    width = max((len(r) for r in rows), default=0)
    walls = set()
    costs = [[1.0] * width for _ in range(height)]
    for r in range(height):
        for c in range(width):
            x = float(rows[r][c]) if c < len(rows[r]) else 0.0
            if not math.isfinite(x) or x <= 0.0:
                walls.add((r, c))
            else:
                costs[r][c] = x
    path = (result.path or []) if result is not None else []
    start = tuple(path[0]) if path else None
    goal = tuple(path[-1]) if path else None
    return walls, start, goal, costs


def _terrain_image(costs, walls):
    """Build an HxWx3 RGB background: free cells shaded by terrain cost
    (light → orange), walls slate. Uniform light when the grid is unweighted."""
    import numpy as np
    from matplotlib import pyplot as plt
    from matplotlib.colors import Normalize

    arr = np.array(costs, dtype=float)
    height, width = arr.shape
    max_cost = float(arr.max()) if arr.size else 1.0
    img = np.empty((height, width, 3))
    if max_cost <= 1.0 + 1e-9:
        img[:] = _FREE_RGB
    else:
        cmap = plt.get_cmap("YlOrBr")
        norm = Normalize(vmin=1.0, vmax=max_cost)
        for r in range(height):
            for c in range(width):
                img[r, c] = cmap(0.10 + 0.65 * norm(arr[r, c]))[:3]
    for (r, c) in walls:
        img[r, c] = _WALL_RGB
    return img


def _blend(base, overlay, alpha):
    return tuple(alpha * o + (1.0 - alpha) * b for o, b in zip(overlay, base))


def _no_ticks(ax):
    ax.set_xticks([])
    ax.set_yticks([])


def plot_grid(map, result, ax: matplotlib.axes.Axes | None = None) -> matplotlib.axes.Axes:
    """Static snapshot: terrain shading, expanded cells, and the path on top.

    Args:
        map (str | list[list[float]]): the ASCII map, or the terrain-cost matrix
            passed to ``search_grid_costs``.
        result (SearchResult): from a search run with ``record=True``.
        ax (matplotlib.axes.Axes, optional): axes to draw on; created if omitted.

    Returns:
        matplotlib.axes.Axes: the axes the grid was drawn on.
    """
    import matplotlib.pyplot as plt

    walls, start, goal, costs = _grid_from_arg(map, result)
    img = _terrain_image(costs, walls)
    # Blend visited cells over the terrain so the costs stay visible underneath.
    for node, _g, _fs in result.trace:
        rc = tuple(node)
        if rc != start and rc != goal:
            img[rc] = _blend(img[rc], _VISITED_RGB, 0.55)
    for node in result.path or []:
        rc = tuple(node)
        if rc != start and rc != goal:
            img[rc] = _PATH_RGB
    if start is not None:
        img[start] = _START_RGB
    if goal is not None:
        img[goal] = _GOAL_RGB

    if ax is None:
        _, ax = plt.subplots()
    ax.imshow(img, interpolation="nearest")
    _no_ticks(ax)
    cost = result.cost if result.found else float("inf")
    ax.set_title(f"expanded={result.nodes_expanded}  cost={cost}")
    logger.info("plot_grid: %d expanded", result.nodes_expanded)
    return ax


def plot_costs(map, ax: matplotlib.axes.Axes | None = None) -> matplotlib.axes.Axes:
    """Heatmap of the terrain costs (walls left blank), with a colourbar.

    Args:
        map (str | list[list[float]]): ASCII map (digits = costs) or cost matrix.
        ax (matplotlib.axes.Axes, optional): axes to draw on; created if omitted.

    Returns:
        matplotlib.axes.Axes: the axes the heatmap was drawn on.
    """
    import matplotlib.pyplot as plt
    import numpy as np

    walls, _start, _goal, costs = _grid_from_arg(map)
    arr = np.array(costs, dtype=float)
    for (r, c) in walls:
        arr[r, c] = np.nan
    if ax is None:
        _, ax = plt.subplots()
    im = ax.imshow(arr, cmap="YlOrBr", interpolation="nearest")
    ax.figure.colorbar(im, ax=ax, label="terrain cost")
    _no_ticks(ax)
    ax.set_title("Terrain costs (walls blank)")
    return ax


def animate_grid(
    map, result, interval=60, show_path=True
) -> matplotlib.animation.FuncAnimation:
    """Animate the search exploring the maze, then trace the final path.

    Each frame marks the next expanded cell; once the frontier has been replayed,
    the solution path is drawn cell by cell. This is the flagship "watch A*
    explore" visualization.

    Args:
        map (str): the ASCII map searched.
        result (SearchResult): run with ``record=True`` (needs the trace).
        interval (int): milliseconds between frames.
        show_path (bool): append the path-drawing frames at the end.

    Returns:
        matplotlib.animation.FuncAnimation: the animation (in a notebook,
        display it with ``HTML(anim.to_jshtml())``).
    """
    import matplotlib.pyplot as plt
    from matplotlib.animation import FuncAnimation

    walls, start, goal, costs = _grid_from_arg(map, result)
    expanded = [tuple(node) for node, _g, _fs in result.trace]
    if not expanded:
        raise ValueError("empty trace; run the search with record=True")
    path = [tuple(n) for n in (result.path or [])] if show_path else []

    img = _terrain_image(costs, walls)

    def mark_endpoints():
        if start is not None:
            img[start] = _START_RGB
        if goal is not None:
            img[goal] = _GOAL_RGB

    mark_endpoints()
    fig, ax = plt.subplots()
    im = ax.imshow(img, interpolation="nearest")
    _no_ticks(ax)
    n_expand, n_path = len(expanded), len(path)
    total = n_expand + n_path
    logger.info("animate_grid: %d frames (%d expand + %d path)", total, n_expand, n_path)

    def update(frame):
        if frame < n_expand:
            cell = expanded[frame]
            if cell not in (start, goal):
                img[cell] = _blend(img[cell], _VISITED_RGB, 0.55)
            ax.set_title(f"exploring… {frame + 1}/{n_expand}")
        else:
            cell = path[frame - n_expand]
            if cell not in (start, goal):
                img[cell] = _PATH_RGB
            cost = result.cost if result.found else float("inf")
            ax.set_title(f"path found — cost={cost}")
        mark_endpoints()
        im.set_data(img)
        return (im,)

    return FuncAnimation(fig, update, frames=total, interval=interval, blit=False)


def plot_frontier(
    result, ax: matplotlib.axes.Axes | None = None, label=None
) -> matplotlib.axes.Axes:
    """Plot frontier size per expansion step — the memory profile of the search
    (the graph-search analogue of a convergence curve)."""
    import matplotlib.pyplot as plt

    sizes = [fs for _node, _g, fs in result.trace]
    if not sizes:
        raise ValueError("empty trace; run the search with record=True")
    if ax is None:
        _, ax = plt.subplots()
    ax.plot(sizes, label=label)
    ax.set_xlabel("expansion step")
    ax.set_ylabel("frontier size")
    ax.set_title("Frontier size over the search")
    if label:
        ax.legend()
    return ax


def compare(results) -> matplotlib.figure.Figure:
    """Bar charts comparing algorithms on the same problem.

    Args:
        results (dict[str, SearchResult]): ``{algorithm_name: result}``.

    Returns:
        matplotlib.figure.Figure: two panels — nodes expanded, and path cost.
    """
    import matplotlib.pyplot as plt

    names = list(results)
    expanded = [results[n].nodes_expanded for n in names]
    costs = [results[n].cost if results[n].found else float("nan") for n in names]
    logger.info("compare: %d algorithms", len(names))

    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(10, 4))
    ax1.bar(names, expanded, color="#90caf9", edgecolor="#37474f")
    ax1.set_ylabel("nodes expanded")
    ax1.set_title("Work (lower is better)")
    ax2.bar(names, costs, color="#fbc02d", edgecolor="#37474f")
    ax2.set_ylabel("path cost")
    ax2.set_title("Quality (lower is better)")
    for ax in (ax1, ax2):
        ax.tick_params(axis="x", rotation=45)
    fig.tight_layout()
    return fig


def _layout(num_nodes, edges):
    """Node positions: networkx spring layout if available, else a circle."""
    import numpy as np

    try:
        import networkx as nx

        g = nx.Graph()
        g.add_nodes_from(range(num_nodes))
        g.add_edges_from((u, v) for u, v, *_ in edges)
        pos = nx.spring_layout(g, seed=0)
        return {n: (float(p[0]), float(p[1])) for n, p in pos.items()}
    except ImportError:
        logger.info("networkx not installed; falling back to a circular layout")
        ang = np.linspace(0, 2 * np.pi, num_nodes, endpoint=False)
        return {n: (float(np.cos(a)), float(np.sin(a))) for n, a in enumerate(ang)}


def plot_graph(
    num_nodes, edges, result, ax: matplotlib.axes.Axes | None = None, node_size=80
) -> matplotlib.axes.Axes:
    """Draw an explicit graph with nodes coloured by their role in the search:
    grey = untouched, blue = expanded, gold = on the path, green/red = start/goal.

    Args:
        num_nodes (int): number of nodes (ids ``0..num_nodes``).
        edges (list): ``(u, v, weight)`` tuples (as returned by the generators).
        result (SearchResult): from ``search_graph`` (run with ``record=True``).
        ax (matplotlib.axes.Axes, optional): axes to draw on; created if omitted.
        node_size (int): scatter marker size.

    Returns:
        matplotlib.axes.Axes: the axes the graph was drawn on.
    """
    import matplotlib.pyplot as plt

    pos = _layout(num_nodes, edges)
    path = list(result.path or [])
    visited = {node for node, _g, _fs in result.trace}
    on_path = set(path)
    start = path[0] if path else None
    goal = path[-1] if path else None

    if ax is None:
        _, ax = plt.subplots()
    # edges first, behind the nodes
    for u, v, *_ in edges:
        (x0, y0), (x1, y1) = pos[u], pos[v]
        ax.plot([x0, x1], [y0, y1], color="#cfd8dc", linewidth=0.6, zorder=1)

    def colour(n):
        if n == start:
            return _START_RGB
        if n == goal:
            return _GOAL_RGB
        if n in on_path:
            return _PATH_RGB
        if n in visited:
            return _VISITED_RGB
        return (0.812, 0.847, 0.863)  # untouched grey

    xs = [pos[n][0] for n in range(num_nodes)]
    ys = [pos[n][1] for n in range(num_nodes)]
    ax.scatter(xs, ys, s=node_size, c=[colour(n) for n in range(num_nodes)],
               edgecolors="#37474f", linewidths=0.5, zorder=2)
    ax.set_xticks([])
    ax.set_yticks([])
    cost = result.cost if result.found else float("inf")
    ax.set_title(f"expanded={result.nodes_expanded}  cost={cost}")
    logger.info("plot_graph: %d nodes, %d visited", num_nodes, len(visited))
    return ax


def _node_key(n):
    """Hashable key for a node (tuples come back as tuples, ints/str as-is)."""
    return tuple(n) if isinstance(n, list) else n


def _node_label(n):
    """Compact label: words and ints verbatim, coordinate tuples as ``r,c``."""
    if isinstance(n, tuple):
        return ",".join(str(x) for x in n)
    return str(n)


def plot_search_tree(
    result,
    ax: matplotlib.axes.Axes | None = None,
    node_size=300,
    with_labels=None,
    max_label_nodes=60,
) -> matplotlib.axes.Axes:
    """Draw the **search tree**: every generated node linked to its best parent,
    laid out top-down with the start at the root and the solution path in gold.

    Needs ``result.tree``, recorded only when the search ran with ``record=True``
    (the default). It is empty for the iterative-deepening and bidirectional
    algorithms, which keep no persistent parent map.

    Args:
        result (SearchResult): a run with ``record=True`` and a non-empty tree.
        ax (matplotlib.axes.Axes, optional): axes to draw on; created if omitted.
        node_size (int): scatter marker size.
        with_labels (bool | None): draw node labels; ``None`` shows them only
            when the tree has at most ``max_label_nodes`` nodes.
        max_label_nodes (int): label-count threshold used when ``with_labels`` is
            ``None``.

    Returns:
        matplotlib.axes.Axes: the axes the tree was drawn on.
    """
    import matplotlib.pyplot as plt

    edges = [(_node_key(p), _node_key(c)) for p, c in result.tree]
    if not edges:
        raise ValueError(
            "no search tree to draw; run with record=True and a main algorithm "
            "(bfs/dfs/ucs/greedy/astar/weighted_astar — not iddfs/ida_star/bidirectional)"
        )

    children, nodes, child_set = {}, set(), set()
    for p, c in edges:
        children.setdefault(p, []).append(c)
        nodes.update((p, c))
        child_set.add(c)

    path = [_node_key(n) for n in (result.path or [])]
    root = path[0] if path else next(iter(nodes - child_set), next(iter(nodes)))

    # Depth by BFS from the root.
    depth = {root: 0}
    queue = deque([root])
    while queue:
        cur = queue.popleft()
        for kid in sorted(children.get(cur, [])):
            if kid not in depth:
                depth[kid] = depth[cur] + 1
                queue.append(kid)

    # Tidy x-coordinates by an iterative post-order: leaves get sequential slots,
    # internal nodes sit above the mean of their children.
    pos = {}
    next_leaf = [0]
    stack, done = [(root, False)], set()
    post = []
    while stack:
        node, processed = stack.pop()
        if processed:
            post.append(node)
            continue
        if node in done:
            continue
        done.add(node)
        stack.append((node, True))
        for kid in sorted(children.get(node, []), reverse=True):
            stack.append((kid, False))
    for node in post:
        kids = children.get(node, [])
        if kids:
            x = sum(pos[k][0] for k in kids) / len(kids)
        else:
            x = next_leaf[0]
            next_leaf[0] += 1
        pos[node] = (x, -depth.get(node, 0))

    if ax is None:
        _, ax = plt.subplots()
    for p, c in edges:
        if p in pos and c in pos:
            (x0, y0), (x1, y1) = pos[p], pos[c]
            ax.plot([x0, x1], [y0, y1], color="#cfd8dc", linewidth=0.8, zorder=1)

    path_set, goal = set(path), (path[-1] if path else None)

    def colour(n):
        if n == root:
            return _START_RGB
        if n == goal:
            return _GOAL_RGB
        if n in path_set:
            return _PATH_RGB
        return _VISITED_RGB

    laid = list(pos)
    ax.scatter(
        [pos[n][0] for n in laid],
        [pos[n][1] for n in laid],
        s=node_size,
        c=[colour(n) for n in laid],
        edgecolors="#37474f",
        linewidths=0.5,
        zorder=2,
    )
    show = with_labels if with_labels is not None else (len(laid) <= max_label_nodes)
    if show:
        for n, (x, y) in pos.items():
            ax.annotate(_node_label(n), (x, y), ha="center", va="center", fontsize=7, zorder=3)
    ax.set_xticks([])
    ax.set_yticks([])
    ax.set_title(f"search tree — {len(laid)} nodes, depth {max(depth.values(), default=0)}")
    logger.info("plot_search_tree: %d nodes", len(laid))
    return ax
