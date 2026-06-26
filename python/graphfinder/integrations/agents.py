"""Agent integration: expose graphfinder routing as a safe LLM tool.

Two layers:

- :func:`make_router` builds a plain, dependency-free ``router(source, target,
  algorithm=None) -> dict`` bound to a fixed graph. It validates inputs, caps the
  work with ``max_nodes`` and restricts the algorithm to a safe allow-list, so it
  is safe to hand to an autonomous agent.
- :func:`as_langchain_tool` wraps that router as a LangChain ``StructuredTool``
  (requires ``langchain-core``; ``pip install "graphfinder[agents]"``).

The graph is bound at tool-creation time; the agent only chooses ``source`` and
``target`` (and optionally an allowed ``algorithm``).
"""
from . import _require

#: Algorithms an agent is allowed to pick (safe, terminating on finite graphs).
SAFE_ALGORITHMS = ("bfs", "dijkstra", "ucs", "astar", "bidirectional")


def make_router(edges, directed=False, default_algorithm="dijkstra", max_nodes=200_000):
    """Build a bound ``router(source, target, algorithm=None) -> dict``.

    ``edges`` is a list of ``(u, v)`` or ``(u, v, weight)`` with arbitrary
    hashable labels. The returned function maps labels internally, runs the
    search with a hard ``max_nodes`` budget, and returns a JSON-friendly dict:
    ``{found, path, cost, nodes_expanded, stop_reason}`` (or ``{error: ...}`` for
    bad input) — never raises, so it is safe inside an agent loop.
    """
    from .. import search_graph

    labels = []
    index = {}

    def _id(node):
        if node not in index:
            index[node] = len(labels)
            labels.append(node)
        return index[node]

    triples = []
    for edge in edges:
        if len(edge) == 3:
            u, v, w = edge
        else:
            u, v = edge
            w = 1.0
        triples.append((_id(u), _id(v), float(w)))
    n = len(labels)

    def router(source, target, algorithm=None):
        algo = (algorithm or default_algorithm).lower()
        if algo not in SAFE_ALGORITHMS:
            return {"error": f"algorithm must be one of {list(SAFE_ALGORITHMS)}"}
        if source not in index:
            return {"error": f"unknown source node: {source!r}"}
        if target not in index:
            return {"error": f"unknown target node: {target!r}"}
        raw = search_graph(
            n,
            triples,
            index[source],
            index[target],
            algorithm=algo,
            undirected=not directed,
            record=False,
            max_nodes=max_nodes,
        )
        path = None if raw.path is None else [labels[i] for i in raw.path]
        return {
            "found": raw.found,
            "path": path,
            "cost": raw.cost if raw.found else None,
            "nodes_expanded": raw.nodes_expanded,
            "stop_reason": raw.stop_reason,
        }

    return router


def as_langchain_tool(
    edges,
    name="find_route",
    description=None,
    directed=False,
    **router_kwargs,
):
    """Wrap :func:`make_router` as a LangChain ``StructuredTool``.

    The graph is bound now; the agent supplies ``source``/``target`` (and an
    optional allowed ``algorithm``). Requires ``langchain-core``.
    """
    pydantic = _require("pydantic", "agents")
    lc_tools = _require("langchain_core.tools", "agents")

    router = make_router(edges, directed=directed, **router_kwargs)
    description = description or (
        "Find a route between two nodes of a fixed graph. "
        "Inputs: source and target node labels (and an optional algorithm from "
        f"{list(SAFE_ALGORITHMS)}). Returns the path, its cost and search stats."
    )

    class RouteInput(pydantic.BaseModel):
        source: str = pydantic.Field(description="start node label")
        target: str = pydantic.Field(description="goal node label")
        algorithm: str = pydantic.Field(
            default="dijkstra", description=f"one of {list(SAFE_ALGORITHMS)}"
        )

    def _run(source, target, algorithm="dijkstra"):
        return router(source, target, algorithm=algorithm)

    return lc_tools.StructuredTool.from_function(
        func=_run, name=name, description=description, args_schema=RouteInput
    )
