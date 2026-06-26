"""Optional ecosystem integrations for graphfinder.

Each submodule bridges a popular library to graphfinder's search, and has its own
optional dependency:

    pip install "graphfinder[networkx]"   # search over networkx graphs
    pip install "graphfinder[scipy]"      # search over scipy.sparse adjacency
    pip install "graphfinder[pandas]"     # edge-list DataFrames + result tables

Imports are lazy: importing this package pulls in none of the extras. Access a
submodule by attribute (``gf.integrations.networkx``) or import it directly
(``from graphfinder.integrations import networkx``). The heavy dependency is only
needed when you actually call a function.

All ``search`` helpers return a :class:`LabeledResult`, which maps the path back
to your original node labels while keeping the native ``SearchResult`` in
``.raw`` (for metrics and ``graphfinder.viz``).
"""
import importlib
from dataclasses import dataclass, field
from typing import Any, Optional


@dataclass
class LabeledResult:
    """A search result with nodes mapped back to their original labels.

    Mirrors the native ``SearchResult`` fields, but ``path`` uses your labels
    (networkx nodes, DataFrame ids, …). The native result is kept in ``raw``
    (its ``path``/``trace`` use the internal integer ids).
    """

    path: Optional[list]
    cost: float
    found: bool
    nodes_expanded: int
    nodes_generated: int
    max_frontier_size: int
    stop_reason: str
    raw: Any = field(repr=False, default=None)

    def __repr__(self) -> str:
        cost = self.cost if self.found else float("inf")
        return (
            f"LabeledResult(found={self.found}, cost={cost}, "
            f"expanded={self.nodes_expanded}, stop={self.stop_reason})"
        )


def _relabel(raw, id_to_label) -> LabeledResult:
    """Wrap a native ``SearchResult``, mapping integer node ids back to labels."""
    path = None if raw.path is None else [id_to_label[i] for i in raw.path]
    return LabeledResult(
        path=path,
        cost=raw.cost,
        found=raw.found,
        nodes_expanded=raw.nodes_expanded,
        nodes_generated=raw.nodes_generated,
        max_frontier_size=raw.max_frontier_size,
        stop_reason=raw.stop_reason,
        raw=raw,
    )


def _require(module: str, extra: str):
    """Import an optional dependency, with a helpful message if it is missing."""
    try:
        return importlib.import_module(module)
    except ImportError as exc:  # pragma: no cover - exercised only without the dep
        raise ImportError(
            f"the '{module}' package is required for this integration; "
            f"install it with:  pip install 'graphfinder[{extra}]'"
        ) from exc


# PEP 562: let ``gf.integrations.networkx`` import the submodule on demand without
# pulling any extra at package-import time.
_SUBMODULES = ("networkx", "scipy", "pandas")


def __getattr__(name: str):
    if name in _SUBMODULES:
        return importlib.import_module(f"{__name__}.{name}")
    raise AttributeError(f"module {__name__!r} has no attribute {name!r}")


def __dir__():
    return sorted(list(globals()) + list(_SUBMODULES))


__all__ = ["LabeledResult"]
