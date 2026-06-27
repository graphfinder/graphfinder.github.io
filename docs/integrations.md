# Integrations

`graphfinder.integrations` bridges graphfinder's search to popular libraries.
Each integration is an **optional extra** and imports its dependency lazily — so
importing graphfinder pulls in none of them:

```bash
pip install "graphfinder[networkx]"   # search over networkx graphs
pip install "graphfinder[scipy]"      # search over scipy.sparse adjacency
pip install "graphfinder[pandas]"     # edge-list DataFrames + result tables
pip install "graphfinder[osm]"        # route on real road networks (OSMnx)
pip install "graphfinder[agents]"     # LangChain routing tool
pip install "graphfinder[gym]"        # GridWorld RL environment + A* oracle
pip install "graphfinder[graphviz]"   # export graphs/paths to DOT/SVG
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

## Agents (LangChain) — a safe routing tool

Expose routing as an LLM tool. `make_router` builds a dependency-free, bound
`router(source, target, algorithm=None) -> dict` that validates inputs, caps work
with `max_nodes`, restricts the algorithm to a safe allow-list, and **never
raises** — so it is safe inside an agent loop:

```python
from graphfinder.integrations import agents

router = agents.make_router(edges)            # edges: (u, v[, weight]) with any labels
router("A", "E", algorithm="dijkstra")
# {'found': True, 'path': ['A','B','C','E'], 'cost': 3.0, 'nodes_expanded': ..., 'stop_reason': 'goal'}
```

Wrap it as a LangChain `StructuredTool` (needs `langchain-core`):

```python
tool = agents.as_langchain_tool(edges, name="find_route")
tool.invoke({"source": "A", "target": "E"})
```

The graph is bound at tool-creation time; the agent only chooses `source`,
`target` and an allowed `algorithm`.

## Gymnasium — a GridWorld env with an A\* oracle

A reinforcement-learning environment backed by a graphfinder grid. The agent goes
from `S` to `G`; walls are `#`, digits `1`–`9` are terrain costs. Observations are
the flattened cell index, actions are moves (4, or 8 with `diagonal=True`), and
the reward is the negative cost of the entered cell.

```python
from graphfinder.integrations import gym as gfgym

env = gfgym.GridWorldEnv("S....\n.###.\n....G")
obs, info = env.reset()

# A* oracle — useful for imitation learning, reward shaping or scoring an agent:
action = gfgym.optimal_action(env)   # the move A* would make from here
path = gfgym.optimal_path(env)       # the whole optimal path of cells
```

## Graphviz — export graphs and paths

Render an edge-list graph (and a found path) to DOT/SVG. `to_dot` builds the DOT
text with **no dependency**; `source` wraps it in a `graphviz.Source` you can
render or display in a notebook (needs the `graphviz` package + `dot` binary).

```python
from graphfinder.integrations import graphviz as gfgv
from graphfinder.integrations import pandas as gfpd

edges = [("A", "B", 1), ("B", "C", 1), ("A", "C", 5)]
r = gfpd.search(__import__("pandas").DataFrame(edges, columns=["source", "target", "weight"]),
                "A", "C")

print(gfgv.to_dot(edges, r))         # DOT string, path highlighted
gfgv.source(edges, r).render("route", format="svg")   # writes route.svg
```

Path nodes are gold, the start green, the goal red, and path edges are drawn
thick.

## More integrations

Have an idea (igraph, a different framework, …)? Open an issue.
