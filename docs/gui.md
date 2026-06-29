# Desktop app (Tauri)

graphfinder ships an optional **desktop GUI** — a modern playground for editing
problems, picking algorithms and *watching* the search run. It is built with
**Tauri 2** (a Rust-backed native shell) and a **Svelte 5** frontend; the backend
calls `graphfinder-core` directly, so there is no Python in the loop.

It lives in [`gui/`](https://github.com/graphfinder/graphfinder.github.io/tree/main/gui)
in the repository.

## What it does

- **Pick any algorithm** — BFS, DFS, UCS/Dijkstra, Greedy, A\*, Weighted A\*,
  IDDFS, DLS, IDA\*, Beam, Bidirectional — with any grid heuristic (Manhattan,
  Euclidean, Octile, Zero), an 8-connectivity toggle and the Weighted-A\* weight.
- **Build the problem** — paint walls, place the start/goal, brush in weighted
  terrain (costs 1–9), resize the board, generate a reproducible random maze
  (seed + density) or load the built-in samples.
- **Animate the search** — replay the expansion order with a speed slider and a
  scrubber, optionally overlay the **search tree**, and read live metrics (cost,
  nodes expanded/generated, peak frontier, stop reason).
- **Compare all** algorithms on the current problem in a single click.

Everything you see is produced by the same core the Python package uses: the
animation is the `trace` (expansion order) and the overlay is the `tree`
(best-parent links) described in [Visualization](visualization.md).

## Run it

Prerequisites: a **Rust** toolchain, **Node ≥ 18**, and the
[Tauri OS prerequisites](https://tauri.app/start/prerequisites/) (macOS: Xcode
Command Line Tools; Linux: `webkit2gtk`; Windows: WebView2).

```bash
cd gui
npm install
npm run tauri dev          # develop with hot reload
npm run tauri build        # bundle a native app/installer
```

## Architecture

```
gui/src/            Svelte 5 + TypeScript UI (App.svelte, lib/api.ts)
gui/src-tauri/      Tauri (Rust): commands → graphfinder-core
```

The four Tauri commands (`run_grid`, `compare_grid`, `random_maze_map`,
`sample_maze`) are a thin bridge: the UI sends an ASCII problem + options, the
backend runs the core and returns the path, metrics, trace and tree as JSON. The
GUI backend is an **isolated Cargo workspace**, so its dependency tree never
affects the library crates' tests or CI.

!!! note "Why Tauri (and not only the web)?"
    The same Svelte UI could run in the browser by compiling the core to WebAssembly.
    The desktop build was chosen first for a native, installable app with full-speed
    search on large instances; a WASM "try it in the browser" page is a natural
    follow-up that would reuse this frontend.
