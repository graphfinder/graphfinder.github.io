//! 2-D grid world — the canonical pathfinding teaching domain.
//!
//! A [`GridGraph`] is a rectangular lattice of [`Cell`]s. Each cell is either
//! free or blocked (a wall); a node's neighbours are its free orthogonal (and
//! optionally diagonal) cells. This is the domain behind the iconic
//! "watch A* explore a maze" animation.

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

/// A 2-D occupancy grid. Orthogonal moves cost `1.0`; diagonal moves (when
/// enabled) cost `√2`.
#[derive(Clone, Debug)]
pub struct GridGraph {
    width: i32,
    height: i32,
    blocked: Vec<bool>,
    diagonal: bool,
}

impl GridGraph {
    /// An empty `width × height` grid with no walls, 4-connected.
    pub fn new(width: i32, height: i32) -> Self {
        assert!(width > 0 && height > 0, "grid must be non-empty");
        Self {
            width,
            height,
            blocked: vec![false; (width * height) as usize],
            diagonal: false,
        }
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

    /// All free cells (handy for visualization / iteration).
    pub fn free_cells(&self) -> impl Iterator<Item = Cell> + '_ {
        (0..self.height).flat_map(move |r| {
            (0..self.width).filter_map(move |col| {
                let c = Cell::new(r, col);
                (!self.is_blocked(c)).then_some(c)
            })
        })
    }

    /// Parse an ASCII map. `#` = wall, `S` = start, `G` = goal, anything else =
    /// free. Returns the grid plus the start and goal cells.
    ///
    /// ```
    /// use graphfinder_core::GridGraph;
    /// let (grid, start, goal) = GridGraph::from_ascii("S..\n.#.\n..G");
    /// assert_eq!((start.row, start.col), (0, 0));
    /// assert_eq!((goal.row, goal.col), (2, 2));
    /// assert!(grid.is_blocked(graphfinder_core::Cell::new(1, 1)));
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
        const ORTHO: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
        for (dr, dc) in ORTHO {
            let n = Cell::new(c.row + dr, c.col + dc);
            if !self.is_blocked(n) {
                out.push((n, 1.0));
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
                    out.push((n, std::f64::consts::SQRT_2));
                }
            }
        }
        out
    }
}
