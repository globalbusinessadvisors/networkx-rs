//! Closeness centrality algorithm

use crate::graph::traits::GraphBase;
use crate::errors::Result;
use std::collections::{HashMap, VecDeque};
use std::hash::Hash;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

/// Calculate closeness centrality for all nodes in the graph
///
/// Closeness centrality of a node is the reciprocal of the sum of the shortest path
/// distances from the node to all other nodes in the graph. Since the sum of distances
/// depends on the number of nodes in the graph, closeness is normalized by the sum of
/// minimum possible distances (n-1).
pub fn closeness_centrality<G, N>(
    graph: &G,
    normalized: bool,
) -> Result<HashMap<N, f64>>
where
    G: GraphBase<NodeId = N> + Sync,
    N: Clone + Hash + Eq + Send + Sync,
{
    let nodes: Vec<N> = graph.nodes().collect();
    let n = nodes.len() as f64;
    
    #[cfg(feature = "parallel")]
    let centrality: HashMap<N, f64> = nodes
        .par_iter()
        .map(|node| {
            let closeness = calculate_closeness_for_node(graph, node.clone(), n, normalized);
            (node.clone(), closeness)
        })
        .collect();
    
    #[cfg(not(feature = "parallel"))]
    let centrality: HashMap<N, f64> = nodes
        .iter()
        .map(|node| {
            let closeness = calculate_closeness_for_node(graph, node.clone(), n, normalized);
            (node.clone(), closeness)
        })
        .collect();
    
    Ok(centrality)
}

/// Calculate closeness centrality for a single node
fn calculate_closeness_for_node<G, N>(
    graph: &G,
    source: N,
    total_nodes: f64,
    normalized: bool,
) -> f64
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let distances = single_source_shortest_path_length(graph, source);
    
    // Calculate sum of distances
    let mut total_distance = 0.0;
    let mut reachable_nodes = 0;
    
    for (_, distance) in distances {
        if distance > 0 {
            total_distance += distance as f64;
            reachable_nodes += 1;
        }
    }
    
    if reachable_nodes == 0 {
        return 0.0;
    }
    
    // Calculate closeness
    let closeness = reachable_nodes as f64 / total_distance;
    
    // Normalize if requested
    if normalized && total_nodes > 1.0 {
        closeness * (reachable_nodes as f64 / (total_nodes - 1.0))
    } else {
        closeness
    }
}

/// Calculate shortest path lengths from a source node using BFS
fn single_source_shortest_path_length<G, N>(
    graph: &G,
    source: N,
) -> HashMap<N, usize>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let mut distances: HashMap<N, usize> = HashMap::new();
    let mut queue = VecDeque::new();
    
    distances.insert(source.clone(), 0);
    queue.push_back(source);
    
    while let Some(current) = queue.pop_front() {
        let current_distance = distances[&current];
        
        for neighbor in graph.neighbors(&current) {
            if !distances.contains_key(&neighbor) {
                distances.insert(neighbor.clone(), current_distance + 1);
                queue.push_back(neighbor);
            }
        }
    }
    
    distances
}

/// Calculate harmonic centrality (variant of closeness centrality)
///
/// Harmonic centrality is a variant of closeness centrality that can handle
/// disconnected graphs better. It is defined as the sum of the reciprocal of
/// shortest path distances.
pub fn harmonic_centrality<G, N>(
    graph: &G,
    normalized: bool,
) -> Result<HashMap<N, f64>>
where
    G: GraphBase<NodeId = N> + Sync,
    N: Clone + Hash + Eq + Send + Sync,
{
    let nodes: Vec<N> = graph.nodes().collect();
    let n = nodes.len() as f64;
    
    #[cfg(feature = "parallel")]
    let centrality: HashMap<N, f64> = nodes
        .par_iter()
        .map(|node| {
            let harmonic = calculate_harmonic_for_node(graph, node.clone(), n, normalized);
            (node.clone(), harmonic)
        })
        .collect();
    
    #[cfg(not(feature = "parallel"))]
    let centrality: HashMap<N, f64> = nodes
        .iter()
        .map(|node| {
            let harmonic = calculate_harmonic_for_node(graph, node.clone(), n, normalized);
            (node.clone(), harmonic)
        })
        .collect();
    
    Ok(centrality)
}

/// Calculate harmonic centrality for a single node
fn calculate_harmonic_for_node<G, N>(
    graph: &G,
    source: N,
    total_nodes: f64,
    normalized: bool,
) -> f64
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let distances = single_source_shortest_path_length(graph, source);
    
    // Calculate sum of reciprocals
    let mut harmonic_sum = 0.0;
    
    for (_, distance) in distances {
        if distance > 0 {
            harmonic_sum += 1.0 / distance as f64;
        }
    }
    
    // Normalize if requested
    if normalized && total_nodes > 1.0 {
        harmonic_sum / (total_nodes - 1.0)
    } else {
        harmonic_sum
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::Graph;
    
    #[test]
    fn test_closeness_centrality_path() {
        let mut graph = Graph::new();
        // Create a path graph: 1-2-3-4
        graph.add_edge(1, 2, None);
        graph.add_edge(2, 3, None);
        graph.add_edge(3, 4, None);
        
        let centrality = closeness_centrality(&graph, false).unwrap();
        
        // Middle nodes should have higher closeness
        assert!(centrality[&2] > centrality[&1]);
        assert!(centrality[&3] > centrality[&4]);
        assert!(centrality[&2] == centrality[&3]); // Symmetry in undirected path
    }
    
    #[test]
    fn test_closeness_centrality_star() {
        let mut graph = Graph::new();
        // Create a star graph with center at node 0
        for i in 1..6 {
            graph.add_edge(0, i, None);
        }
        
        let centrality = closeness_centrality(&graph, false).unwrap();
        
        // Center node should have maximum closeness
        for i in 1..6 {
            assert!(centrality[&0] > centrality[&i]);
        }
        
        // All leaf nodes should have equal closeness
        for i in 2..6 {
            assert_eq!(centrality[&1], centrality[&i]);
        }
    }
    
    #[test]
    fn test_harmonic_centrality() {
        let mut graph = Graph::new();
        // Create a simple connected graph
        graph.add_edge(1, 2, None);
        graph.add_edge(2, 3, None);
        graph.add_edge(3, 4, None);
        graph.add_edge(4, 1, None); // Make it a cycle
        
        let centrality = harmonic_centrality(&graph, true).unwrap();
        
        // All nodes should have equal harmonic centrality in a cycle
        assert!((centrality[&1] - centrality[&2]).abs() < 1e-10);
        assert!((centrality[&2] - centrality[&3]).abs() < 1e-10);
        assert!((centrality[&3] - centrality[&4]).abs() < 1e-10);
    }
}