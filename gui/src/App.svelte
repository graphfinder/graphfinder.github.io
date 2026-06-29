<script lang="ts">
  import {
    runGrid,
    compareGrid,
    randomMaze,
    sampleMaze,
    type GridConfig,
    type ResultDto,
    type CompareRow,
  } from "./lib/api";

  // --- static option tables ------------------------------------------------
  const ALGOS: [string, string][] = [
    ["bfs", "BFS"],
    ["dfs", "DFS"],
    ["ucs", "UCS / Dijkstra"],
    ["greedy", "Greedy"],
    ["astar", "A*"],
    ["weighted_astar", "Weighted A*"],
    ["iddfs", "IDDFS"],
    ["dls", "DLS"],
    ["ida_star", "IDA*"],
    ["beam", "Beam"],
    ["bidirectional", "Bidirectional"],
  ];
  const HEURS: [string, string][] = [
    ["manhattan", "Manhattan"],
    ["euclidean", "Euclidean"],
    ["octile", "Octile"],
    ["zero", "Zero (uninformed)"],
  ];
  const BRUSHES: [string, string][] = [
    ["wall", "Wall"],
    ["erase", "Erase"],
    ["start", "Start"],
    ["goal", "Goal"],
    ["terrain", "Terrain"],
  ];

  // colours (match graphfinder.viz)
  const C = {
    wall: "#37474f",
    free: "#f3f4f6",
    visited: "rgba(144,202,249,0.60)",
    path: "#fbc02d",
    start: "#43a047",
    goal: "#e53935",
    grid: "#d4d9e6",
    tree: "rgba(38,50,56,0.35)",
  };

  const DEFAULT_MAP = [
    "S.........",
    ".####.###.",
    ".#..2.#...",
    ".#.##2#.#.",
    "...#..2#..",
    ".#.###.##.",
    ".#...3...#",
    ".###.###.#",
    "...#...#.G",
  ].join("\n");

  // --- reactive state ------------------------------------------------------
  let cells = $state<string[][]>(parseMap(DEFAULT_MAP));
  let algorithm = $state("astar");
  let heuristic = $state("manhattan");
  let diagonal = $state(false);
  let weight = $state(2);
  let brush = $state("wall");
  let terrainValue = $state(5);
  let speed = $state(12);

  let result = $state<ResultDto | null>(null);
  let frame = $state(0);
  let playing = $state(false);
  let busy = $state(false);
  let error = $state<string | null>(null);
  let showTree = $state(false);
  let compareRows = $state<CompareRow[] | null>(null);

  // size controls
  let newW = $state(10);
  let newH = $state(9);
  let density = $state(0.28);
  let seed = $state(7);

  let canvas: HTMLCanvasElement | undefined = $state();
  let painting = false;

  // --- derived -------------------------------------------------------------
  let rows = $derived(cells.length);
  let cols = $derived(cells[0]?.length ?? 0);
  let cellSize = $derived(Math.max(8, Math.min(46, Math.floor(680 / Math.max(rows, cols, 1)))));
  let traceLen = $derived(result?.trace.length ?? 0);

  function config(): GridConfig {
    return {
      map: toMap(cells),
      algorithm,
      heuristic,
      diagonal,
      weight,
      beamWidth: algorithm === "beam" ? 100 : null,
      depthLimit: algorithm === "dls" || algorithm === "iddfs" ? 200 : null,
      maxNodes: null,
    };
  }

  // --- map <-> cells helpers ----------------------------------------------
  function parseMap(map: string): string[][] {
    const lines = map.split("\n").filter((l) => l.length);
    const w = Math.max(...lines.map((l) => l.length), 1);
    return lines.map((l) => {
      const row = l.split("");
      while (row.length < w) row.push(".");
      return row;
    });
  }
  function toMap(c: string[][]): string {
    return c.map((row) => row.join("")).join("\n");
  }

  function invalidate() {
    result = null;
    frame = 0;
    playing = false;
    compareRows = null;
  }

  function findChar(ch: string): [number, number] | null {
    for (let r = 0; r < cells.length; r++)
      for (let c = 0; c < cells[r].length; c++) if (cells[r][c] === ch) return [r, c];
    return null;
  }

  // --- editing -------------------------------------------------------------
  function applyAt(ev: PointerEvent, isDown: boolean) {
    if (!canvas) return;
    const rect = canvas.getBoundingClientRect();
    const c = Math.floor((ev.clientX - rect.left) / cellSize);
    const r = Math.floor((ev.clientY - rect.top) / cellSize);
    if (r < 0 || c < 0 || r >= rows || c >= cols) return;
    const cur = cells[r][c];

    if (brush === "start" || brush === "goal") {
      if (!isDown) return; // place once, not on drag
      const mark = brush === "start" ? "S" : "G";
      const prev = findChar(mark);
      if (prev) cells[prev[0]][prev[1]] = ".";
      if (cur === "S" || cur === "G") return; // don't stack endpoints
      cells[r][c] = mark;
    } else {
      if (cur === "S" || cur === "G") return; // protect endpoints
      if (brush === "wall") cells[r][c] = "#";
      else if (brush === "erase") cells[r][c] = ".";
      else if (brush === "terrain") cells[r][c] = String(terrainValue);
    }
    invalidate();
    draw();
  }

  // --- run / compare -------------------------------------------------------
  async function run() {
    busy = true;
    error = null;
    compareRows = null;
    try {
      result = await runGrid(config());
      frame = 0;
      playing = true;
    } catch (e) {
      error = String(e);
      result = null;
    } finally {
      busy = false;
    }
  }

  async function doCompare() {
    busy = true;
    error = null;
    try {
      compareRows = await compareGrid(config());
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  async function loadSample(name: string) {
    try {
      cells = parseMap(await sampleMaze(name));
      invalidate();
    } catch (e) {
      error = String(e);
    }
  }
  async function loadRandom() {
    try {
      cells = parseMap(await randomMaze(newW, newH, density, seed));
      seed += 1;
      invalidate();
    } catch (e) {
      error = String(e);
    }
  }
  function blankGrid() {
    const g: string[][] = Array.from({ length: newH }, () => Array.from({ length: newW }, () => "."));
    g[0][0] = "S";
    g[newH - 1][newW - 1] = "G";
    cells = g;
    invalidate();
  }
  function clearWalls() {
    cells = cells.map((row) => row.map((ch) => (ch === "#" || /[1-9]/.test(ch) ? "." : ch)));
    invalidate();
  }

  // --- playback ------------------------------------------------------------
  function togglePlay() {
    if (!result) return;
    if (frame >= traceLen) frame = 0;
    playing = !playing;
  }
  function step() {
    if (!result) return;
    playing = false;
    frame = Math.min(traceLen, frame + 1);
  }
  function resetAnim() {
    frame = 0;
    playing = false;
  }

  $effect(() => {
    if (!playing) return;
    const perTick = Math.max(1, Math.round(speed));
    const id = setInterval(() => {
      frame = frame + perTick;
      if (frame >= traceLen) {
        frame = traceLen;
        playing = false;
      }
    }, 33);
    return () => clearInterval(id);
  });

  // stop painting if the pointer is released anywhere
  $effect(() => {
    const up = () => (painting = false);
    window.addEventListener("pointerup", up);
    return () => window.removeEventListener("pointerup", up);
  });

  // --- drawing -------------------------------------------------------------
  function terrainColor(d: number): string {
    const t = (d - 1) / 8;
    const a = [243, 244, 246];
    const b = [161, 86, 30];
    const mix = a.map((x, i) => Math.round(x + (b[i] - x) * t));
    return `rgb(${mix[0]},${mix[1]},${mix[2]})`;
  }

  function draw() {
    const cv = canvas;
    if (!cv) return;
    const ctx = cv.getContext("2d");
    if (!ctx) return;
    const cs = cellSize;
    cv.width = cols * cs;
    cv.height = rows * cs;

    // base cells
    for (let r = 0; r < rows; r++) {
      for (let c = 0; c < cols; c++) {
        const ch = cells[r][c];
        let fill = C.free;
        if (ch === "#") fill = C.wall;
        else if (/[1-9]/.test(ch)) fill = terrainColor(Number(ch));
        ctx.fillStyle = fill;
        ctx.fillRect(c * cs, r * cs, cs, cs);
      }
    }

    // visited overlay (up to the current frame)
    if (result) {
      const upto = Math.min(frame, result.trace.length);
      ctx.fillStyle = C.visited;
      for (let i = 0; i < upto; i++) {
        const s = result.trace[i];
        const ch = cells[s.r]?.[s.c];
        if (ch !== "S" && ch !== "G") ctx.fillRect(s.c * cs, s.r * cs, cs, cs);
      }

      // search-tree overlay (optional)
      if (showTree) {
        ctx.strokeStyle = C.tree;
        ctx.lineWidth = Math.max(1, cs * 0.06);
        ctx.beginPath();
        for (const [[pr, pc], [cr, cc]] of result.tree) {
          ctx.moveTo(pc * cs + cs / 2, pr * cs + cs / 2);
          ctx.lineTo(cc * cs + cs / 2, cr * cs + cs / 2);
        }
        ctx.stroke();
      }

      // final path once the trace has fully played
      if (frame >= result.trace.length && result.found) {
        ctx.fillStyle = C.path;
        for (const [r, c] of result.path) {
          const ch = cells[r]?.[c];
          if (ch !== "S" && ch !== "G") ctx.fillRect(c * cs, r * cs, cs, cs);
        }
      }
    }

    // start / goal markers
    drawEndpoint(ctx, findChar("S"), C.start, cs);
    drawEndpoint(ctx, findChar("G"), C.goal, cs);

    // grid lines
    ctx.strokeStyle = C.grid;
    ctx.lineWidth = 1;
    ctx.beginPath();
    for (let r = 0; r <= rows; r++) {
      ctx.moveTo(0, r * cs);
      ctx.lineTo(cols * cs, r * cs);
    }
    for (let c = 0; c <= cols; c++) {
      ctx.moveTo(c * cs, 0);
      ctx.lineTo(c * cs, rows * cs);
    }
    ctx.stroke();
  }

  function drawEndpoint(
    ctx: CanvasRenderingContext2D,
    pos: [number, number] | null,
    color: string,
    cs: number,
  ) {
    if (!pos) return;
    const [r, c] = pos;
    ctx.fillStyle = color;
    ctx.fillRect(c * cs, r * cs, cs, cs);
  }

  // redraw whenever inputs that affect the picture change
  $effect(() => {
    // touch the reactive deps so the effect re-runs
    void cells;
    void result;
    void frame;
    void showTree;
    void cellSize;
    draw();
  });

  function fmt(x: number | null): string {
    return x === null || x === undefined ? "∞" : Number.isInteger(x) ? String(x) : x.toFixed(2);
  }
  let maxExpanded = $derived(
    compareRows ? Math.max(1, ...compareRows.map((r) => r.nodesExpanded)) : 1,
  );
</script>

<div class="app">
  <header>
    <div class="brand">
      <span class="logo"></span>
      <div>
        <h1>graphfinder</h1>
        <p class="tag">traversal &amp; pathfinding playground</p>
      </div>
    </div>
    <div class="actions">
      <button class="primary" onclick={run} disabled={busy}>▶ Run</button>
      <button onclick={togglePlay} disabled={!result}>{playing ? "⏸ Pause" : "⏵ Play"}</button>
      <button onclick={step} disabled={!result}>⏭ Step</button>
      <button onclick={resetAnim} disabled={!result}>↺ Reset</button>
      <button onclick={doCompare} disabled={busy}>⊞ Compare all</button>
    </div>
  </header>

  <main>
    <aside class="sidebar">
      <section>
        <h2>Algorithm</h2>
        <label>
          Algorithm
          <select bind:value={algorithm} onchange={invalidate}>
            {#each ALGOS as [v, l]}<option value={v}>{l}</option>{/each}
          </select>
        </label>
        <label>
          Heuristic
          <select bind:value={heuristic} onchange={invalidate}>
            {#each HEURS as [v, l]}<option value={v}>{l}</option>{/each}
          </select>
        </label>
        <label class="row">
          <input type="checkbox" bind:checked={diagonal} onchange={invalidate} />
          8-connected (diagonal)
        </label>
        {#if algorithm === "weighted_astar"}
          <label>
            Weight w = {weight}
            <input type="range" min="1" max="5" step="0.5" bind:value={weight} onchange={invalidate} />
          </label>
        {/if}
      </section>

      <section>
        <h2>Edit the maze</h2>
        <div class="brushes">
          {#each BRUSHES as [v, l]}
            <button class:active={brush === v} onclick={() => (brush = v)}>{l}</button>
          {/each}
        </div>
        {#if brush === "terrain"}
          <label>
            Terrain cost = {terrainValue}
            <input type="range" min="2" max="9" step="1" bind:value={terrainValue} />
          </label>
        {/if}
        <div class="grid2">
          <label>W <input type="number" min="2" max="60" bind:value={newW} /></label>
          <label>H <input type="number" min="2" max="60" bind:value={newH} /></label>
        </div>
        <div class="btn-row">
          <button onclick={blankGrid}>New blank</button>
          <button onclick={clearWalls}>Clear walls</button>
        </div>
        <div class="grid2">
          <label>Density <input type="number" min="0" max="0.6" step="0.02" bind:value={density} /></label>
          <label>Seed <input type="number" min="0" bind:value={seed} /></label>
        </div>
        <div class="btn-row">
          <button onclick={loadRandom}>🎲 Random</button>
          <button onclick={() => loadSample("open")}>Sample: open</button>
          <button onclick={() => loadSample("wall")}>Sample: wall</button>
        </div>
      </section>

      <section>
        <h2>Animation</h2>
        <label>
          Speed (cells/tick) = {speed}
          <input type="range" min="1" max="60" step="1" bind:value={speed} />
        </label>
        <label class="row">
          <input type="checkbox" bind:checked={showTree} />
          Show search tree
        </label>
      </section>
    </aside>

    <section class="stage">
      {#if error}<div class="error">{error}</div>{/if}
      <div class="canvas-wrap">
        <canvas
          bind:this={canvas}
          onpointerdown={(e) => {
            painting = true;
            applyAt(e, true);
          }}
          onpointermove={(e) => {
            if (painting) applyAt(e, false);
          }}
        ></canvas>
      </div>
      <div class="legend">
        <span><i style="background:{C.start}"></i> start</span>
        <span><i style="background:{C.goal}"></i> goal</span>
        <span><i style="background:#90caf9"></i> expanded</span>
        <span><i style="background:{C.path}"></i> path</span>
        <span><i style="background:{C.wall}"></i> wall</span>
        <span><i style="background:#c8853c"></i> terrain</span>
      </div>
      {#if result}
        <input
          class="scrub"
          type="range"
          min="0"
          max={traceLen}
          bind:value={frame}
          oninput={() => (playing = false)}
        />
        <div class="scrub-label">step {Math.min(frame, traceLen)} / {traceLen}</div>
      {/if}
    </section>

    <aside class="results">
      <section>
        <h2>Result</h2>
        {#if result}
          <dl class="metrics">
            <dt>Found</dt><dd>{result.found ? "yes" : "no"}</dd>
            <dt>Cost</dt><dd>{fmt(result.cost)}</dd>
            <dt>Path length</dt><dd>{result.path.length}</dd>
            <dt>Expanded</dt><dd>{result.nodesExpanded}</dd>
            <dt>Generated</dt><dd>{result.nodesGenerated}</dd>
            <dt>Peak frontier</dt><dd>{result.maxFrontierSize}</dd>
            <dt>Stop</dt><dd>{result.stopReason}</dd>
          </dl>
        {:else}
          <p class="muted">Press <b>Run</b> to search and animate.</p>
        {/if}
      </section>

      {#if compareRows}
        <section>
          <h2>Compare</h2>
          <div class="compare">
            {#each compareRows as row}
              <div class="crow">
                <span class="cname">{row.algorithm}</span>
                <div class="bar">
                  <div class="fill" style="width:{(row.nodesExpanded / maxExpanded) * 100}%"></div>
                </div>
                <span class="cval">{row.nodesExpanded} exp · {fmt(row.cost)}</span>
              </div>
            {/each}
          </div>
          <p class="muted small">Bars: nodes expanded (work). Lower is better.</p>
        </section>
      {/if}
    </aside>
  </main>
</div>

<style>
  .app {
    display: flex;
    flex-direction: column;
    height: 100%;
  }
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 18px;
    border-bottom: 1px solid var(--border);
    background: var(--panel);
  }
  .brand {
    display: flex;
    gap: 12px;
    align-items: center;
  }
  .logo {
    width: 34px;
    height: 34px;
    border-radius: 9px;
    background: linear-gradient(135deg, #7986cb, #3f51b5);
    box-shadow: inset 0 0 0 2px rgba(255, 255, 255, 0.12);
  }
  h1 {
    font-size: 18px;
    margin: 0;
    letter-spacing: 0.2px;
  }
  .tag {
    margin: 0;
    color: var(--muted);
    font-size: 12px;
  }
  .actions {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }

  main {
    flex: 1;
    display: grid;
    grid-template-columns: 270px 1fr 280px;
    gap: 0;
    min-height: 0;
  }
  .sidebar,
  .results {
    overflow-y: auto;
    padding: 14px;
    background: var(--panel);
    border-right: 1px solid var(--border);
  }
  .results {
    border-right: none;
    border-left: 1px solid var(--border);
  }
  section {
    margin-bottom: 18px;
  }
  h2 {
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.6px;
    color: var(--muted);
    margin: 0 0 10px;
  }
  label {
    display: block;
    font-size: 13px;
    color: var(--muted);
    margin-bottom: 10px;
  }
  label.row {
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--text);
  }
  select,
  input[type="range"] {
    width: 100%;
    margin-top: 4px;
  }
  .grid2 {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px;
  }
  .grid2 input {
    width: 100%;
  }
  .btn-row {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
    margin-bottom: 10px;
  }
  .btn-row button {
    flex: 1;
    font-size: 12px;
    padding: 6px 8px;
  }
  .brushes {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
    margin-bottom: 10px;
  }
  .brushes button {
    flex: 1;
    font-size: 12px;
    padding: 6px 4px;
  }
  .brushes button.active {
    background: var(--accent);
    border-color: var(--accent-hi);
    color: #fff;
  }

  .stage {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 18px;
    gap: 12px;
    min-width: 0;
  }
  .canvas-wrap {
    background: var(--panel-2);
    padding: 12px;
    border-radius: 14px;
    border: 1px solid var(--border);
    box-shadow: 0 10px 40px rgba(0, 0, 0, 0.35);
  }
  canvas {
    display: block;
    image-rendering: pixelated;
    border-radius: 4px;
    touch-action: none;
    cursor: crosshair;
  }
  .legend {
    display: flex;
    gap: 14px;
    flex-wrap: wrap;
    color: var(--muted);
    font-size: 12px;
  }
  .legend i {
    display: inline-block;
    width: 12px;
    height: 12px;
    border-radius: 3px;
    margin-right: 5px;
    vertical-align: -1px;
  }
  .scrub {
    width: min(680px, 100%);
  }
  .scrub-label {
    color: var(--muted);
    font-size: 12px;
  }

  .metrics {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 6px 12px;
    margin: 0;
    font-size: 13px;
  }
  .metrics dt {
    color: var(--muted);
  }
  .metrics dd {
    margin: 0;
    text-align: right;
    font-variant-numeric: tabular-nums;
  }
  .muted {
    color: var(--muted);
    font-size: 13px;
  }
  .small {
    font-size: 11px;
  }

  .compare {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .crow {
    display: grid;
    grid-template-columns: 78px 1fr;
    align-items: center;
    gap: 6px;
    font-size: 12px;
  }
  .cname {
    color: var(--text);
  }
  .bar {
    grid-column: 2;
    height: 10px;
    background: var(--panel-2);
    border-radius: 6px;
    overflow: hidden;
  }
  .fill {
    height: 100%;
    background: linear-gradient(90deg, #7986cb, #3f51b5);
  }
  .cval {
    grid-column: 2;
    color: var(--muted);
    font-variant-numeric: tabular-nums;
  }

  .error {
    background: #4a1d22;
    border: 1px solid var(--red);
    color: #ffd7d7;
    padding: 8px 12px;
    border-radius: 8px;
    font-size: 13px;
    align-self: stretch;
  }
</style>
