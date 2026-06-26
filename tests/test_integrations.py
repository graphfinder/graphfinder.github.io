"""Tests for graphfinder.integrations (networkx, scipy, pandas).

Each test skips if its optional dependency is not installed.
"""
import pytest

import graphfinder as gf

# A small weighted graph used across the integrations.
#   A--1--B--1--C
#   |           |
#   4           1
#   |           |
#   D-----1-----E
EDGES = [("A", "B", 1.0), ("B", "C", 1.0), ("C", "E", 1.0), ("E", "D", 1.0), ("A", "D", 4.0)]
# Shortest A->E: A-B-C-E (cost 3) beats A-D-E (cost 5).


def test_networkx_search_matches_nx():
    nx = pytest.importorskip("networkx")
    from graphfinder.integrations import networkx as gfnx

    g = nx.Graph()
    g.add_weighted_edges_from(EDGES)
    r = gfnx.search(g, "A", "E", algorithm="dijkstra")
    assert r.found
    assert r.path == ["A", "B", "C", "E"]
    assert r.cost == 3.0
    # Drop-in agreement with networkx itself.
    assert r.cost == nx.shortest_path_length(g, "A", "E", weight="weight")


def test_networkx_directed_is_honoured():
    nx = pytest.importorskip("networkx")
    from graphfinder.integrations import networkx as gfnx

    g = nx.DiGraph()
    g.add_weighted_edges_from([("A", "B", 1.0), ("B", "C", 1.0)])
    assert gfnx.search(g, "A", "C", algorithm="bfs").found
    assert not gfnx.search(g, "C", "A", algorithm="bfs").found  # no reverse edge


def test_scipy_sparse_search_matches_csgraph():
    sp = pytest.importorskip("scipy.sparse")
    csg = pytest.importorskip("scipy.sparse.csgraph")
    from graphfinder.integrations import scipy as gfsp

    # 0->1->2 weights 1,1 ; direct 0->2 weight 5
    data = sp.csr_matrix([[0, 1, 5], [0, 0, 1], [0, 0, 0]], dtype=float)
    r = gfsp.search(data, 0, 2, algorithm="dijkstra", directed=True)
    assert r.found and r.path == [0, 1, 2] and r.cost == 2.0
    dist = csg.dijkstra(data, directed=True, indices=0)[2]
    assert r.cost == dist


def test_pandas_search_and_tables():
    pd = pytest.importorskip("pandas")
    from graphfinder.integrations import pandas as gfpd

    df = pd.DataFrame(EDGES, columns=["from", "to", "cost"])
    r = gfpd.search(
        df, "A", "E", source="from", target="to", weight="cost",
        algorithm="dijkstra", record=True,
    )
    assert r.found and r.path == ["A", "B", "C", "E"] and r.cost == 3.0

    trace = gfpd.trace_dataframe(r)
    assert list(trace.columns) == ["step", "node", "g", "frontier_size"]
    assert len(trace) == r.nodes_expanded

    results = {
        "bfs": gfpd.search(df, "A", "E", source="from", target="to"),
        "dijkstra": r,
    }
    table = gfpd.compare_dataframe(results)
    assert set(table.index) == {"bfs", "dijkstra"}
    assert "nodes_expanded" in table.columns


def test_lazy_attribute_access():
    # gf.integrations.<name> works without importing submodules explicitly.
    pytest.importorskip("networkx")
    assert hasattr(gf.integrations, "networkx")


def test_missing_dependency_message():
    # _require raises a helpful, install-hint error for an absent module.
    from graphfinder.integrations import _require

    with pytest.raises(ImportError, match=r"graphfinder\[nope\]"):
        _require("a_module_that_does_not_exist_xyz", "nope")
