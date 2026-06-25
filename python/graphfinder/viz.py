"""Visualization helpers. Consume the `SearchResult` from the Rust core.

Requires matplotlib (``pip install graphfinder[viz]``). Functions:
  - animate_grid(map, result)          # the flagship: watch the search explore
  - plot_grid(map, result, ax=None)    # static snapshot (visited + path)
  - plot_frontier(result, ax=None)     # frontier size per expansion (memory)
  - compare(results)                   # bar charts across algorithms
  - plot_graph(n, edges, result)       # general graph, nodes coloured by state

matplotlib is imported lazily inside each function, so importing this module is
cheap and never required for the core search API.
"""
import logging

logger = logging.getLogger(__name__)

# Cell state codes and their colours.
#   0 free   1 wall   2 visited/expanded   3 path   4 start   5 goal
_GRID_COLORS = ["#f5f5f5", "#37474f", "#90caf9", "#fbc02d", "#43a047", "#e53935"]
_FREE, _WALL, _VISITED, _PATH, _START, _GOAL = range(6)


def _parse_map(map_str):
    """Parse an ASCII map into ``(width, height, walls, start, goal)``."""
    lines = [ln for ln in map_str.splitlines() if ln]
    height = len(lines)
    width = max((len(ln) for ln in lines), default=0)
    walls, start, goal = set(), None, None
    for r, line in enumerate(lines):
        for c, ch in enumerate(line):
            if ch == "#":
                walls.add((r, c))
            elif ch == "S":
                start = (r, c)
            elif ch == "G":
                goal = (r, c)
    return width, height, walls, start, goal


def _base_grid(width, height, walls):
    import numpy as np

    grid = np.zeros((height, width), dtype=int)
    for (r, c) in walls:
        grid[r, c] = _WALL
    return grid


def _grid_image(ax, grid):
    from matplotlib.colors import BoundaryNorm, ListedColormap

    cmap = ListedColormap(_GRID_COLORS)
    norm = BoundaryNorm(range(len(_GRID_COLORS) + 1), cmap.N)
    im = ax.imshow(grid, cmap=cmap, norm=norm, interpolation="nearest")
    ax.set_xticks([])
    ax.set_yticks([])
    return im


def _mark_endpoints(grid, start, goal):
    if start is not None:
        grid[start] = _START
    if goal is not None:
        grid[goal] = _GOAL


def plot_grid(map, result, ax=None):
    """Static snapshot: walls, every expanded cell shaded, the path on top.

    Args:
        map (str): the ASCII map searched.
        result (SearchResult): from a search run with ``record=True``.
        ax: optional matplotlib Axes.

    Returns:
        The matplotlib Axes.
    """
    import matplotlib.pyplot as plt

    width, height, walls, start, goal = _parse_map(map)
    grid = _base_grid(width, height, walls)
    for node, _g, _fs in result.trace:
        grid[tuple(node)] = _VISITED
    for node in result.path or []:
        grid[tuple(node)] = _PATH
    _mark_endpoints(grid, start, goal)

    if ax is None:
        _, ax = plt.subplots()
    _grid_image(ax, grid)
    cost = result.cost if result.found else float("inf")
    ax.set_title(f"expanded={result.nodes_expanded}  cost={cost}")
    logger.info("plot_grid: %dx%d, %d expanded", width, height, result.nodes_expanded)
    return ax


def animate_grid(map, result, interval=60, show_path=True):
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
        A ``matplotlib.animation.FuncAnimation`` (in a notebook, display with
        ``HTML(anim.to_jshtml())``).
    """
    import matplotlib.pyplot as plt
    from matplotlib.animation import FuncAnimation

    width, height, walls, start, goal = _parse_map(map)
    expanded = [tuple(node) for node, _g, _fs in result.trace]
    if not expanded:
        raise ValueError("empty trace; run the search with record=True")
    path = [tuple(n) for n in (result.path or [])] if show_path else []

    grid = _base_grid(width, height, walls)
    _mark_endpoints(grid, start, goal)
    fig, ax = plt.subplots()
    im = _grid_image(ax, grid)
    n_expand, n_path = len(expanded), len(path)
    total = n_expand + n_path
    logger.info("animate_grid: %d frames (%d expand + %d path)", total, n_expand, n_path)

    def update(frame):
        if frame < n_expand:
            cell = expanded[frame]
            if cell not in (start, goal):
                grid[cell] = _VISITED
            ax.set_title(f"exploring… {frame + 1}/{n_expand}")
        else:
            cell = path[frame - n_expand]
            if cell not in (start, goal):
                grid[cell] = _PATH
            cost = result.cost if result.found else float("inf")
            ax.set_title(f"path found — cost={cost}")
        _mark_endpoints(grid, start, goal)
        im.set_data(grid)
        return (im,)

    return FuncAnimation(fig, update, frames=total, interval=interval, blit=False)


def plot_frontier(result, ax=None, label=None):
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


def compare(results):
    """Bar charts comparing algorithms on the same problem.

    Args:
        results (dict[str, SearchResult]): ``{algorithm_name: result}``.

    Returns:
        The matplotlib Figure (two panels: nodes expanded, and path cost).
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


def plot_graph(num_nodes, edges, result, ax=None, node_size=80):
    """Draw an explicit graph with nodes coloured by their role in the search:
    grey = untouched, blue = expanded, gold = on the path, green/red = start/goal.

    Args:
        num_nodes (int): number of nodes (ids ``0..num_nodes``).
        edges (list): ``(u, v, weight)`` tuples (as returned by the generators).
        result (SearchResult): from ``search_graph`` (run with ``record=True``).
        ax: optional matplotlib Axes.
        node_size (int): scatter marker size.

    Returns:
        The matplotlib Axes.
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
            return _GRID_COLORS[_START]
        if n == goal:
            return _GRID_COLORS[_GOAL]
        if n in on_path:
            return _GRID_COLORS[_PATH]
        if n in visited:
            return _GRID_COLORS[_VISITED]
        return "#cfd8dc"

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
