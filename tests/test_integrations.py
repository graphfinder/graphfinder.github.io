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


def test_missing_endpoint_raises_keyerror_naming_container():
    # The shared _require_node helper must raise KeyError naming the missing
    # endpoint and the right container ("graph" vs "edge list").
    nx = pytest.importorskip("networkx")
    from graphfinder.integrations import networkx as gfnx

    g = nx.Graph()
    g.add_weighted_edges_from(EDGES)
    with pytest.raises(KeyError, match="source node 'ZZZ' is not in the graph"):
        gfnx.search(g, "ZZZ", "E")

    pd = pytest.importorskip("pandas")
    from graphfinder.integrations import pandas as gfpd

    df = pd.DataFrame(EDGES, columns=["source", "target", "weight"])
    with pytest.raises(KeyError, match="target node 'ZZZ' is not in the edge list"):
        gfpd.search(df, "A", "ZZZ", weight="weight")


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


def test_osm_haversine():
    from graphfinder.integrations import osm

    assert osm.haversine(0, 0, 0, 0) == 0.0
    # ~1 degree of latitude ≈ 111 km.
    d = osm.haversine(0.0, 0.0, 1.0, 0.0)
    assert 110_000 < d < 112_000


def test_osm_geographic_astar():
    nx = pytest.importorskip("networkx")
    from graphfinder.integrations import osm

    # A geographic triangle: the direct edge 1→3 is shorter than 1→2→3.
    g = nx.Graph()
    g.add_node(1, x=0.0, y=0.0)
    g.add_node(2, x=0.0, y=0.01)  # north
    g.add_node(3, x=0.01, y=0.0)  # east
    pairs = [(1, 2), (2, 3), (1, 3)]
    for u, v in pairs:
        nu, nv = g.nodes[u], g.nodes[v]
        g.add_edge(u, v, length=osm.haversine(nu["y"], nu["x"], nv["y"], nv["x"]))

    r = osm.search(g, 1, 3, algorithm="astar")
    assert r.found
    assert r.path == [1, 3]  # direct beats the detour
    assert r.cost == pytest.approx(g[1][3]["length"])


def test_osm_route_requires_osmnx():
    nx = pytest.importorskip("networkx")
    from graphfinder.integrations import osm

    g = nx.Graph()
    g.add_node(1, x=0.0, y=0.0)
    g.add_node(2, x=0.01, y=0.0)
    g.add_edge(1, 2, length=1.0)
    try:
        import osmnx  # noqa: F401
    except ImportError:
        with pytest.raises(ImportError, match=r"graphfinder\[osm\]"):
            osm.route(g, (0.0, 0.0), (0.0, 0.01))


def test_agents_make_router():
    from graphfinder.integrations import agents

    router = agents.make_router(EDGES)
    out = router("A", "E", algorithm="dijkstra")
    assert out["found"] and out["path"] == ["A", "B", "C", "E"] and out["cost"] == 3.0
    # never raises on bad input — returns an error dict (safe for agents)
    assert "error" in router("A", "Z")  # unknown target
    assert "error" in router("A", "E", algorithm="rm -rf")  # disallowed algorithm


def test_agents_langchain_tool():
    pytest.importorskip("langchain_core")
    from graphfinder.integrations import agents

    tool = agents.as_langchain_tool(EDGES, name="route")
    assert tool.name == "route"
    res = tool.invoke({"source": "A", "target": "E"})
    assert res["found"] and res["path"][0] == "A" and res["path"][-1] == "E"


def test_gym_env_and_oracle():
    pytest.importorskip("gymnasium")
    from graphfinder.integrations import gym as gfgym

    maze = "S...\n.##.\n...G"
    env = gfgym.GridWorldEnv(maze)
    obs, info = env.reset()
    assert obs == 0  # start at (0,0) → index 0

    # Following the A* oracle must reach the goal with a finite reward.
    steps, terminated = 0, False
    total = 0.0
    while not terminated and steps < 100:
        action = gfgym.optimal_action(env)
        assert action is not None
        obs, reward, terminated, truncated, info = env.step(action)
        total += reward
        steps += 1
    assert terminated
    # Oracle path cost equals graphfinder's A* cost from the start.
    opt = gf.search(maze, algorithm="astar")
    assert steps == len(opt.path) - 1


def test_gym_illegal_move_penalty():
    pytest.importorskip("gymnasium")
    from graphfinder.integrations import gym as gfgym

    env = gfgym.GridWorldEnv("SG")  # 1x2: moving up (action 0) is illegal
    env.reset()
    _obs, reward, terminated, _trunc, _info = env.step(0)  # up into the wall/edge
    assert reward == -1.0 and not terminated


def test_graphviz_to_dot_no_dependency():
    from graphfinder.integrations import graphviz as gfgv

    edges = [("A", "B", 1.0), ("B", "C", 1.0), ("A", "C", 5.0)]

    class _R:  # minimal stand-in for a result with a path
        path = ["A", "B", "C"]

    dot = gfgv.to_dot(edges, _R(), directed=False)
    assert dot.startswith("graph G {")
    assert '"A" -- "B"' in dot
    assert "penwidth=3" in dot  # path edges highlighted
    assert "#43a047" in dot  # start node coloured green


def test_graphviz_escapes_backslash_and_quote():
    # A node id containing '\' and '"' must produce valid, correctly escaped DOT.
    from graphfinder.integrations import graphviz as gfgv

    dot = gfgv.to_dot([('a\\b', 'c"d', 1.0)], None, directed=True)
    assert '"a\\\\b"' in dot  # backslash escaped as \\
    assert '"c\\"d"' in dot  # quote escaped as \"


def test_graphviz_source_requires_pkg():
    pytest.importorskip("graphviz")
    from graphfinder.integrations import graphviz as gfgv

    src = gfgv.source([("A", "B", 1.0)])
    assert "A" in src.source


def test_lazy_attribute_access():
    # gf.integrations.<name> works without importing submodules explicitly.
    pytest.importorskip("networkx")
    assert hasattr(gf.integrations, "networkx")


def test_missing_dependency_message():
    # _require raises a helpful, install-hint error for an absent module.
    from graphfinder.integrations import _require

    with pytest.raises(ImportError, match=r"graphfinder\[nope\]"):
        _require("a_module_that_does_not_exist_xyz", "nope")
