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
