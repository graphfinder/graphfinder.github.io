# Integrations

`graphfinder.integrations` bridges graphfinder's search to popular libraries.
Each integration is an **optional extra** and imports its dependency lazily — so
importing graphfinder pulls in none of them:

```bash
pip install "graphfinder[networkx]"   # search over networkx graphs
pip install "graphfinder[scipy]"      # search over scipy.sparse adjacency
pip install "graphfinder[pandas]"     # edge-list DataFrames + result tables
pip install "graphfinder[osm]"        # route on real road networks (OSMnx)
```

Every `search` helper returns a `LabeledResult`: the `path` is mapped back to
your original node labels, while the native `SearchResult` stays in `.raw` (for
metrics and [`graphfinder.viz`](visualization.md)).

```python
r.path            # ["A", "B", "C", ...]  (your labels)
r.cost, r.found, r.nodes_expanded, r.stop_reason
r.raw             # native SearchResult (integer-id path/trace, for viz)
```

## NetworkX

A drop-in alternative to `nx.shortest_path` / `nx.astar_path` that also reports
search instrumentation (nodes expanded, frontier peak, trace). Directed graphs
are honoured automatically; edge weights come from the `weight` attribute.

```python
import networkx as nx
from graphfinder.integrations import networkx as gfnx

g = nx.Graph()
g.add_weighted_edges_from([("A", "B", 1), ("B", "C", 1), ("A", "C", 5)])

r = gfnx.search(g, "A", "C", algorithm="dijkstra")
r.path            # ['A', 'B', 'C']
r.cost            # 2.0

# A* with a custom heuristic over your node labels:
gfnx.search(g, "A", "C", algorithm="astar", heuristic=lambda u, v: 0.0)
```

Use `gfnx.to_edgelist(g)` if you just want the `(num_nodes, edges, index,
labels, directed)` mapping.

## SciPy (`scipy.sparse`)

Search directly over a sparse adjacency matrix — comparable to
`scipy.sparse.csgraph.dijkstra` / `shortest_path`, with the same `directed`
convention (default `True`). Node ids are the matrix indices.

```python
import scipy.sparse as sp
from graphfinder.integrations import scipy as gfsp

adj = sp.csr_matrix([[0, 1, 5], [0, 0, 1], [0, 0, 0]], dtype=float)
r = gfsp.search(adj, source=0, target=2, algorithm="dijkstra", directed=True)
r.path            # [0, 1, 2]
r.cost            # 2.0
```

## pandas

Build a graph from an edge-list `DataFrame`, and turn results into tidy tables.

```python
import pandas as pd
from graphfinder.integrations import pandas as gfpd

df = pd.DataFrame(
    [("A", "B", 1), ("B", "C", 1), ("A", "C", 5)],
    columns=["from", "to", "cost"],
)

r = gfpd.search(df, "A", "C", source="from", target="to", weight="cost",
                algorithm="dijkstra", record=True)
r.path            # ['A', 'B', 'C']

gfpd.trace_dataframe(r)       # step, node, g, frontier_size  (one row per expansion)
gfpd.compare_dataframe({      # one row per algorithm
    "bfs": gfpd.search(df, "A", "C", source="from", target="to"),
    "dijkstra": r,
})
```

`compare_dataframe` columns: `found, cost, nodes_expanded, nodes_generated,
max_frontier_size, path_len` — the tabular companion to
[`viz.compare`](visualization.md).

## OSMnx — routing on real road networks

Route over geographic graphs with an A\* that uses a great-circle (haversine)
heuristic. The core `search` works on **any** networkx graph whose nodes have
`x` (lon) / `y` (lat) attributes — as OSMnx graphs do — and needs only
`networkx`. The `route` / `plot_route` helpers snap lat/lon points to nodes and
draw the route on the map, and require `osmnx` (`pip install "graphfinder[osm]"`).

```python
import osmnx as ox
from graphfinder.integrations import osm

G = ox.graph_from_place("Madrid, Spain", network_type="drive")

# Route between two (lat, lon) points — nearest nodes are found automatically.
r = osm.route(G, (40.4170, -3.7035), (40.4531, -3.6883))   # Sol → Bernabéu
print(r.cost, "metres,", len(r.path), "nodes,", r.nodes_expanded, "expanded")
osm.plot_route(G, r)

# Or search between known node ids directly (networkx-only):
osm.search(G, orig_id, dest_id, algorithm="astar")
```

Because the haversine heuristic is in metres and OSMnx edge `length` is in metres,
A\* is admissible and returns the shortest route while expanding far fewer nodes
than Dijkstra.

## More integrations

The roadmap also includes a **PyTorch** learned-heuristic tutorial (plugging a
neural estimate into A\* via the [custom heuristic](heuristics.md) hook) and a
**LangChain/LangGraph** routing tool. Open an issue if you'd like one prioritised.
