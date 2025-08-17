//! Utility functions for NetworkX-RS

use std::collections::HashMap;
use std::hash::Hash;

/// Helper function to create a mapping from nodes to indices
pub fn create_node_index_map<N: Hash + Eq + Clone>(nodes: &[N]) -> HashMap<N, usize> {
    nodes.iter()
        .enumerate()
        .map(|(i, n)| (n.clone(), i))
        .collect()
}

/// Helper function to calculate the degree of nodes
pub fn calculate_degrees<N: Hash + Eq + Clone>(
    edges: &[(N, N)],
) -> HashMap<N, usize> {
    let mut degrees = HashMap::new();
    for (u, v) in edges {
        *degrees.entry(u.clone()).or_insert(0) += 1;
        *degrees.entry(v.clone()).or_insert(0) += 1;
    }
    degrees
}
