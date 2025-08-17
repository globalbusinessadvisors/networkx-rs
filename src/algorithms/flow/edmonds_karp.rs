//! Edmonds-Karp algorithm for maximum flow

use super::MaxFlowResult;
use crate::graph::traits::GraphBase;
use crate::errors::{NetworkXError, Result};
use std::collections::{HashMap, VecDeque, HashSet};
use std::hash::Hash;

/// Compute maximum flow using Edmonds-Karp algorithm (BFS-based Ford-Fulkerson)
pub fn edmonds_karp<G, N>(
    graph: &G,
    source: N,
    sink: N,
) -> Result<MaxFlowResult<N>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    if source == sink {
        return Err(NetworkXError::InvalidInput(
            "Source and sink must be different nodes".to_string()
        ));
    }
    
    if !graph.has_node(&source) {
        return Err(NetworkXError::NodeNotFound(
            "Source node not found".to_string()
        ));
    }
    
    if !graph.has_node(&sink) {
        return Err(NetworkXError::NodeNotFound(
            "Sink node not found".to_string()
        ));
    }
    
    // Initialize flow dictionary
    let mut flow_dict: HashMap<(N, N), f64> = HashMap::new();
    
    // Build capacity dictionary
    let mut capacity: HashMap<(N, N), f64> = HashMap::new();
    for (u, v, weight) in graph.edges() {
        capacity.insert((u.clone(), v.clone()), weight);
        // Initialize flow to 0
        flow_dict.insert((u.clone(), v.clone()), 0.0);
        // For residual graph, we need reverse edges
        if !capacity.contains_key(&(v.clone(), u.clone())) {
            capacity.insert((v, u), 0.0);
        }
    }
    
    let mut max_flow = 0.0;
    
    // Find augmenting paths using BFS
    while let Some((path, bottleneck)) = bfs_augmenting_path(
        graph,
        &source,
        &sink,
        &capacity,
        &flow_dict,
    ) {
        // Update flow along the path
        for i in 0..path.len() - 1 {
            let u = path[i].clone();
            let v = path[i + 1].clone();
            
            // Update forward flow
            *flow_dict.entry((u.clone(), v.clone())).or_insert(0.0) += bottleneck;
            // Update reverse flow
            *flow_dict.entry((v, u)).or_insert(0.0) -= bottleneck;
        }
        
        max_flow += bottleneck;
    }
    
    // Clean up flow dictionary (remove zero and negative flows)
    flow_dict.retain(|_, &mut flow| flow > 0.0);
    
    Ok(MaxFlowResult {
        flow_value: max_flow,
        flow_dict,
    })
}

/// Find an augmenting path using BFS
fn bfs_augmenting_path<G, N>(
    graph: &G,
    source: &N,
    sink: &N,
    capacity: &HashMap<(N, N), f64>,
    flow: &HashMap<(N, N), f64>,
) -> Option<(Vec<N>, f64)>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let mut parent: HashMap<N, N> = HashMap::new();
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    
    queue.push_back(source.clone());
    visited.insert(source.clone());
    
    while let Some(node) = queue.pop_front() {
        if node == *sink {
            // Reconstruct path and find bottleneck
            let mut path = vec![sink.clone()];
            let mut current = sink.clone();
            let mut bottleneck = f64::INFINITY;
            
            while let Some(prev) = parent.get(&current) {
                path.push(prev.clone());
                
                // Calculate residual capacity
                let edge_capacity = capacity
                    .get(&(prev.clone(), current.clone()))
                    .copied()
                    .unwrap_or(0.0);
                let edge_flow = flow
                    .get(&(prev.clone(), current.clone()))
                    .copied()
                    .unwrap_or(0.0);
                let residual = edge_capacity - edge_flow;
                
                bottleneck = bottleneck.min(residual);
                current = prev.clone();
            }
            
            path.reverse();
            return Some((path, bottleneck));
        }
        
        // Explore neighbors in residual graph
        for neighbor in graph.neighbors(&node) {
            if !visited.contains(&neighbor) {
                let edge_capacity = capacity
                    .get(&(node.clone(), neighbor.clone()))
                    .copied()
                    .unwrap_or(0.0);
                let edge_flow = flow
                    .get(&(node.clone(), neighbor.clone()))
                    .copied()
                    .unwrap_or(0.0);
                
                if edge_capacity > edge_flow {
                    visited.insert(neighbor.clone());
                    parent.insert(neighbor.clone(), node.clone());
                    queue.push_back(neighbor);
                }
            }
        }
        
        // Also check reverse edges in residual graph
        for other in graph.nodes() {
            if graph.has_edge(&other, &node) && !visited.contains(&other) {
                let reverse_flow = flow
                    .get(&(other.clone(), node.clone()))
                    .copied()
                    .unwrap_or(0.0);
                
                if reverse_flow > 0.0 {
                    visited.insert(other.clone());
                    parent.insert(other.clone(), node.clone());
                    queue.push_back(other);
                }
            }
        }
    }
    
    None
}

/// Get just the flow value using Edmonds-Karp
pub fn edmonds_karp_flow<G, N>(
    graph: &G,
    source: N,
    sink: N,
) -> Result<f64>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let result = edmonds_karp(graph, source, sink)?;
    Ok(result.flow_value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::DiGraph;
    use crate::graph::traits::GraphBase;
    
    #[test]
    fn test_edmonds_karp_simple() {
        let mut graph = DiGraph::new();
        
        // Simple flow network
        graph.add_edge(0, 1, Some(10.0));
        graph.add_edge(0, 2, Some(10.0));
        graph.add_edge(1, 2, Some(2.0));
        graph.add_edge(1, 3, Some(4.0));
        graph.add_edge(1, 4, Some(8.0));
        graph.add_edge(2, 4, Some(9.0));
        graph.add_edge(3, 5, Some(10.0));
        graph.add_edge(4, 3, Some(6.0));
        graph.add_edge(4, 5, Some(10.0));
        
        let result = edmonds_karp(&graph, 0, 5).unwrap();
        
        // Maximum flow should be 19
        assert_eq!(result.flow_value, 19.0);
    }
    
    #[test]
    fn test_edmonds_karp_disconnected() {
        let mut graph = DiGraph::new();
        
        // Disconnected source and sink
        graph.add_edge(0, 1, Some(10.0));
        graph.add_edge(2, 3, Some(10.0));
        
        let result = edmonds_karp(&graph, 0, 3).unwrap();
        
        // No flow possible
        assert_eq!(result.flow_value, 0.0);
    }
    
    #[test]
    fn test_edmonds_karp_bottleneck() {
        let mut graph = DiGraph::new();
        
        // Path with bottleneck
        graph.add_edge(0, 1, Some(100.0));
        graph.add_edge(1, 2, Some(1.0)); // Bottleneck
        graph.add_edge(2, 3, Some(100.0));
        
        let result = edmonds_karp(&graph, 0, 3).unwrap();
        
        // Flow limited by bottleneck
        assert_eq!(result.flow_value, 1.0);
    }
}