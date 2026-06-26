"""SciPy integration: run graphfinder's search over a ``scipy.sparse`` adjacency
matrix — a drop-in alternative to ``scipy.sparse.csgraph`` shortest-path routines
that also reports search instrumentation.

    from graphfinder.integrations import scipy as gfsp
    r = gfsp.search(adjacency_csr, source=0, target=n - 1, algorithm="dijkstra")

Requires ``scipy`` (``pip install "graphfinder[scipy]"``). Node ids are the
matrix indices ``0..n-1``.
"""
from . import _relabel, _require


def to_edgelist(matrix):
    """Map a square ``scipy.sparse`` matrix to ``(num_nodes, edges)``.

    Each stored entry ``(i, j) = w`` becomes a directed edge ``i → j`` of weight
    ``w``. Use a symmetric matrix (or ``directed=False`` in :func:`search`) for an
    undirected graph.
    """
    sp = _require("scipy.sparse", "scipy")
    coo = sp.coo_matrix(matrix)
    if coo.shape[0] != coo.shape[1]:
        raise ValueError("adjacency matrix must be square")
    n = int(coo.shape[0])
    edges = [
        (int(i), int(j), float(w))
        for i, j, w in zip(coo.row, coo.col, coo.data)
    ]
    return n, edges


def search(matrix, source, target, algorithm="dijkstra", directed=True, **kwargs):
    """Search for a path in a ``scipy.sparse`` adjacency matrix.

    ``directed`` defaults to ``True`` (matching ``scipy.sparse.csgraph``); set it
    to ``False`` to treat the matrix as undirected. Extra keyword arguments pass
    through to :func:`graphfinder.search_graph`.

    Returns a :class:`graphfinder.integrations.LabeledResult` (node ids are the
    matrix indices).
    """
    from .. import search_graph

    n, edges = to_edgelist(matrix)
    if not (0 <= source < n and 0 <= target < n):
        raise IndexError("source and target must be valid matrix indices")
    raw = search_graph(
        n,
        edges,
        source,
        target,
        algorithm=algorithm,
        undirected=not directed,
        **kwargs,
    )
    return _relabel(raw, list(range(n)))
