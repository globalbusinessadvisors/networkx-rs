//! Bellman-Ford algorithm for shortest paths with negative weights

use crate::graph::traits::GraphBase;
use crate::errors::{NetworkXError, Result};
use std::collections::HashMap;
use std::hash::Hash;

/// Bellman-Ford algorithm for shortest paths
///
/// Handles negative edge weights and detects negative cycles
///
/// # Arguments
/// * `graph` - The graph to search
/// * `source` - Starting node
/// * `weight_fn` - Function to get edge weights (defaults to 1.0)
///
/// # Returns
/// * `Ok((distances, predecessors))` - Distance map and predecessor map
/// * `Err` - If negative cycle detected or source not in graph
pub fn bellman_ford<G, N>(
    graph: &G,
    source: N,
    weight_fn: Option<impl Fn(&N, &N) -> f64>,
) -> Result<(HashMap<N, f64>, HashMap<N, N>)>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq + Ord,
{
    if !graph.has_node(&source) {
        return Err(NetworkXError::NodeNotFound(format!("Source node not found")));
    }

    
    let mut distances: HashMap<N, f64> = HashMap::new();
    let mut predecessors: HashMap<N, N> = HashMap::new();
    
    // Initialize distances
    for node in graph.nodes() {
        distances.insert(node.clone(), f64::INFINITY);
    }
    distances.insert(source.clone(), 0.0);
    
    let nodes: Vec<N> = graph.nodes().collect();
    let n = nodes.len();
    
    // Relax edges repeatedly
    for _ in 0..n - 1 {
        let mut updated = false;
        
        for u in &nodes {
            if distances[u] == f64::INFINITY {
                continue;
            }
            
            for v in graph.neighbors(u) {
                let w = if let Some(ref wf) = weight_fn {
                    wf(u, &v)
                } else {
                    1.0
                };
                let new_dist = distances[u] + w;
                
                if new_dist < distances[&v] {
                    distances.insert(v.clone(), new_dist);
                    predecessors.insert(v.clone(), u.clone());
                    updated = true;
                }
            }
        }
        
        if !updated {
            break;
        }
    }
    
    // Check for negative cycles
    for u in &nodes {
        if distances[u] == f64::INFINITY {
            continue;
        }
        
        for v in graph.neighbors(u) {
            let w = if let Some(ref wf) = weight_fn {
                wf(u, &v)
            } else {
                1.0
            };
            if distances[u] + w < distances[&v] {
                return Err(NetworkXError::NegativeCycle(
                    "Graph contains negative cycle".to_string()
                ));
            }
        }
    }
    
    Ok((distances, predecessors))
}

/// Get shortest path from source to target using Bellman-Ford
pub fn bellman_ford_path<G, N>(
    graph: &G,
    source: N,
    target: N,
    weight_fn: Option<impl Fn(&N, &N) -> f64>,
) -> Result<Option<(Vec<N>, f64)>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq + Ord,
{
    let (distances, predecessors) = bellman_ford(graph, source.clone(), weight_fn)?;
    
    if !distances.contains_key(&target) || distances[&target] == f64::INFINITY {
        return Ok(None);
    }
    
    // Reconstruct path
    let mut path = vec![target.clone()];
    let mut current = target.clone();
    
    while let Some(prev) = predecessors.get(&current) {
        if *prev == source {
            path.push(source.clone());
            break;
        }
        path.push(prev.clone());
        current = prev.clone();
    }
    
    path.reverse();
    Ok(Some((path, distances[&target])))
}

/// Check if graph has negative cycle reachable from source
pub fn has_negative_cycle<G, N>(
    graph: &G,
    source: N,
    weight_fn: Option<impl Fn(&N, &N) -> f64>,
) -> bool
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq + Ord,
{
    bellman_ford(graph, source, weight_fn).is_err()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::DiGraph;

    #[test]
    fn test_bellman_ford_positive_weights() {
        let mut graph = DiGraph::new();
        graph.add_edge(0, 1, Some(5.0));
        graph.add_edge(0, 2, Some(3.0));
        graph.add_edge(1, 2, Some(1.0));
        graph.add_edge(2, 3, Some(2.0));
        
        let (distances, _) = bellman_ford(&graph, 0, Some(|u: &i32, v: &i32| {
            graph.get_edge_weight(u, v).unwrap_or(1.0)
        })).unwrap();
        
        assert_eq!(distances[&0], 0.0);
        assert_eq!(distances[&1], 5.0);
        assert_eq!(distances[&2], 3.0);
        assert_eq!(distances[&3], 5.0);
    }
    
    #[test]
    fn test_bellman_ford_negative_weights() {
        let mut graph = DiGraph::new();
        graph.add_edge(0, 1, Some(1.0));
        graph.add_edge(1, 2, Some(-3.0));
        graph.add_edge(2, 3, Some(2.0));
        
        let (distances, _) = bellman_ford(&graph, 0, Some(|u: &i32, v: &i32| {
            graph.get_edge_weight(u, v).unwrap_or(1.0)
        })).unwrap();
        
        assert_eq!(distances[&0], 0.0);
        assert_eq!(distances[&1], 1.0);
        assert_eq!(distances[&2], -2.0);
        assert_eq!(distances[&3], 0.0);
    }
    
    #[test]
    fn test_negative_cycle_detection() {
        let mut graph = DiGraph::new();
        graph.add_edge(0, 1, Some(1.0));
        graph.add_edge(1, 2, Some(-3.0));
        graph.add_edge(2, 0, Some(1.0)); // Creates negative cycle
        
        assert!(has_negative_cycle(&graph, 0, Some(|u: &i32, v: &i32| {
            graph.get_edge_weight(u, v).unwrap_or(1.0)
        })));
    }
}