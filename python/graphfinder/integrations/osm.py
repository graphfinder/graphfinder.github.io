"""Geographic routing on road networks (OSMnx).

Two layers:

- :func:`search` runs A* between two nodes of a **geographic** networkx graph —
  one whose nodes carry ``x`` (longitude) and ``y`` (latitude) attributes, as
  OSMnx graphs do — using a great-circle (haversine) heuristic. It only needs
  ``networkx``.
- :func:`route` and :func:`plot_route` are convenience wrappers around **OSMnx**
  (nearest-node lookup from lat/lon points, and map plotting). They require
  ``osmnx`` (``pip install "graphfinder[osm]"``).

The haversine heuristic is admissible when the edge ``weight`` is physical length
in metres (OSMnx's ``length``), so A* returns the shortest route.
"""
import math

from . import _relabel, _require

#: Mean Earth radius in metres.
EARTH_RADIUS_M = 6_371_000.0


def haversine(lat1, lon1, lat2, lon2):
    """Great-circle distance in **metres** between two ``(lat, lon)`` points."""
    phi1, phi2 = math.radians(lat1), math.radians(lat2)
    dphi = math.radians(lat2 - lat1)
    dlambda = math.radians(lon2 - lon1)
    a = math.sin(dphi / 2) ** 2 + math.cos(phi1) * math.cos(phi2) * math.sin(dlambda / 2) ** 2
    return 2 * EARTH_RADIUS_M * math.asin(math.sqrt(a))


def search(graph, orig, dest, weight="length", x="x", y="y", algorithm="astar", **kwargs):
    """A* between node ids ``orig`` and ``dest`` on a geographic networkx graph.

    Edge cost is read from the ``weight`` attribute (default ``"length"``); the
    heuristic is the haversine distance from each node's ``x``/``y`` (lon/lat)
    attributes to the goal. Directed graphs are honoured. Extra keyword arguments
    (``record``, ``max_nodes``, the ``weight`` multiplier of ``weighted_astar``,
    …) pass through to :func:`graphfinder.search_graph`.

    Returns a :class:`graphfinder.integrations.LabeledResult` whose ``path`` uses
    the original (OSM) node ids.
    """
    _require("networkx", "networkx")
    from .. import search_graph
    from .networkx import to_edgelist

    n, edges, index, labels, directed = to_edgelist(graph, weight)
    if orig not in index:
        raise KeyError(f"origin node {orig!r} is not in the graph")
    if dest not in index:
        raise KeyError(f"destination node {dest!r} is not in the graph")
    coords = graph.nodes
    gx, gy = coords[dest][x], coords[dest][y]

    def heuristic(i, _j):
        node = coords[labels[i]]
        return haversine(node[y], node[x], gy, gx)

    raw = search_graph(
        n,
        edges,
        index[orig],
        index[dest],
        algorithm=algorithm,
        undirected=not directed,
        heuristic=heuristic,
        **kwargs,
    )
    return _relabel(raw, labels)


def route(graph, orig_point, dest_point, weight="length", **kwargs):
    """Route between two ``(lat, lon)`` points on an OSMnx graph.

    Snaps each point to its nearest graph node (via ``osmnx``) and then calls
    :func:`search`. Requires ``osmnx``.
    """
    ox = _require("osmnx", "osm")
    orig = ox.distance.nearest_nodes(graph, X=orig_point[1], Y=orig_point[0])
    dest = ox.distance.nearest_nodes(graph, X=dest_point[1], Y=dest_point[0])
    return search(graph, orig, dest, weight=weight, **kwargs)


def plot_route(graph, result, **kwargs):
    """Plot a route over the OSMnx graph. ``result`` is a
    :class:`~graphfinder.integrations.LabeledResult` (or a list of node ids).
    Requires ``osmnx``; extra keyword arguments pass to ``ox.plot_graph_route``.
    """
    ox = _require("osmnx", "osm")
    nodes = getattr(result, "path", result)
    if not nodes:
        raise ValueError("no route to plot (result.path is empty)")
    return ox.plot_graph_route(graph, nodes, **kwargs)
