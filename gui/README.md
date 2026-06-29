# graphfinder desktop GUI

A modern desktop playground for graphfinder, built with **Tauri 2** (Rust) and
**Svelte 5**. The Rust backend depends directly on `graphfinder-core` — no Python
in the loop — and the UI animates the search using the per-step `trace` and the
`tree` the core records.

![icon](src-tauri/icons/128x128.png)

## What you can do

- **Choose any algorithm** — BFS, DFS, UCS/Dijkstra, Greedy, A\*, Weighted A\*,
  IDDFS, DLS, IDA\*, Beam, Bidirectional — and any grid **heuristic** (Manhattan,
  Euclidean, Octile, Zero), with 8-connectivity and the Weighted-A\* `w` knob.
- **Configure the problem**: paint walls, set the start/goal, brush in weighted
  terrain (costs 1–9), resize the grid, generate a reproducible **random maze**
  (seed + density), or load the built-in samples.
- **Watch it run**: animate the expansion order with a speed control and a
  scrubber, overlay the **search tree**, and read the live metrics (cost,
  expanded/generated, peak frontier, stop reason).
- **Compare all** algorithms on the current problem in one click.

## Run it (development)

Prerequisites: a **Rust** toolchain, **Node ≥ 18**, and the Tauri OS deps
(macOS: Xcode CLT; Linux: `webkit2gtk`; Windows: WebView2 — see
<https://tauri.app/start/prerequisites/>).

```bash
cd gui
npm install
npm run tauri dev      # launches the desktop app with hot reload
```

## Build a distributable

```bash
cd gui
npm run tauri build    # bundles a native app/installer into src-tauri/target/release/bundle/
```

On macOS, a full bundle wants an `.icns`; regenerate the whole icon set from a
single source image with:

```bash
npm run tauri icon path/to/logo.png
```

## How it fits together

```
gui/
  src/                 Svelte 5 + TypeScript frontend
    App.svelte         grid editor, controls, canvas animation, metrics, compare
    lib/api.ts         typed wrappers over the Tauri commands
  src-tauri/
    src/main.rs        Tauri commands → graphfinder-core (run_grid, compare_grid,
                       random_maze_map, sample_maze)
    tauri.conf.json    window + bundle config
```

The backend is an isolated Cargo workspace (it is `exclude`d from the library
workspace) so its GUI dependency tree never touches `cargo test`/CI for the
library crates.
