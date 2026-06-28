//! # graphfinder-core
//!
//! A trait-based, instrumented core for **graph traversal and pathfinding**,
//! covering both **uninformed** (BFS, DFS, UCS/Dijkstra) and **informed**
//! (Greedy, A*, Weighted A*) search. Sibling project to `turboswarm`; same
//! design priorities, in order: **visualization, algorithm comparison, code
//! clarity, performance**.
//!
//! ## The one idea
//!
//! All of these algorithms are the *same* loop ([`search`]) — Russell &
//! Norvig's GENERAL-SEARCH — differing only in:
//!
//! - the **[`Frontier`]** (FIFO → BFS, LIFO → DFS, priority queue → the rest), and
//! - the evaluation function `priority = g_coeff·g(n) + h_coeff·h(n)`,
//!
//! both bundled in an [`Algorithm`]. See [`Algorithm::bfs`], [`Algorithm::astar`], …
//!
//! ## Quickstart
//!
//! ```
//! use graphfinder_core::{search, Algorithm, GridGraph, Manhattan};
//!
//! let (grid, start, goal) = GridGraph::from_ascii(
//!     "S....\n.###.\n...#.\n.#.#.\n.#..G",
//! );
//! let result = search(&grid, start, goal, Algorithm::astar(), &Manhattan, true);
//! assert!(result.found());
//! println!("cost={} expanded={}", result.cost, result.nodes_expanded);
//! ```

pub mod domains;
pub mod frontier;
pub mod graph;
pub mod heuristic;
pub mod puzzles;
pub mod search;
pub mod shortest_paths;
pub mod strategies;
pub mod traits;

pub use frontier::{Fifo, Lifo, PriorityQueue};
pub use graph::{Cell, CsrGraph, GridGraph};
pub use heuristic::{Euclidean, Manhattan, Octile, Zero};
pub use puzzles::{
    Hanoi, HanoiMisplaced, LadderHamming, NPuzzle, PuzzleManhattan, PuzzleMisplaced, WordLadder,
};
pub use search::{
    search, search_with, Algorithm, FrontierKind, SearchResult, StopReason, TraceStep,
};
pub use shortest_paths::{bellman_ford, floyd_warshall, AllPairs, ShortestPaths};
pub use strategies::{beam_search, bidirectional, dls, ida_star, iddfs};
pub use traits::{Frontier, Graph, Heuristic};
