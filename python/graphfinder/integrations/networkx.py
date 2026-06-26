"""NetworkX integration: run graphfinder's search over a ``networkx`` graph.

    from graphfinder.integrations import networkx as gfnx
    path = gfnx.search(G, source, target, algorithm="astar",
                       heuristic=lambda u, v: ...).path

Requires ``networkx`` (``pip install "graphfinder[networkx]"``).
"""
from . import _relabel, _require


def to_edgelist(graph, weight="weight"):
    """Map a networkx graph to graphfinder's edge-list form.

    Returns ``(num_nodes, edges, index, labels, directed)`` where ``index`` maps
    each node label to an integer id and ``labels`` is the reverse list. Edge
    weights are read from the ``weight`` attribute (default ``1.0``).
    """
    _require("networkx", "networkx")
    labels = list(graph.nodes())
    index = {node: i for i, node in enumerate(labels)}
    edges = [
        (index[u], index[v], float(data.get(weight, 1.0)))
        for u, v, data in graph.edges(data=True)
    ]
    return len(labels), edges, index, labels, graph.is_directed()


def search(graph, source, target, algorithm="dijkstra", weight="weight", **kwargs):
    """Search for a path from ``source`` to ``target`` in a networkx graph.

    A drop-in alternative to ``nx.shortest_path`` / ``nx.astar_path`` that also
    reports search instrumentation. Directed graphs are honoured automatically.
    Extra keyword arguments (``heuristic``, ``weight`` of ``weighted_astar``,
    ``beam_width``, ``max_nodes``, ``record``, …) pass through to
    :func:`graphfinder.search_graph`.

    Returns a :class:`graphfinder.integrations.LabeledResult` whose ``path`` uses
    the original networkx node labels.
    """
    from .. import search_graph

    n, edges, index, labels, directed = to_edgelist(graph, weight)
    if source not in index:
        raise KeyError(f"source node {source!r} is not in the graph")
    if target not in index:
        raise KeyError(f"target node {target!r} is not in the graph")
    raw = search_graph(
        n,
        edges,
        index[source],
        index[target],
        algorithm=algorithm,
        undirected=not directed,
        **kwargs,
    )
    return _relabel(raw, labels)
