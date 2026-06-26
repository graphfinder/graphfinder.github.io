"""Python-side tests for the graphfinder binding. Run with: pytest -q

Requires the extension to be built: `maturin develop --release`.
"""
import math

import pytest

import graphfinder as gf


def test_grid_astar_is_optimal():
    maze = gf.sample_maze("wall")
    bfs = gf.search(maze, algorithm="bfs", heuristic="zero")
    astar = gf.search(maze, algorithm="astar", heuristic="manhattan")
    assert astar.found and bfs.found
    assert astar.cost == bfs.cost
    # path is a list of (row, col) tuples
    assert isinstance(astar.path[0], tuple) and len(astar.path[0]) == 2


def test_trace_recording_toggle():
    maze = gf.sample_maze("open")
    on = gf.search(maze, algorithm="astar", record=True)
    off = gf.search(maze, algorithm="astar", record=False)
    assert len(on.trace) == on.nodes_expanded
    assert len(off.trace) == 0
    # each trace item is (node, g, frontier_size)
    node, g, fsize = on.trace[0]
    assert isinstance(fsize, int) and g >= 0.0


def test_node_budget_stops_early():
    maze = gf.sample_maze("wall")
    r = gf.search(maze, algorithm="bfs", heuristic="zero", max_nodes=3)
    assert not r.found
    assert r.stop_reason == "node_limit"
    assert r.nodes_expanded == 3


def test_unknown_algorithm_raises():
    with pytest.raises(ValueError):
        gf.search(gf.sample_maze("open"), algorithm="nope")


def test_unreachable_goal():
    maze = "S....\n....#\n...#G\n....#"
    r = gf.search(maze, algorithm="bfs", heuristic="zero")
    assert not r.found
    assert r.stop_reason == "exhausted"
    assert math.isinf(r.cost)


def test_explicit_graph_bidirectional_agrees_with_bfs():
    edges = gf.gen_barabasi_albert(150, 3, seed=11)
    bfs = gf.search_graph(150, edges, 0, 149, algorithm="bfs")
    bidi = gf.search_graph(150, edges, 0, 149, algorithm="bidirectional")
    assert bfs.found and bidi.found
    assert bfs.cost == bidi.cost
    assert isinstance(bfs.path[0], int)


def test_implicit_graph_bfs_minimal_ops():
    # Reach 27 from 1 using +1 and *2; BFS finds the fewest operations.
    def succ(s):
        return [(s + 1, 1.0), (s * 2, 1.0)] if s < 100 else []

    r = gf.search(succ, start=1, goal=27, algorithm="bfs")
    assert r.found
    assert r.cost == 7.0
    assert r.path[0] == 1 and r.path[-1] == 27


def test_grid_custom_heuristic_matches_named():
    # A custom callable equal to Manhattan must reproduce the named result.
    maze = gf.sample_maze("wall")
    named = gf.search(maze, algorithm="astar", heuristic="manhattan")

    def manhattan(node, goal):
        return abs(node[0] - goal[0]) + abs(node[1] - goal[1])

    custom = gf.search(maze, algorithm="astar", heuristic=manhattan)
    assert custom.found
    assert custom.cost == named.cost
    assert custom.nodes_expanded == named.nodes_expanded


def test_weighted_grid_ascii_digits():
    # Top row is expensive (cost 9); bottom row is a cheap detour.
    maze = "S99G\n1111"
    bfs = gf.search(maze, algorithm="bfs", heuristic="zero")
    ucs = gf.search(maze, algorithm="ucs", heuristic="zero")
    assert bfs.cost == 19.0 and len(bfs.path) == 4  # fewest steps, expensive
    assert ucs.cost == 5.0 and len(ucs.path) == 6  # least cost, longer
    assert ucs.cost < bfs.cost


def test_search_grid_costs_matrix():
    # 0.0 is a wall; A* must detour around it and pay the terrain.
    costs = [
        [1.0, 1.0, 1.0],
        [9.0, 0.0, 1.0],
        [1.0, 1.0, 1.0],
    ]
    r = gf.search_grid_costs(costs, start=(0, 0), goal=(2, 0), algorithm="astar")
    assert r.found
    # (0,0)->(0,1)1->(0,2)1->(1,2)1->(2,2)1->(2,1)1->(2,0)1 = 6  beats going through 9
    assert r.cost == 6.0


def test_search_grid_costs_rejects_wall_endpoint():
    costs = [[1.0, 1.0], [0.0, 1.0]]
    with pytest.raises(ValueError):
        gf.search_grid_costs(costs, start=(1, 0), goal=(0, 0))  # start is a wall


def test_grid_invalid_heuristic_raises():
    with pytest.raises(ValueError):
        gf.search(gf.sample_maze("open"), algorithm="astar", heuristic=42)


def test_graph_custom_heuristic_runs_astar():
    # A* on an explicit graph with a (trivially admissible) custom heuristic.
    edges = gf.gen_barabasi_albert(120, 3, seed=4)
    bfs = gf.search_graph(120, edges, 0, 119, algorithm="bfs")
    astar = gf.search_graph(
        120, edges, 0, 119, algorithm="astar", heuristic=lambda n, goal: 0.0
    )
    assert astar.found
    assert astar.cost == bfs.cost  # zero heuristic ⇒ optimal, same as BFS on unit costs


def test_implicit_graph_astar_with_heuristic():
    def succ(s):
        return [(s + 1, 1.0), (s * 2, 1.0)] if s < 100 else []

    def h(s, goal):
        return 0.0 if s >= goal else 1.0  # admissible: at least one op left

    r = gf.search(succ, start=1, goal=27, algorithm="astar", heuristic=h)
    assert r.found and r.cost == 7.0


def test_generators_are_reproducible():
    a = gf.gen_erdos_renyi(80, 0.1, 5)
    b = gf.gen_erdos_renyi(80, 0.1, 5)
    assert a == b
    assert all(u < v for (u, v, _) in a)  # unique undirected edges


def test_random_maze_ascii_roundtrips():
    m = gf.random_maze_ascii(12, 12, 0.25, seed=3)
    assert "S" in m and "G" in m
    r = gf.search(m, algorithm="astar")  # may or may not be solvable
    assert r.stop_reason in ("goal", "exhausted")
