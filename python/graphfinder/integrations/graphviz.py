"""Graphviz integration: export a graph (and a found path) to DOT / SVG.

    from graphfinder.integrations import graphviz as gfgv
    dot = gfgv.to_dot(edges, result)          # a DOT string — no dependency
    gfgv.source(edges, result).render("route", format="svg")   # needs graphviz

``to_dot`` builds the DOT text itself, so it needs nothing; ``source`` wraps it in
a ``graphviz.Source`` you can render or display in a notebook (requires the
``graphviz`` Python package, and the Graphviz ``dot`` binary to render).
"""
from . import _require

_PATH_COLOR = "#fbc02d"
_START_COLOR = "#43a047"
_GOAL_COLOR = "#e53935"
_NODE_COLOR = "#cfd8dc"


def _nodes_in_order(edges):
    seen, order = set(), []
    for edge in edges:
        for node in (edge[0], edge[1]):
            if node not in seen:
                seen.add(node)
                order.append(node)
    return order


def to_dot(edges, result=None, directed=False, name="G"):
    """Build a Graphviz DOT string for an edge-list graph, highlighting the path
    from ``result`` (a ``LabeledResult`` or anything with a ``.path``).

    Nodes on the path are gold, the start green and the goal red; path edges are
    drawn thick. Edge weights (the optional 3rd item of each edge) become labels.
    """
    gtype, conn = ("digraph", "->") if directed else ("graph", "--")
    path = list(getattr(result, "path", None) or []) if result is not None else []
    path_set = set(path)
    path_edges = set(zip(path, path[1:]))
    start = path[0] if path else None
    goal = path[-1] if path else None

    def nid(x):
        # Escape backslashes first, then quotes, so a label containing '\' or '"'
        # produces valid DOT (order matters: escaping quotes first would then
        # double-escape the backslashes it introduces).
        escaped = str(x).replace("\\", "\\\\").replace('"', '\\"')
        return '"' + escaped + '"'

    lines = [f"{gtype} {name} {{", f'  node [style=filled fillcolor="{_NODE_COLOR}"];']
    for node in _nodes_in_order(edges):
        color = None
        if node == start:
            color = _START_COLOR
        elif node == goal:
            color = _GOAL_COLOR
        elif node in path_set:
            color = _PATH_COLOR
        if color is not None:
            lines.append(f'  {nid(node)} [fillcolor="{color}"];')
    for edge in edges:
        u, v = edge[0], edge[1]
        weight = edge[2] if len(edge) > 2 else None
        attrs = []
        on_path = (u, v) in path_edges or (not directed and (v, u) in path_edges)
        if on_path:
            attrs += [f'color="{_PATH_COLOR}"', "penwidth=3"]
        if weight is not None:
            attrs.append(f'label="{weight:g}"')
        suffix = f" [{' '.join(attrs)}]" if attrs else ""
        lines.append(f"  {nid(u)} {conn} {nid(v)}{suffix};")
    lines.append("}")
    return "\n".join(lines)


def source(edges, result=None, directed=False, name="G"):
    """Return a ``graphviz.Source`` for the graph (renderable / notebook-display).
    Requires the ``graphviz`` package (``pip install "graphfinder[graphviz]"``)."""
    gv = _require("graphviz", "graphviz")
    return gv.Source(to_dot(edges, result, directed=directed, name=name))
