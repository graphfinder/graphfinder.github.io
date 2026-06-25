//! Graph domains: the concrete worlds the algorithms search over.

mod csr;
mod grid;

pub use csr::CsrGraph;
pub use grid::{Cell, GridGraph};
