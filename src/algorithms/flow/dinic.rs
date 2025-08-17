//! Dinic's algorithm for maximum flow

use super::MaxFlowResult;
use crate::graph::traits::GraphBase;
use crate::errors::Result;
use std::hash::Hash;

/// Compute maximum flow using Dinic's algorithm
pub fn dinic<G, N>(
    graph: &G,
    source: N,
    sink: N,
) -> Result<MaxFlowResult<N>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    use std::collections::{HashMap, VecDeque};
    
    let mut flow_dict: HashMap<(N, N), f64> = HashMap::new();
    let mut total_flow = 0.0;
    
    // Initialize flow to 0 for all edges
    for node in graph.nodes() {
        for neighbor in graph.neighbors(&node) {
            flow_dict.insert((node.clone(), neighbor), 0.0);
        }
    }
    
    // Dinic's algorithm main loop
    loop {
        // Build level graph using BFS
        let level_graph = build_level_graph(graph, &source, &sink, &flow_dict);
        
        if level_graph.is_empty() {
            break; // No augmenting path exists
        }
        
        // Find blocking flow in the level graph
        while let Some(path_flow) = find_blocking_flow(
            graph,
            &source,
            &sink,
            &mut flow_dict,
            &level_graph,
        ) {
            total_flow += path_flow;
        }
    }
    
    Ok(MaxFlowResult {
        flow_value: total_flow,
        flow_dict,
    })
}

/// Build level graph for Dinic's algorithm
fn build_level_graph<G, N>(
    graph: &G,
    source: &N,
    sink: &N,
    flow_dict: &std::collections::HashMap<(N, N), f64>,
) -> std::collections::HashMap<N, usize>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    use std::collections::{HashMap, VecDeque};
    
    let mut level: HashMap<N, usize> = HashMap::new();
    let mut queue = VecDeque::new();
    
    level.insert(source.clone(), 0);
    queue.push_back(source.clone());
    
    while let Some(current) = queue.pop_front() {
        if current == *sink {
            break;
        }
        
        let current_level = level[&current];
        
        for neighbor in graph.neighbors(&current) {
            if !level.contains_key(&neighbor) {
                let capacity = graph.get_edge_weight(&current, &neighbor).unwrap_or(0.0);
                let current_flow = flow_dict.get(&(current.clone(), neighbor.clone())).copied().unwrap_or(0.0);
                let residual_capacity = capacity - current_flow;
                
                if residual_capacity > 0.0 {
                    level.insert(neighbor.clone(), current_level + 1);
                    queue.push_back(neighbor);
                }
            }
        }
    }
    
    level
}

/// Find blocking flow using DFS
fn find_blocking_flow<G, N>(
    graph: &G,
    source: &N,
    sink: &N,
    flow_dict: &mut std::collections::HashMap<(N, N), f64>,
    level_graph: &std::collections::HashMap<N, usize>,
) -> Option<f64>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    use std::collections::HashMap;
    
    let mut visited = std::collections::HashSet::new();
    let mut path = Vec::new();
    
    if let Some(path_flow) = dfs_blocking_flow(
        graph,
        source,
        sink,
        flow_dict,
        level_graph,
        &mut visited,
        &mut path,
        f64::INFINITY,
    ) {
        // Update flow along the path
        for (u, v) in &path {
            let current_flow = flow_dict.get(&(u.clone(), v.clone())).copied().unwrap_or(0.0);
            flow_dict.insert((u.clone(), v.clone()), current_flow + path_flow);
            
            // Update reverse flow
            let reverse_flow = flow_dict.get(&(v.clone(), u.clone())).copied().unwrap_or(0.0);
            flow_dict.insert((v.clone(), u.clone()), reverse_flow - path_flow);
        }
        
        Some(path_flow)
    } else {
        None
    }
}

/// DFS for finding blocking flow in level graph
fn dfs_blocking_flow<G, N>(
    graph: &G,
    current: &N,
    sink: &N,
    flow_dict: &std::collections::HashMap<(N, N), f64>,
    level_graph: &std::collections::HashMap<N, usize>,
    visited: &mut std::collections::HashSet<N>,
    path: &mut Vec<(N, N)>,
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
    let current_level = level_graph.get(current)?;
    
    for neighbor in graph.neighbors(current) {
        if let Some(&neighbor_level) = level_graph.get(&neighbor) {
            if neighbor_level == current_level + 1 && !visited.contains(&neighbor) {
                let capacity = graph.get_edge_weight(current, &neighbor).unwrap_or(0.0);
                let current_flow = flow_dict.get(&(current.clone(), neighbor.clone())).copied().unwrap_or(0.0);
                let residual_capacity = capacity - current_flow;
                
                if residual_capacity > 0.0 {
                    path.push((current.clone(), neighbor.clone()));
                    let bottleneck = min_capacity.min(residual_capacity);
                    
                    if let Some(path_flow) = dfs_blocking_flow(
                        graph,
                        &neighbor,
                        sink,
                        flow_dict,
                        level_graph,
                        visited,
                        path,
                        bottleneck,
                    ) {
                        return Some(path_flow);
                    }
                    
                    path.pop();
                }
            }
        }
    }
    
    None
}

/// Get just the flow value
pub fn dinic_flow<G, N>(
    graph: &G,
    source: N,
    sink: N,
) -> Result<f64>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let result = dinic(graph, source, sink)?;
    Ok(result.flow_value)
}