//! Maze instances: a few fixed ASCII maps plus a seeded random generator.

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::graph::{Cell, GridGraph};

/// A ready-to-search instance: the grid plus its start and goal cells.
pub struct Maze {
    pub grid: GridGraph,
    pub start: Cell,
    pub goal: Cell,
}

impl Maze {
    /// Build a maze from an ASCII map (see [`GridGraph::from_ascii`]).
    pub fn from_ascii(map: &str) -> Self {
        let (grid, start, goal) = GridGraph::from_ascii(map);
        Self { grid, start, goal }
    }
}

/// A small open room with a single blocking wall — the optimal path costs 8.
pub const SAMPLE_OPEN: &str = "\
S........
.........
....#....
....#....
....#....
........G";

/// A corridor maze where greedy search is tempted into a dead end.
pub const SAMPLE_WALL: &str = "\
S.#......
..#.####.
..#.#..#.
..#.#.##.
....#...G
#####.#..";

/// A reproducible random maze: a `width × height` grid where each interior cell
/// is a wall with probability `obstacle_density`. The start `(0,0)` and goal
/// `(height-1, width-1)` are always left free. Same `seed` ⇒ same maze.
///
/// Note: the result is *not* guaranteed solvable — that is intentional, so the
/// "no path" case can be exercised. Check `SearchResult::found()`.
pub fn random_maze(width: i32, height: i32, obstacle_density: f64, seed: u64) -> Maze {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut grid = GridGraph::new(width, height);
    let start = Cell::new(0, 0);
    let goal = Cell::new(height - 1, width - 1);
    for r in 0..height {
        for c in 0..width {
            let cell = Cell::new(r, c);
            if cell == start || cell == goal {
                continue;
            }
            if rng.gen::<f64>() < obstacle_density {
                grid.block(cell);
            }
        }
    }
    Maze { grid, start, goal }
}
