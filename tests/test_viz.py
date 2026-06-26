"""Tests for graphfinder.viz. Headless (Agg backend); they assert the helpers
build the expected objects, not pixel output. Run with: pytest -q
"""
import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt
import pytest
from matplotlib.animation import FuncAnimation

import graphfinder as gf

MAZE = gf.sample_maze("wall")


def teardown_function(_):
    plt.close("all")


def test_plot_grid_returns_axes():
    r = gf.search(MAZE, algorithm="astar", record=True)
    ax = gf.viz.plot_grid(MAZE, r)
    assert ax is not None
    # background image present
    assert len(ax.images) == 1


def test_animate_grid_frame_count():
    r = gf.search(MAZE, algorithm="astar", record=True)
    anim = gf.viz.animate_grid(MAZE, r, interval=10)
    assert isinstance(anim, FuncAnimation)
    expected = r.nodes_expanded + len(r.path)
    assert anim._save_count == expected or anim.save_count == expected


def test_animate_grid_requires_trace():
    r = gf.search(MAZE, algorithm="astar", record=False)
    with pytest.raises(ValueError):
        gf.viz.animate_grid(MAZE, r)


def test_plot_frontier_returns_axes():
    r = gf.search(MAZE, algorithm="ucs", heuristic="zero", record=True)
    ax = gf.viz.plot_frontier(r, label="UCS")
    assert ax.get_ylabel() == "frontier size"
    assert len(ax.lines) == 1


def test_compare_returns_figure():
    results = {
        "BFS": gf.search(MAZE, algorithm="bfs", heuristic="zero"),
        "A*": gf.search(MAZE, algorithm="astar", heuristic="manhattan"),
    }
    fig = gf.viz.compare(results)
    assert len(fig.axes) == 2


def test_plot_grid_weighted_ascii():
    r = gf.search("S99G\n1111", algorithm="astar", record=True)
    ax = gf.viz.plot_grid("S99G\n1111", r)
    assert len(ax.images) == 1


def test_plot_grid_from_cost_matrix():
    costs = [[1.0, 1.0, 1.0], [9.0, 0.0, 1.0], [1.0, 1.0, 1.0]]
    r = gf.search_grid_costs(costs, start=(0, 0), goal=(2, 0), algorithm="astar", record=True)
    ax = gf.viz.plot_grid(costs, r)  # cost matrix accepted directly
    assert len(ax.images) == 1


def test_plot_costs_heatmap():
    ax = gf.viz.plot_costs("S99G\n1111")
    assert len(ax.images) == 1  # the heatmap


def test_plot_graph_returns_axes():
    edges = gf.gen_barabasi_albert(40, 2, seed=1)
    r = gf.search_graph(40, edges, 0, 39, algorithm="bfs", record=True)
    ax = gf.viz.plot_graph(40, edges, r)
    assert len(ax.collections) == 1  # the node scatter
