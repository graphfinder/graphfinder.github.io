"""pandas integration: build a graph from an edge-list ``DataFrame`` and turn
search results into tidy tables.

    from graphfinder.integrations import pandas as gfpd
    r = gfpd.search(df, "A", "F", source="from", target="to", weight="cost")
    gfpd.trace_dataframe(r)         # per-expansion table
    gfpd.compare_dataframe(results) # one row per algorithm

Requires ``pandas`` (``pip install "graphfinder[pandas]"``).
"""
from . import _relabel, _require, _require_node


def to_edgelist(df, source="source", target="target", weight=None):
    """Map an edge-list DataFrame to ``(num_nodes, edges, index, labels)``.

    ``source``/``target`` name the endpoint columns; ``weight`` (optional) names
    the weight column (absent ⇒ unit weights). Node labels are whatever appears
    in those columns.
    """
    pd = _require("pandas", "pandas")
    # unique() keeps first-appearance order, so node ids are deterministic and
    # independent of label type. (Deliberately not Index.union(), which sorts and
    # would break on unorderable or mixed-type labels.)
    labels = list(pd.unique(pd.concat([df[source], df[target]], ignore_index=True)))
    index = {node: i for i, node in enumerate(labels)}
    if weight is None:
        edges = [(index[u], index[v], 1.0) for u, v in zip(df[source], df[target])]
    else:
        edges = [
            (index[u], index[v], float(w))
            for u, v, w in zip(df[source], df[target], df[weight])
        ]
    return len(labels), edges, index, labels


def search(
    df,
    source_node,
    target_node,
    source="source",
    target="target",
    weight=None,
    directed=False,
    algorithm="dijkstra",
    **kwargs,
):
    """Search a graph described by an edge-list DataFrame.

    ``source_node``/``target_node`` are the start/goal *labels*; ``source``/
    ``target``/``weight`` are *column names*. Extra keyword arguments pass through
    to :func:`graphfinder.search_graph`. Returns a
    :class:`graphfinder.integrations.LabeledResult` with the original labels.
    """
    from .. import search_graph

    n, edges, index, labels = to_edgelist(df, source, target, weight)
    s = _require_node(index, source_node, "source", "edge list")
    t = _require_node(index, target_node, "target", "edge list")
    raw = search_graph(
        n,
        edges,
        s,
        t,
        algorithm=algorithm,
        undirected=not directed,
        **kwargs,
    )
    return _relabel(raw, labels)


def _raw(result):
    """Accept a LabeledResult or a native SearchResult."""
    return getattr(result, "raw", None) or result


def trace_dataframe(result):
    """A tidy DataFrame of the per-expansion trace: ``step, node, g,
    frontier_size``. The search must have been run with ``record=True``."""
    pd = _require("pandas", "pandas")
    raw = _raw(result)
    rows = [
        {"step": i, "node": node, "g": g, "frontier_size": fs}
        for i, (node, g, fs) in enumerate(raw.trace)
    ]
    return pd.DataFrame(rows, columns=["step", "node", "g", "frontier_size"])


def compare_dataframe(results):
    """One row per algorithm: ``found, cost, nodes_expanded, nodes_generated,
    max_frontier_size, path_len``. ``results`` is a ``{name: result}`` dict of
    LabeledResult or native SearchResult."""
    pd = _require("pandas", "pandas")
    rows = []
    for name, result in results.items():
        raw = _raw(result)
        rows.append(
            {
                "algorithm": name,
                "found": raw.found,
                "cost": raw.cost if raw.found else float("nan"),
                "nodes_expanded": raw.nodes_expanded,
                "nodes_generated": raw.nodes_generated,
                "max_frontier_size": raw.max_frontier_size,
                "path_len": len(raw.path) if raw.path else 0,
            }
        )
    return pd.DataFrame(rows).set_index("algorithm")
