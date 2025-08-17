//! Graph coloring algorithms

pub mod greedy;
pub mod dsatur;
pub mod welsh_powell;
pub mod chromatic;

pub use greedy::{greedy_color, greedy_color_with_interchange};
pub use dsatur::dsatur_coloring;
pub use welsh_powell::welsh_powell_coloring;
pub use chromatic::{chromatic_number, is_k_colorable, chromatic_polynomial};

use std::collections::HashMap;
use std::hash::Hash;

/// Result of a graph coloring
#[derive(Debug, Clone)]
pub struct ColoringResult<N> {
    /// The color assignment for each node
    pub coloring: HashMap<N, usize>,
    /// The number of colors used
    pub num_colors: usize,
}

impl<N: Clone + Hash + Eq> ColoringResult<N> {
    /// Create a new coloring result
    pub fn new(coloring: HashMap<N, usize>) -> Self {
        let num_colors = coloring.values().max().copied().unwrap_or(0) + 1;
        ColoringResult { coloring, num_colors }
    }
    
    /// Check if the coloring is valid (no adjacent nodes have same color)
    pub fn is_valid<G>(&self, graph: &G) -> bool
    where
        G: crate::graph::traits::GraphBase<NodeId = N>,
    {
        for (node, color) in &self.coloring {
            for neighbor in graph.neighbors(node) {
                if let Some(neighbor_color) = self.coloring.get(&neighbor) {
                    if color == neighbor_color {
                        return false;
                    }
                }
            }
        }
        true
    }
}