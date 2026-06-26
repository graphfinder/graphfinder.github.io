//! 2-D grid world — the canonical pathfinding teaching domain.
//!
//! A [`GridGraph`] is a rectangular lattice of [`Cell`]s. Each cell is either a
//! wall or free, and every free cell carries a **terrain cost** (default `1.0`).
//! Moving *into* a cell costs that cell's terrain (× `√2` for a diagonal step),
//! so on a weighted grid Dijkstra/A\* genuinely differ from BFS. This is the
//! domain behind the iconic "watch A* explore a maze" animation.

use crate::traits::Graph;

/// A grid cell at `(row, col)`. `row` grows downward, `col` rightward, matching
/// how ASCII maps read. Coordinates are `i32` so heuristics can subtract freely.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Cell {
    pub row: i32,
    pub col: i32,
}

impl Cell {
    pub fn new(row: i32, col: i32) -> Self {
        Self { row, col }
    }
}

/// A 2-D grid with per-cell terrain cost. Entering a cell costs its terrain
/// (default `1.0`); diagonal steps (when enabled) cost `terrain × √2`.
#[derive(Clone, Debug)]
pub struct GridGraph {
    width: i32,
    height: i32,
    blocked: Vec<bool>,
    cost: Vec<f64>,
    diagonal: bool,
}

impl GridGraph {
    /// An empty `width × height` grid: no walls, every cell terrain `1.0`,
    /// 4-connected.
    pub fn new(width: i32, height: i32) -> Self {
        assert!(width > 0 && height > 0, "grid must be non-empty");
        let n = (width * height) as usize;
        Self {
            width,
            height,
            blocked: vec![false; n],
            cost: vec![1.0; n],
            diagonal: false,
        }
    }

    /// Build a grid from a rectangular matrix of terrain costs. A cell whose
    /// cost is non-positive or non-finite becomes a wall; otherwise it is a free
    /// cell with that movement cost. Rows may be ragged (short rows are padded
    /// with walls).
    pub fn from_costs(rows: &[Vec<f64>]) -> Self {
        let height = rows.len() as i32;
        let width = rows.iter().map(|r| r.len()).max().unwrap_or(0) as i32;
        assert!(width > 0 && height > 0, "cost grid must be non-empty");
        let mut grid = Self::new(width, height);
        for (r, row) in rows.iter().enumerate() {
            for c in 0..(width as usize) {
                let cell = Cell::new(r as i32, c as i32);
                match row.get(c) {
                    Some(&w) if w.is_finite() && w > 0.0 => grid.set_cost(cell, w),
                    _ => grid.block(cell),
                }
            }
        }
        grid
    }

    /// Enable 8-connectivity (diagonal moves, cost `√2`). Builder style.
    pub fn with_diagonal(mut self, on: bool) -> Self {
        self.diagonal = on;
        self
    }

    pub fn width(&self) -> i32 {
        self.width
    }
    pub fn height(&self) -> i32 {
        self.height
    }
    pub fn diagonal(&self) -> bool {
        self.diagonal
    }

    fn index(&self, c: Cell) -> usize {
        (c.row * self.width + c.col) as usize
    }

    /// Whether `c` lies inside the grid.
    pub fn in_bounds(&self, c: Cell) -> bool {
        c.row >= 0 && c.col >= 0 && c.row < self.height && c.col < self.width
    }

    /// Whether `c` is a wall (out-of-bounds counts as blocked).
    pub fn is_blocked(&self, c: Cell) -> bool {
        !self.in_bounds(c) || self.blocked[self.index(c)]
    }

    /// Mark `c` as a wall.
    pub fn block(&mut self, c: Cell) {
        if self.in_bounds(c) {
            let i = self.index(c);
            self.blocked[i] = true;
        }
    }

    /// Mark `c` as free.
    pub fn unblock(&mut self, c: Cell) {
        if self.in_bounds(c) {
            let i = self.index(c);
            self.blocked[i] = false;
        }
    }

    /// Terrain cost of entering `c` (the default is `1.0`). Out-of-bounds cells
    /// report `1.0`; it is the caller's job to check `is_blocked`.
    pub fn cost_at(&self, c: Cell) -> f64 {
        if self.in_bounds(c) {
            self.cost[self.index(c)]
        } else {
            1.0
        }
    }

    /// Set the terrain cost of entering `c` (must be positive). Higher cost makes
    /// the cell more expensive to pass through; use [`GridGraph::block`] for an
    /// impassable wall.
    pub fn set_cost(&mut self, c: Cell, cost: f64) {
        assert!(
            cost > 0.0 && cost.is_finite(),
            "terrain cost must be positive"
        );
        if self.in_bounds(c) {
            let i = self.index(c);
            self.cost[i] = cost;
        }
    }

    /// Whether any cell has a non-default terrain cost.
    pub fn is_weighted(&self) -> bool {
        self.cost.iter().any(|&w| w != 1.0)
    }

    /// All free cells (handy for visualization / iteration).
    pub fn free_cells(&self) -> impl Iterator<Item = Cell> + '_ {
        (0..self.height).flat_map(move |r| {
            (0..self.width).filter_map(move |col| {
                let c = Cell::new(r, col);
                (!self.is_blocked(c)).then_some(c)
            })
        })
    }

    /// Parse an ASCII map. `#` = wall, `S` = start, `G` = goal, a digit `1`–`9`
    /// = a free cell with that **terrain cost**, anything else (e.g. `.`) = a
    /// free cell of cost `1.0`. Returns the grid plus the start and goal cells.
    ///
    /// ```
    /// use graphfinder_core::{Cell, GridGraph};
    /// // A row of expensive terrain ('9') between start and goal.
    /// let (grid, start, goal) = GridGraph::from_ascii("S99G");
    /// assert_eq!(grid.cost_at(Cell::new(0, 1)), 9.0);
    /// assert_eq!((goal.row, goal.col), (0, 3));
    /// ```
    pub fn from_ascii(map: &str) -> (Self, Cell, Cell) {
        let lines: Vec<&str> = map.lines().filter(|l| !l.is_empty()).collect();
        let height = lines.len() as i32;
        let width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0) as i32;
        let mut grid = Self::new(width, height);
        let mut start = None;
        let mut goal = None;
        for (r, line) in lines.iter().enumerate() {
            for (col, ch) in line.chars().enumerate() {
                let cell = Cell::new(r as i32, col as i32);
                match ch {
                    '#' => grid.block(cell),
                    'S' => start = Some(cell),
                    'G' => goal = Some(cell),
                    '1'..='9' => grid.set_cost(cell, ch.to_digit(10).unwrap() as f64),
                    _ => {}
                }
            }
        }
        let start = start.expect("ASCII map must contain a start cell 'S'");
        let goal = goal.expect("ASCII map must contain a goal cell 'G'");
        (grid, start, goal)
    }
}

impl Graph for GridGraph {
    type Node = Cell;

    fn neighbors(&self, c: &Cell) -> Vec<(Cell, f64)> {
        let mut out = Vec::with_capacity(if self.diagonal { 8 } else { 4 });
        // Entering a cell costs its terrain; a diagonal step costs terrain × √2.
        const ORTHO: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
        for (dr, dc) in ORTHO {
            let n = Cell::new(c.row + dr, c.col + dc);
            if !self.is_blocked(n) {
                out.push((n, self.cost_at(n)));
            }
        }
        if self.diagonal {
            const DIAG: [(i32, i32); 4] = [(-1, -1), (-1, 1), (1, -1), (1, 1)];
            for (dr, dc) in DIAG {
                let n = Cell::new(c.row + dr, c.col + dc);
                // Disallow corner-cutting: both orthogonal cells must be free.
                let side_a = Cell::new(c.row + dr, c.col);
                let side_b = Cell::new(c.row, c.col + dc);
                if !self.is_blocked(n) && !self.is_blocked(side_a) && !self.is_blocked(side_b) {
                    out.push((n, self.cost_at(n) * std::f64::consts::SQRT_2));
                }
            }
        }
        out
    }
}
