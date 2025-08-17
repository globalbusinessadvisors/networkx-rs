//! Path connectivity algorithms

use crate::graph::traits::GraphBase;
use crate::errors::Result;
use std::hash::Hash;

/// Check if there is a path between two nodes
pub fn has_path<G, N>(graph: &G, source: N, target: N) -> Result<bool>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    use std::collections::{HashSet, VecDeque};
    
    if source == target {
        return Ok(true);
    }
    
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    
    queue.push_back(source);
    
    while let Some(node) = queue.pop_front() {
        if node == target {
            return Ok(true);
        }
        
        if visited.insert(node.clone()) {
            for neighbor in graph.neighbors(&node) {
                if !visited.contains(&neighbor) {
                    queue.push_back(neighbor);
                }
            }
        }
    }
    
    Ok(false)
}

/// Find node-disjoint paths between two nodes
pub fn node_disjoint_paths<G, N>(
    graph: &G,
    source: N,
    target: N,
    k: Option<usize>,
) -> Result<Vec<Vec<N>>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    todo!("Implement node-disjoint paths using max flow")
}

/// Find edge-disjoint paths between two nodes
pub fn edge_disjoint_paths<G, N>(
    graph: &G,
    source: N,
    target: N,
    k: Option<usize>,
) -> Result<Vec<Vec<N>>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    todo!("Implement edge-disjoint paths using max flow")
}