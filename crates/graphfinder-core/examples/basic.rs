//! Basic example: run A* on a small ASCII maze and print the path it found,
//! overlaid on the map. Run with:
//!
//! ```text
//! cargo run --example basic -p graphfinder-core
//! ```

use std::collections::HashSet;

use graphfinder_core::domains::{Maze, SAMPLE_OPEN};
use graphfinder_core::{search, Algorithm, Cell, Manhattan};

fn main() {
    let maze = Maze::from_ascii(SAMPLE_OPEN);
    let result = search(
        &maze.grid,
        maze.start,
        maze.goal,
        Algorithm::astar(),
        &Manhattan,
        true,
    );

    println!("A* on a {}x{} grid", maze.grid.width(), maze.grid.height());
    match &result.path {
        Some(path) => {
            println!(
                "path found: cost = {}, {} steps",
                result.cost,
                path.len() - 1
            );
            println!("nodes expanded = {}", result.nodes_expanded);
            println!("peak frontier  = {}", result.max_frontier_size);
            println!();
            print_map(&maze, path);
        }
        None => println!("no path exists ({:?})", result.stop_reason),
    }
}

/// Render the maze with the path drawn as `*`, start `S`, goal `G`, walls `#`.
fn print_map(maze: &Maze, path: &[Cell]) {
    let on_path: HashSet<Cell> = path.iter().copied().collect();
    for row in 0..maze.grid.height() {
        let mut line = String::new();
        for col in 0..maze.grid.width() {
            let c = Cell::new(row, col);
            let ch = if c == maze.start {
                'S'
            } else if c == maze.goal {
                'G'
            } else if maze.grid.is_blocked(c) {
                '#'
            } else if on_path.contains(&c) {
                '*'
            } else {
                '.'
            };
            line.push(ch);
        }
        println!("{line}");
    }
}
