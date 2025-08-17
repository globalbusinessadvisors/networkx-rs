//! Betweenness centrality algorithm

use crate::graph::traits::GraphBase;
use crate::errors::Result;
use std::collections::{HashMap, VecDeque};
use std::hash::Hash;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

/// Calculate betweenness centrality for all nodes in the graph
///
/// Betweenness centrality is a measure of centrality based on shortest paths.
/// For every pair of vertices in a graph, there exists at least one shortest path
/// between the vertices such that the sum of the weights of the edges is minimized.
/// The betweenness centrality for each vertex is the number of these shortest paths
/// that pass through the vertex.
pub fn betweenness_centrality<G, N>(
    graph: &G,
    normalized: bool,
    endpoints: bool,
) -> Result<HashMap<N, f64>>
where
    G: GraphBase<NodeId = N> + crate::graph::traits::GraphAlgorithms + Sync,
    N: Clone + Hash + Eq + Send + Sync,
{
    let mut centrality: HashMap<N, f64> = HashMap::new();
    let nodes: Vec<N> = graph.nodes().collect();
    
    // Initialize centrality scores to 0
    for node in &nodes {
        centrality.insert(node.clone(), 0.0);
    }
    
    // Parallel or sequential processing based on feature flag
    #[cfg(feature = "parallel")]
    let results: Vec<HashMap<N, f64>> = nodes
        .par_iter()
        .map(|source| single_source_shortest_path_basic(graph, source.clone(), endpoints))
        .collect();
    
    #[cfg(not(feature = "parallel"))]
    let results: Vec<HashMap<N, f64>> = nodes
        .iter()
        .map(|source| single_source_shortest_path_basic(graph, source.clone(), endpoints))
        .collect();
    
    // Aggregate results
    for partial in results {
        for (node, value) in partial {
            *centrality.entry(node).or_insert(0.0) += value;
        }
    }
    
    // Normalize if requested
    if normalized {
        let n = nodes.len() as f64;
        let scale = if n > 2.0 {
            1.0 / ((n - 1.0) * (n - 2.0))
        } else {
            1.0
        };
        
        // Account for directed vs undirected graphs
        let scale = if graph.is_directed() {
            scale
        } else {
            scale * 2.0
        };
        
        for value in centrality.values_mut() {
            *value *= scale;
        }
    }
    
    Ok(centrality)
}

/// Compute betweenness centrality contributions using Brandes' algorithm
fn single_source_shortest_path_basic<G, N>(
    graph: &G,
    source: N,
    endpoints: bool,
) -> HashMap<N, f64>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let mut centrality: HashMap<N, f64> = HashMap::new();
    let mut stack: Vec<N> = Vec::new();
    let mut paths: HashMap<N, Vec<N>> = HashMap::new();
    let mut sigma: HashMap<N, f64> = HashMap::new();
    let mut distance: HashMap<N, i32> = HashMap::new();
    let mut delta: HashMap<N, f64> = HashMap::new();
    
    // Initialize
    for node in graph.nodes() {
        paths.insert(node.clone(), Vec::new());
        sigma.insert(node.clone(), 0.0);
        distance.insert(node.clone(), -1);
        delta.insert(node.clone(), 0.0);
    }
    
    sigma.insert(source.clone(), 1.0);
    distance.insert(source.clone(), 0);
    
    let mut queue = VecDeque::new();
    queue.push_back(source.clone());
    
    // BFS to find shortest paths
    while let Some(v) = queue.pop_front() {
        stack.push(v.clone());
        let d_v = distance[&v];
        
        for w in graph.neighbors(&v) {
            // First time we reach w?
            if distance[&w] < 0 {
                queue.push_back(w.clone());
                distance.insert(w.clone(), d_v + 1);
            }
            // Shortest path to w via v?
            if distance[&w] == d_v + 1 {
                sigma.insert(w.clone(), sigma[&w] + sigma[&v]);
                paths.get_mut(&w).unwrap().push(v.clone());
            }
        }
    }
    
    // Accumulation phase
    while let Some(w) = stack.pop() {
        for v in &paths[&w] {
            let coeff = sigma[v] / sigma[&w] * (1.0 + delta[&w]);
            delta.insert(v.clone(), delta[v] + coeff);
        }
        if w != source {
            let value = if endpoints {
                delta[&w]
            } else {
                delta[&w] - 1.0
            };
            *centrality.entry(w).or_insert(0.0) += value;
        }
    }
    
    centrality
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::Graph;
    
    #[test]
    fn test_betweenness_centrality_simple() {
        let mut graph = Graph::new();
        graph.add_edge(1, 2, None);
        graph.add_edge(2, 3, None);
        graph.add_edge(3, 4, None);
        
        let centrality = betweenness_centrality(&graph, false, false).unwrap();
        
        // Nodes 2 and 3 should have higher centrality
        assert!(centrality[&2] > centrality[&1]);
        assert!(centrality[&3] > centrality[&4]);
    }
    
    #[test]
    fn test_betweenness_centrality_star() {
        let mut graph = Graph::new();
        // Create a star graph with center at node 0
        for i in 1..5 {
            graph.add_edge(0, i, None);
        }
        
        let centrality = betweenness_centrality(&graph, false, false).unwrap();
        
        // Center node should have maximum centrality
        for i in 1..5 {
            assert!(centrality[&0] > centrality[&i]);
        }
    }
}