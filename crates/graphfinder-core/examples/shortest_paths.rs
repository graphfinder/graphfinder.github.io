//! Bellman–Ford and Floyd–Warshall on a small directed graph with a **negative
//! edge** — the case Dijkstra/A\* cannot handle. Shows single-source distances,
//! all-pairs distances, and negative-cycle detection.
//!
//! ```text
//! cargo run --example shortest_paths -p graphfinder-core
//! ```

use graphfinder_core::{bellman_ford, floyd_warshall, CsrGraph};

fn main() {
    // A directed graph where the cheapest 0→2 route goes *through* node 1 over a
    // negative edge (cost 4 + (−3) = 1), beating the direct edge of cost 5.
    let edges = [
        (0, 1, 4.0),
        (0, 2, 5.0),
        (1, 2, -3.0),
        (2, 3, 2.0),
        (1, 3, 6.0),
    ];
    let g = CsrGraph::from_edges(4, &edges, /* undirected = */ false);

    println!("== Bellman–Ford (single source 0) ==");
    let sp = bellman_ford(&g, 0);
    println!("negative cycle reachable: {}", sp.negative_cycle);
    for t in 0..g.num_nodes() {
        match sp.path_to(t) {
            Some(path) => println!("0 → {t}: cost {:>4}  via {:?}", sp.dist[t], path),
            None => println!("0 → {t}: unreachable"),
        }
    }

    println!("\n== Floyd–Warshall (all pairs) ==");
    let ap = floyd_warshall(&g);
    print!("     ");
    for j in 0..ap.num_nodes() {
        print!("{j:>5}");
    }
    println!();
    for i in 0..ap.num_nodes() {
        print!("{i:>3}: ");
        for j in 0..ap.num_nodes() {
            let d = ap.distance(i, j);
            if d.is_finite() {
                print!("{d:>5}");
            } else {
                print!("    ∞");
            }
        }
        println!();
    }
    println!("\npath 0 → 3: {:?}", ap.path(0, 3));

    // Add an edge 2 → 0 of cost −4: now 0→1→2→0 sums to −3, a negative cycle.
    println!("\n== With a negative cycle (add 2 → 0 = −4) ==");
    let mut with_cycle = edges.to_vec();
    with_cycle.push((2, 0, -4.0));
    let g2 = CsrGraph::from_edges(4, &with_cycle, false);
    println!(
        "Bellman–Ford negative_cycle = {}",
        bellman_ford(&g2, 0).negative_cycle
    );
    println!(
        "Floyd–Warshall negative_cycle = {}",
        floyd_warshall(&g2).negative_cycle
    );
}
