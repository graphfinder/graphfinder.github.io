"""graphfinder — graph traversal & pathfinding with a Rust core, Python API.

The computation runs in Rust (native module ``graphfinder_native``); this Python
layer adds an ergonomic ``search`` dispatcher and (Phase 4) visualization.

Example:
    >>> import graphfinder as gf
    >>> r = gf.search(gf.sample_maze("open"), algorithm="astar")
    >>> r.found
    True

    >>> # explicit weighted graph
    >>> edges = gf.gen_barabasi_albert(100, 3, seed=1)
    >>> r = gf.search_graph(100, edges, start=0, goal=99, algorithm="bidirectional")

    >>> # implicit graph (states are ints or tuples of ints)
    >>> def succ(s):
    ...     return [(s + 1, 1.0), (s * 2, 1.0)] if s < 50 else []
    >>> r = gf.search(succ, start=1, goal=27, algorithm="bfs")

    >>> # weighted terrain: digits 1-9 are costs, or a full matrix
    >>> gf.search("S99G\n1111", algorithm="ucs").cost   # 5.0
    >>> gf.search_grid_costs([[1, 1], [9, 1]], (0, 0), (1, 1))

Algorithms (`algorithm=`): "bfs", "dfs", "ucs"/"dijkstra", "greedy", "astar",
"weighted_astar", "iddfs", "dls", "ida_star", "beam", "bidirectional".
Heuristics (`heuristic=`): a built-in grid name ("zero", "manhattan",
"euclidean", "octile") *or* a custom callable ``h(node, goal) -> float`` (works
in any domain — grids, explicit graphs and implicit graphs).
"""

import logging

from .graphfinder_native import (
    SearchResult,
    gen_barabasi_albert,
    gen_erdos_renyi,
    gen_watts_strogatz,
    random_maze_ascii,
    sample_maze,
    search_graph,
    search_grid,
    search_grid_costs,
    search_implicit,
)
from . import integrations, viz

# Library best practice: never emit log output on import; the application opts in.
logging.getLogger(__name__).addHandler(logging.NullHandler())

__version__ = "0.6.0"


def search(domain, start=None, goal=None, **kwargs):
    """Convenience dispatcher over the domain type.

    - ``domain`` is a ``str`` → treated as an ASCII maze map (``search_grid``).
    - ``domain`` is callable → an implicit graph successor function
      (``search_implicit``); ``start`` and ``goal`` are required.

    For explicit graphs use :func:`search_graph` directly. Extra keyword
    arguments are forwarded to the underlying function.
    """
    if isinstance(domain, str):
        return search_grid(domain, **kwargs)
    if callable(domain):
        if start is None or goal is None:
            raise ValueError("implicit-graph search requires start and goal")
        return search_implicit(domain, start, goal, **kwargs)
    raise TypeError(
        "domain must be an ASCII map (str) or a successor callable; "
        "use search_graph(...) for explicit edge-list graphs"
    )


__all__ = [
    "search",
    "search_grid",
    "search_grid_costs",
    "search_graph",
    "search_implicit",
    "gen_erdos_renyi",
    "gen_barabasi_albert",
    "gen_watts_strogatz",
    "sample_maze",
    "random_maze_ascii",
    "SearchResult",
    "viz",
    "integrations",
    "__version__",
]
