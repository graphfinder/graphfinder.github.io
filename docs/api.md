# API reference

Reference generated from the docstrings of the `graphfinder` package.

## Search

::: graphfinder
    options:
      members:
        - search
        - search_grid
        - search_grid_costs
        - search_graph
        - search_implicit
        - SearchResult

## Puzzles

Implicit state-space domains. See [Implicit puzzles](puzzles.md).

::: graphfinder
    options:
      show_root_heading: false
      members:
        - search_npuzzle
        - search_hanoi
        - search_wordladder

## Shortest paths

Negative-weight shortest-path algorithms. See [Shortest paths](shortest-paths.md).

::: graphfinder
    options:
      show_root_heading: false
      members:
        - bellman_ford
        - floyd_warshall
        - ShortestPaths
        - AllPairs

## Instances

::: graphfinder
    options:
      show_root_heading: false
      members:
        - sample_maze
        - random_maze_ascii
        - gen_erdos_renyi
        - gen_barabasi_albert
        - gen_watts_strogatz

## Visualization

::: graphfinder.viz
    options:
      members:
        - animate_grid
        - plot_grid
        - plot_costs
        - plot_frontier
        - compare
        - plot_graph
        - plot_search_tree

## Integrations

See [Integrations](integrations.md) for usage. Each returns a `LabeledResult`.

::: graphfinder.integrations
    options:
      members:
        - LabeledResult

::: graphfinder.integrations.networkx
    options:
      members:
        - search
        - to_edgelist

::: graphfinder.integrations.scipy
    options:
      members:
        - search

::: graphfinder.integrations.pandas
    options:
      members:
        - search
        - trace_dataframe
        - compare_dataframe

::: graphfinder.integrations.osm
    options:
      members:
        - search
        - route
        - plot_route
        - haversine

::: graphfinder.integrations.agents
    options:
      members:
        - make_router
        - as_langchain_tool

::: graphfinder.integrations.gym
    options:
      members:
        - GridWorldEnv
        - optimal_path
        - optimal_action

::: graphfinder.integrations.graphviz
    options:
      members:
        - to_dot
        - source
