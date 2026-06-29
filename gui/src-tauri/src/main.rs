// Prevent a second console window on Windows in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! Tauri backend for the graphfinder desktop GUI. It is a thin bridge: the UI
//! describes a problem (an ASCII grid + an algorithm + options), the backend
//! runs `graphfinder-core` and returns the path, metrics, the per-step trace and
//! the search tree as plain JSON for the Svelte frontend to animate.

use serde::{Deserialize, Serialize};

use graphfinder_core::domains::{random_maze, SAMPLE_OPEN, SAMPLE_WALL};
use graphfinder_core::{
    beam_search, bidirectional, dls, ida_star, iddfs, search_with, Algorithm, Cell, Euclidean,
    GridGraph, Heuristic, Manhattan, Octile, SearchResult, StopReason, Zero,
};

/// A problem + algorithm to run, as sent by the UI (camelCase keys).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GridConfig {
    /// ASCII map: `#` wall, `S` start, `G` goal, `1`–`9` terrain cost, `.` free.
    map: String,
    /// `bfs`, `dfs`, `ucs`, `greedy`, `astar`, `weighted_astar`, `iddfs`, `dls`,
    /// `ida_star`, `beam`, `bidirectional`.
    algorithm: String,
    /// `zero`, `manhattan`, `euclidean`, `octile`.
    heuristic: String,
    /// 8-connectivity (use with `octile`).
    #[serde(default)]
    diagonal: bool,
    /// `w` for weighted A*.
    #[serde(default = "default_weight")]
    weight: f64,
    /// Beam width for `beam` (default unbounded).
    #[serde(default)]
    beam_width: Option<usize>,
    /// Depth limit for `dls`/`iddfs`.
    #[serde(default)]
    depth_limit: Option<usize>,
    /// Optional expansion budget.
    #[serde(default)]
    max_nodes: Option<usize>,
}

fn default_weight() -> f64 {
    2.0
}

#[derive(Serialize)]
struct Step {
    r: i32,
    c: i32,
    g: f64,
    frontier: usize,
}

/// The full outcome handed back to the UI. `cost` is `null` (JSON) when no path.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ResultDto {
    found: bool,
    cost: f64,
    nodes_expanded: usize,
    nodes_generated: usize,
    max_frontier_size: usize,
    stop_reason: String,
    path: Vec<[i32; 2]>,
    trace: Vec<Step>,
    tree: Vec<[[i32; 2]; 2]>,
}

/// One row of the algorithm comparison table.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CompareRow {
    algorithm: String,
    found: bool,
    cost: f64,
    nodes_expanded: usize,
    max_frontier_size: usize,
    path_len: usize,
}

fn stop_str(reason: StopReason) -> String {
    match reason {
        StopReason::GoalReached => "goal",
        StopReason::FrontierExhausted => "exhausted",
        StopReason::NodeLimit => "node_limit",
    }
    .to_string()
}

/// Run the configured algorithm with a concrete heuristic type. Always records,
/// so the UI gets the trace + tree it needs to animate.
fn dispatch<H: Heuristic<Cell> + Clone>(
    grid: &GridGraph,
    start: Cell,
    goal: Cell,
    cfg: &GridConfig,
    h: H,
) -> SearchResult<Cell> {
    let max_nodes = cfg.max_nodes;
    match cfg.algorithm.as_str() {
        "bfs" => search_with(grid, start, goal, Algorithm::bfs(), &h, true, max_nodes),
        "dfs" => search_with(grid, start, goal, Algorithm::dfs(), &h, true, max_nodes),
        "ucs" | "dijkstra" => search_with(grid, start, goal, Algorithm::ucs(), &h, true, max_nodes),
        "greedy" => search_with(grid, start, goal, Algorithm::greedy(), &h, true, max_nodes),
        "weighted_astar" | "wastar" => search_with(
            grid,
            start,
            goal,
            Algorithm::weighted_astar(cfg.weight),
            &h,
            true,
            max_nodes,
        ),
        "iddfs" => iddfs(grid, start, goal, cfg.depth_limit.unwrap_or(1000), true),
        "dls" => dls(grid, start, goal, cfg.depth_limit.unwrap_or(64), true),
        "ida_star" | "idastar" => ida_star(grid, start, goal, h, true),
        "beam" => beam_search(
            grid,
            start,
            goal,
            h,
            cfg.beam_width.unwrap_or(usize::MAX),
            true,
        ),
        "bidirectional" | "bidir" => bidirectional(grid, start, goal, true),
        // default: A*
        _ => search_with(grid, start, goal, Algorithm::astar(), &h, true, max_nodes),
    }
}

/// Pick the heuristic type by name, then dispatch on the algorithm.
fn run(grid: &GridGraph, start: Cell, goal: Cell, cfg: &GridConfig) -> SearchResult<Cell> {
    match cfg.heuristic.as_str() {
        "manhattan" => dispatch(grid, start, goal, cfg, Manhattan),
        "euclidean" => dispatch(grid, start, goal, cfg, Euclidean),
        "octile" => dispatch(grid, start, goal, cfg, Octile),
        _ => dispatch(grid, start, goal, cfg, Zero),
    }
}

fn build_grid(cfg: &GridConfig) -> Result<(GridGraph, Cell, Cell), String> {
    if !cfg.map.contains('S') || !cfg.map.contains('G') {
        return Err("the map must contain a start 'S' and a goal 'G'".into());
    }
    let (mut grid, start, goal) = GridGraph::from_ascii(&cfg.map);
    if cfg.diagonal {
        grid = grid.with_diagonal(true);
    }
    Ok((grid, start, goal))
}

#[tauri::command]
fn run_grid(config: GridConfig) -> Result<ResultDto, String> {
    let (grid, start, goal) = build_grid(&config)?;
    let r = run(&grid, start, goal, &config);
    Ok(ResultDto {
        found: r.found(),
        cost: r.cost,
        nodes_expanded: r.nodes_expanded,
        nodes_generated: r.nodes_generated,
        max_frontier_size: r.max_frontier_size,
        stop_reason: stop_str(r.stop_reason),
        path: r
            .path
            .as_ref()
            .map(|p| p.iter().map(|c| [c.row, c.col]).collect())
            .unwrap_or_default(),
        trace: r
            .trace
            .iter()
            .map(|s| Step {
                r: s.expanded.row,
                c: s.expanded.col,
                g: s.g,
                frontier: s.frontier_size,
            })
            .collect(),
        tree: r
            .tree
            .iter()
            .map(|(p, c)| [[p.row, p.col], [c.row, c.col]])
            .collect(),
    })
}

#[tauri::command]
fn compare_grid(config: GridConfig) -> Result<Vec<CompareRow>, String> {
    let (grid, start, goal) = build_grid(&config)?;
    let algorithms = ["bfs", "dfs", "ucs", "greedy", "astar", "weighted_astar"];
    let rows = algorithms
        .iter()
        .map(|name| {
            let mut cfg = config.clone();
            cfg.algorithm = (*name).to_string();
            let r = run(&grid, start, goal, &cfg);
            CompareRow {
                algorithm: (*name).to_string(),
                found: r.found(),
                cost: r.cost,
                nodes_expanded: r.nodes_expanded,
                max_frontier_size: r.max_frontier_size,
                path_len: r.path.as_ref().map(|p| p.len()).unwrap_or(0),
            }
        })
        .collect();
    Ok(rows)
}

/// Render a `GridGraph`-backed maze region as an ASCII map with `S`/`G`.
#[tauri::command]
fn random_maze_map(width: i32, height: i32, density: f64, seed: u64) -> String {
    let m = random_maze(width, height, density, seed);
    let mut out = String::new();
    for row in 0..height {
        for col in 0..width {
            let cell = Cell::new(row, col);
            let ch = if cell == m.start {
                'S'
            } else if cell == m.goal {
                'G'
            } else if m.grid.is_blocked(cell) {
                '#'
            } else {
                '.'
            };
            out.push(ch);
        }
        if row + 1 < height {
            out.push('\n');
        }
    }
    out
}

#[tauri::command]
fn sample_maze(name: String) -> Result<String, String> {
    match name.as_str() {
        "open" => Ok(SAMPLE_OPEN.to_string()),
        "wall" => Ok(SAMPLE_WALL.to_string()),
        other => Err(format!(
            "unknown sample maze: '{other}' (use 'open' or 'wall')"
        )),
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            run_grid,
            compare_grid,
            random_maze_map,
            sample_maze
        ])
        .run(tauri::generate_context!())
        .expect("error while running the graphfinder GUI");
}
