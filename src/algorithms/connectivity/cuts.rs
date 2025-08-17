//! Cut algorithms for graphs

use crate::graph::traits::GraphBase;
use crate::errors::Result;
use std::collections::HashSet;
use std::hash::Hash;

/// Find the minimum cut of a graph using max flow min cut theorem
pub fn minimum_cut<G, N>(graph: &G) -> Result<(f64, HashSet<N>, HashSet<N>)>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let nodes: Vec<N> = graph.nodes().collect();
    if nodes.len() < 2 {
        return Ok((0.0, HashSet::new(), HashSet::new()));
    }
    
    // Use max flow between arbitrary source and sink for global min cut
    let source = nodes[0].clone();
    let sink = nodes[1].clone();
    
    // Use the flow module's minimum_cut function
    crate::algorithms::flow::minimum_cut(graph, source, sink)
}

/// Find the minimum edge cut
pub fn minimum_edge_cut<G, N>(graph: &G) -> Result<Vec<(N, N)>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let (cut_value, left_set, right_set) = minimum_cut(graph)?;
    let mut cut_edges = Vec::new();
    
    // Find edges crossing the cut
    for left_node in &left_set {
        for neighbor in graph.neighbors(left_node) {
            if right_set.contains(&neighbor) {
                cut_edges.push((left_node.clone(), neighbor));
            }
        }
    }
    
    Ok(cut_edges)
}

/// Compute node connectivity (simplified implementation)
pub fn node_connectivity<G, N>(graph: &G) -> Result<usize>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let nodes: Vec<N> = graph.nodes().collect();
    let n = nodes.len();
    
    if n <= 1 {
        return Ok(0);
    }
    
    if n == 2 {
        return Ok(if graph.has_edge(&nodes[0], &nodes[1]) { 1 } else { 0 });
    }
    
    // For each pair of non-adjacent nodes, find min cut
    let mut min_connectivity = usize::MAX;
    
    for i in 0..n {
        for j in (i + 1)..n {
            if !graph.has_edge(&nodes[i], &nodes[j]) {
                // Use max flow to find min cut (simplified)
                // This is a basic approximation - full implementation would need node splitting
                let (cut_value, _, _) = minimum_cut(graph)?;
                min_connectivity = min_connectivity.min(cut_value as usize);
            }
        }
    }
    
    if min_connectivity == usize::MAX {
        // All nodes are adjacent, connectivity is n-1
        Ok(n - 1)
    } else {
        Ok(min_connectivity)
    }
}

/// Compute edge connectivity
pub fn edge_connectivity<G, N>(graph: &G) -> Result<usize>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let nodes: Vec<N> = graph.nodes().collect();
    if nodes.len() < 2 {
        return Ok(0);
    }
    
    // Edge connectivity is the minimum cut value over all pairs
    let mut min_connectivity = usize::MAX;
    
    for i in 0..nodes.len() {
        for j in (i + 1)..nodes.len() {
            let (cut_value, _, _) = crate::algorithms::flow::minimum_cut(
                graph, 
                nodes[i].clone(), 
                nodes[j].clone()
            )?;
            min_connectivity = min_connectivity.min(cut_value as usize);
        }
    }
    
    Ok(if min_connectivity == usize::MAX { 0 } else { min_connectivity })
}

/// Check if graph is k-edge connected
pub fn is_k_edge_connected<G, N>(graph: &G, k: usize) -> Result<bool>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let connectivity = edge_connectivity(graph)?;
    Ok(connectivity >= k)
}