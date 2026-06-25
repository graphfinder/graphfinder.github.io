//! Frontier implementations — one per family of search algorithm.
//!
//! Each type implements [`crate::traits::Frontier`]. The search loop is written
//! once against the trait; swapping the concrete frontier swaps the algorithm.

use std::cmp::Ordering;
use std::collections::{BinaryHeap, VecDeque};

use crate::traits::Frontier;

/// First-in, first-out queue → **Breadth-First Search**. Ignores priority.
#[derive(Debug, Default)]
pub struct Fifo<N> {
    queue: VecDeque<N>,
}

impl<N> Fifo<N> {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }
}

impl<N> Frontier<N> for Fifo<N> {
    fn push(&mut self, node: N, _priority: f64) {
        self.queue.push_back(node);
    }
    fn pop(&mut self) -> Option<N> {
        self.queue.pop_front()
    }
    fn len(&self) -> usize {
        self.queue.len()
    }
}

/// Last-in, first-out stack → **Depth-First Search**. Ignores priority.
#[derive(Debug, Default)]
pub struct Lifo<N> {
    stack: Vec<N>,
}

impl<N> Lifo<N> {
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }
}

impl<N> Frontier<N> for Lifo<N> {
    fn push(&mut self, node: N, _priority: f64) {
        self.stack.push(node);
    }
    fn pop(&mut self) -> Option<N> {
        self.stack.pop()
    }
    fn len(&self) -> usize {
        self.stack.len()
    }
}

/// A frontier entry: the node plus the key it is ordered by. `seq` gives a
/// deterministic FIFO tie-break among equal priorities (reproducibility).
struct Entry<N> {
    priority: f64,
    seq: u64,
    node: N,
}

impl<N> PartialEq for Entry<N> {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.seq == other.seq
    }
}
impl<N> Eq for Entry<N> {}

impl<N> Ord for Entry<N> {
    fn cmp(&self, other: &Self) -> Ordering {
        // `BinaryHeap` is a max-heap; we want the *smallest* priority to pop
        // first, so we reverse the priority comparison. Ties break on the
        // smaller `seq` (inserted earlier → popped earlier).
        other
            .priority
            .partial_cmp(&self.priority)
            .unwrap_or(Ordering::Equal)
            .then_with(|| other.seq.cmp(&self.seq))
    }
}
impl<N> PartialOrd for Entry<N> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Min-priority queue → **Uniform-Cost Search / Dijkstra / Greedy / A***,
/// depending on the priority the loop pushes. Pops the lowest-priority node.
#[derive(Default)]
pub struct PriorityQueue<N> {
    heap: BinaryHeap<Entry<N>>,
    seq: u64,
}

impl<N> PriorityQueue<N> {
    pub fn new() -> Self {
        Self {
            heap: BinaryHeap::new(),
            seq: 0,
        }
    }
}

impl<N> Frontier<N> for PriorityQueue<N> {
    fn push(&mut self, node: N, priority: f64) {
        self.heap.push(Entry {
            priority,
            seq: self.seq,
            node,
        });
        self.seq += 1;
    }
    fn pop(&mut self) -> Option<N> {
        self.heap.pop().map(|e| e.node)
    }
    fn len(&self) -> usize {
        self.heap.len()
    }
}
