//! Problem instances ("benchmarks") with known structure, for examples,
//! teaching and tests — the analogue of turboswarm's benchmark functions.

mod graphs;
mod maze;

pub use graphs::{barabasi_albert, erdos_renyi, watts_strogatz};
pub use maze::{random_maze, Maze, SAMPLE_OPEN, SAMPLE_WALL};
