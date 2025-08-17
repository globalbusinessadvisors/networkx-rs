//! Ford-Fulkerson algorithm for maximum flow

use super::MaxFlowResult;
use crate::graph::traits::GraphBase;
use crate::errors::Result;
use std::hash::Hash;

/// Compute maximum flow using Ford-Fulkerson algorithm with DFS
pub fn ford_fulkerson<G, N>(
    graph: &G,
    source: N,
    sink: N,
) -> Result<MaxFlowResult<N>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    use std::collections::HashMap;
    
    let mut flow_dict: HashMap<(N, N), f64> = HashMap::new();
    let mut total_flow = 0.0;
    
    // Initialize flow to 0 for all edges
    for node in graph.nodes() {
        for neighbor in graph.neighbors(&node) {
            flow_dict.insert((node.clone(), neighbor), 0.0);
        }
    }
    
    // Ford-Fulkerson with DFS-based path finding
    loop {
        // Find an augmenting path using DFS
        let mut visited = std::collections::HashSet::new();
        let mut parent: HashMap<N, N> = HashMap::new();
        
        if let Some(path_flow) = dfs_find_path(
            graph,
            &source,
            &sink,
            &flow_dict,
            &mut visited,
            &mut parent,
            f64::INFINITY,
        ) {
            // Reconstruct the path
            let mut path = Vec::new();
            let mut current = sink.clone();
            
            while current != source {
                let prev = parent.get(&current).unwrap().clone();
                path.push((prev.clone(), current.clone()));
                current = prev;
            }
            path.reverse();
            
            // Update flow along the path
            for (u, v) in path {
                let current_flow = flow_dict.get(&(u.clone(), v.clone())).copied().unwrap_or(0.0);
                flow_dict.insert((u.clone(), v.clone()), current_flow + path_flow);
                
                // Update reverse flow
                let reverse_flow = flow_dict.get(&(v.clone(), u.clone())).copied().unwrap_or(0.0);
                flow_dict.insert((v, u), reverse_flow - path_flow);
            }
            
            total_flow += path_flow;
        } else {
            break;
        }
    }
    
    Ok(MaxFlowResult {
        flow_value: total_flow,
        flow_dict,
    })
}

/// DFS-based augmenting path finding for Ford-Fulkerson
fn dfs_find_path<G, N>(
    graph: &G,
    current: &N,
    sink: &N,
    flow_dict: &std::collections::HashMap<(N, N), f64>,
    visited: &mut std::collections::HashSet<N>,
    parent: &mut std::collections::HashMap<N, N>,
    min_capacity: f64,
) -> Option<f64>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    if current == sink {
        return Some(min_capacity);
    }
    
    visited.insert(current.clone());
    
    for neighbor in graph.neighbors(current) {
        if !visited.contains(&neighbor) {
            let capacity = graph.get_edge_weight(current, &neighbor).unwrap_or(0.0);
            let current_flow = flow_dict.get(&(current.clone(), neighbor.clone())).copied().unwrap_or(0.0);
            let residual_capacity = capacity - current_flow;
            
            if residual_capacity > 0.0 {
                parent.insert(neighbor.clone(), current.clone());
                let bottleneck = min_capacity.min(residual_capacity);
                
                if let Some(path_flow) = dfs_find_path(
                    graph,
                    &neighbor,
                    sink,
                    flow_dict,
                    visited,
                    parent,
                    bottleneck,
                ) {
                    return Some(path_flow);
                }
            }
        }
    }
    
    None
}

/// Get just the flow value
pub fn ford_fulkerson_flow<G, N>(
    graph: &G,
    source: N,
    sink: N,
) -> Result<f64>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let result = ford_fulkerson(graph, source, sink)?;
    Ok(result.flow_value)
}