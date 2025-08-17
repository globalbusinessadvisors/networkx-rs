//! Canonical labeling and automorphism detection

use crate::graph::traits::GraphBase;
use crate::errors::Result;
use std::hash::Hash;

/// Compute canonical labeling of a graph (basic implementation using degree sequence)
pub fn canonical_labeling<G, N>(graph: &G) -> Result<Vec<N>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    use std::collections::HashMap;
    
    let nodes: Vec<N> = graph.nodes().collect();
    if nodes.is_empty() {
        return Ok(Vec::new());
    }
    
    // Basic canonical labeling using degree sequence and neighbor degrees
    let mut node_signatures: Vec<(N, Vec<usize>)> = Vec::new();
    
    for node in &nodes {
        let degree = graph.degree(node);
        let mut neighbor_degrees: Vec<usize> = graph.neighbors(node)
            .map(|neighbor| graph.degree(&neighbor))
            .collect();
        neighbor_degrees.sort_unstable();
        
        let mut signature = vec![degree];
        signature.extend(neighbor_degrees);
        
        node_signatures.push((node.clone(), signature));
    }
    
    // Sort by signature to get canonical ordering
    node_signatures.sort_by(|a, b| a.1.cmp(&b.1));
    
    Ok(node_signatures.into_iter().map(|(node, _)| node).collect())
}

/// Check if a mapping is an automorphism
pub fn is_automorphism<G, N>(graph: &G, mapping: &std::collections::HashMap<N, N>) -> bool
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    // Check if mapping preserves adjacency
    for node in graph.nodes() {
        if let Some(mapped_node) = mapping.get(&node) {
            // Check if the degree is preserved
            if graph.degree(&node) != graph.degree(mapped_node) {
                return false;
            }
            
            // Check if adjacency is preserved for all neighbors
            for neighbor in graph.neighbors(&node) {
                if let Some(mapped_neighbor) = mapping.get(&neighbor) {
                    if graph.has_edge(&node, &neighbor) != graph.has_edge(mapped_node, mapped_neighbor) {
                        return false;
                    }
                } else {
                    return false; // Incomplete mapping
                }
            }
        } else {
            return false; // Incomplete mapping
        }
    }
    
    true
}