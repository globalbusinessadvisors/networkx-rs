//! Utility functions and helpers

use crate::graph::Node;
use std::collections::HashMap;

/// Convert node labels to indices
pub fn create_node_index_map<T: Clone + Eq + std::hash::Hash>(
    nodes: &[T]
) -> (HashMap<T, Node>, Vec<T>) {
    let mut node_to_index = HashMap::new();
    let mut index_to_node = Vec::new();
    
    for (i, node) in nodes.iter().enumerate() {
        node_to_index.insert(node.clone(), i);
        index_to_node.push(node.clone());
    }
    
    (node_to_index, index_to_node)
}

/// Parallel iteration utilities
#[cfg(feature = "parallel")]
pub mod parallel {
    use rayon::prelude::*;
    
    pub fn parallel_map<T, U, F>(items: Vec<T>, f: F) -> Vec<U>
    where
        T: Send + Sync,
        U: Send + Sync,
        F: Fn(T) -> U + Send + Sync,
    {
        items.into_par_iter().map(f).collect()
    }
}