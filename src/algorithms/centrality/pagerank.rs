//! PageRank algorithm implementation

use crate::graph::traits::GraphBase;
use crate::errors::{NetworkXError, Result};
use std::collections::HashMap;
use std::hash::Hash;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

/// Calculate PageRank for all nodes in the graph
///
/// PageRank is a link analysis algorithm that assigns a numerical weighting to each
/// element of a hyperlinked set of documents, with the purpose of measuring its
/// relative importance within the set.
pub fn pagerank<G, N>(
    graph: &G,
    alpha: f64,
    personalization: Option<HashMap<N, f64>>,
    max_iter: usize,
    tolerance: f64,
) -> Result<HashMap<N, f64>>
where
    G: GraphBase<NodeId = N> + crate::graph::traits::GraphAlgorithms + Sync,
    N: Clone + Hash + Eq + Send + Sync,
{
    let nodes: Vec<N> = graph.nodes().collect();
    let n = nodes.len();
    
    if n == 0 {
        return Ok(HashMap::new());
    }
    
    // Validate alpha parameter
    if alpha < 0.0 || alpha > 1.0 {
        return Err(NetworkXError::InvalidInput(
            format!("Alpha must be between 0 and 1, got {}", alpha)
        ));
    }
    
    // Initialize PageRank values
    let mut rank: HashMap<N, f64> = HashMap::new();
    let init_value = 1.0 / n as f64;
    for node in &nodes {
        rank.insert(node.clone(), init_value);
    }
    
    // Set up personalization vector
    let personalization = if let Some(p) = personalization {
        // Normalize personalization vector
        let sum: f64 = p.values().sum();
        if sum == 0.0 {
            return Err(NetworkXError::InvalidInput(
                "Personalization vector sum is zero".to_string()
            ));
        }
        let mut normalized = HashMap::new();
        for (node, value) in p {
            normalized.insert(node, value / sum);
        }
        normalized
    } else {
        // Default: uniform distribution
        let mut p = HashMap::new();
        for node in &nodes {
            p.insert(node.clone(), 1.0 / n as f64);
        }
        p
    };
    
    // Calculate out-degree for each node
    let out_degree: HashMap<N, usize> = nodes
        .iter()
        .map(|node| {
            let degree = graph.neighbors(node).count();
            (node.clone(), degree)
        })
        .collect();
    
    // Identify dangling nodes (nodes with no outgoing edges)
    let dangling_nodes: Vec<N> = nodes
        .iter()
        .filter(|node| out_degree[node] == 0)
        .cloned()
        .collect();
    
    // Power iteration
    for iteration in 0..max_iter {
        let rank_prev = rank.clone();
        
        // Calculate dangling weight
        let dangling_weight: f64 = dangling_nodes
            .iter()
            .map(|node| rank_prev[node])
            .sum();
        
        #[cfg(feature = "parallel")]
        let new_ranks: Vec<(N, f64)> = nodes
            .par_iter()
            .map(|node| {
                let new_rank = calculate_node_pagerank(
                    graph,
                    node,
                    &rank_prev,
                    &out_degree,
                    &personalization,
                    alpha,
                    dangling_weight,
                    n as f64,
                );
                (node.clone(), new_rank)
            })
            .collect();
        
        #[cfg(not(feature = "parallel"))]
        let new_ranks: Vec<(N, f64)> = nodes
            .iter()
            .map(|node| {
                let new_rank = calculate_node_pagerank(
                    graph,
                    node,
                    &rank_prev,
                    &out_degree,
                    &personalization,
                    alpha,
                    dangling_weight,
                    n as f64,
                );
                (node.clone(), new_rank)
            })
            .collect();
        
        // Update ranks
        for (node, new_rank) in new_ranks {
            rank.insert(node, new_rank);
        }
        
        // Check for convergence
        let mut converged = true;
        for node in &nodes {
            if (rank[node] - rank_prev[node]).abs() > tolerance {
                converged = false;
                break;
            }
        }
        
        if converged {
            break;
        }
        
        if iteration == max_iter - 1 {
            // Not an error, just didn't converge fully
            // This is common for large graphs
        }
    }
    
    // Ensure ranks sum to 1
    let sum: f64 = rank.values().sum();
    if sum > 0.0 {
        for value in rank.values_mut() {
            *value /= sum;
        }
    }
    
    Ok(rank)
}

/// Calculate PageRank for a single node in one iteration
fn calculate_node_pagerank<G, N>(
    graph: &G,
    node: &N,
    rank_prev: &HashMap<N, f64>,
    out_degree: &HashMap<N, usize>,
    personalization: &HashMap<N, f64>,
    alpha: f64,
    dangling_weight: f64,
    n: f64,
) -> f64
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    // Sum of PageRank from incoming neighbors
    let mut rank_sum = 0.0;
    
    // In directed graphs, we need to find nodes that point to this node
    // For undirected graphs, neighbors are bidirectional
    for other_node in graph.nodes() {
        if graph.has_edge(&other_node, node) {
            let out_deg = out_degree[&other_node];
            if out_deg > 0 {
                let weight = graph.get_edge_weight(&other_node, node).unwrap_or(1.0);
                rank_sum += rank_prev[&other_node] * weight / out_deg as f64;
            }
        }
    }
    
    // PageRank formula with damping factor and personalization
    let personalization_value = personalization.get(node).copied().unwrap_or(1.0 / n);
    
    (1.0 - alpha) * personalization_value +
    alpha * (rank_sum + dangling_weight * personalization_value)
}

/// Calculate HITS (Hyperlink-Induced Topic Search) algorithm
///
/// HITS algorithm computes two numbers for each node: its authority (estimates the value
/// of the content of the page) and its hub value (estimates the value of its links to
/// other pages).
pub fn hits<G, N>(
    graph: &G,
    max_iter: usize,
    tolerance: f64,
    normalized: bool,
) -> Result<(HashMap<N, f64>, HashMap<N, f64>)>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let nodes: Vec<N> = graph.nodes().collect();
    let n = nodes.len();
    
    if n == 0 {
        return Ok((HashMap::new(), HashMap::new()));
    }
    
    // Initialize hub and authority scores
    let mut hubs: HashMap<N, f64> = HashMap::new();
    let mut authorities: HashMap<N, f64> = HashMap::new();
    let init_value = 1.0 / (n as f64).sqrt();
    
    for node in &nodes {
        hubs.insert(node.clone(), init_value);
        authorities.insert(node.clone(), init_value);
    }
    
    // Power iteration
    for iteration in 0..max_iter {
        let hubs_prev = hubs.clone();
        let authorities_prev = authorities.clone();
        
        // Update authority scores
        for node in &nodes {
            let mut auth_sum = 0.0;
            // Sum hub scores of nodes that point to this node
            for other in graph.nodes() {
                if graph.has_edge(&other, node) {
                    let weight = graph.get_edge_weight(&other, node).unwrap_or(1.0);
                    auth_sum += hubs_prev[&other] * weight;
                }
            }
            authorities.insert(node.clone(), auth_sum);
        }
        
        // Update hub scores
        for node in &nodes {
            let mut hub_sum = 0.0;
            // Sum authority scores of nodes this node points to
            for neighbor in graph.neighbors(node) {
                let weight = graph.get_edge_weight(node, &neighbor).unwrap_or(1.0);
                hub_sum += authorities[&neighbor] * weight;
            }
            hubs.insert(node.clone(), hub_sum);
        }
        
        // Normalize
        let auth_norm: f64 = authorities.values().map(|v| v * v).sum::<f64>().sqrt();
        let hub_norm: f64 = hubs.values().map(|v| v * v).sum::<f64>().sqrt();
        
        if auth_norm > 0.0 {
            for value in authorities.values_mut() {
                *value /= auth_norm;
            }
        }
        
        if hub_norm > 0.0 {
            for value in hubs.values_mut() {
                *value /= hub_norm;
            }
        }
        
        // Check for convergence
        let mut converged = true;
        for node in &nodes {
            if (hubs[node] - hubs_prev[node]).abs() > tolerance ||
               (authorities[node] - authorities_prev[node]).abs() > tolerance {
                converged = false;
                break;
            }
        }
        
        if converged {
            break;
        }
    }
    
    // Normalize to [0, 1] range if requested
    if normalized {
        if let Some(max_hub) = hubs.values().cloned().fold(None, |a: Option<f64>, b| {
            Some(a.map_or(b, |a| a.max(b)))
        }) {
            if max_hub > 0.0 {
                for value in hubs.values_mut() {
                    *value /= max_hub;
                }
            }
        }
        
        if let Some(max_auth) = authorities.values().cloned().fold(None, |a: Option<f64>, b| {
            Some(a.map_or(b, |a| a.max(b)))
        }) {
            if max_auth > 0.0 {
                for value in authorities.values_mut() {
                    *value /= max_auth;
                }
            }
        }
    }
    
    Ok((hubs, authorities))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{Graph, DiGraph};
    
    #[test]
    fn test_pagerank_simple() {
        let mut graph = DiGraph::new();
        // Create a simple directed graph
        graph.add_edge(1, 2, None);
        graph.add_edge(2, 3, None);
        graph.add_edge(3, 1, None);
        graph.add_edge(3, 4, None);
        
        let rank = pagerank(&graph, 0.85, None, 100, 1e-6).unwrap();
        
        // Node 3 should have high PageRank (receives from 2, sends to 1 and 4)
        assert!(rank[&3] > rank[&4]);
        
        // Sum should be approximately 1
        let sum: f64 = rank.values().sum();
        assert!((sum - 1.0).abs() < 1e-6);
    }
    
    #[test]
    fn test_pagerank_star() {
        let mut graph = Graph::new();
        // Create a star graph with center at node 0
        for i in 1..6 {
            graph.add_edge(0, i, None);
        }
        
        let rank = pagerank(&graph, 0.85, None, 100, 1e-6).unwrap();
        
        // Center node should have highest PageRank
        for i in 1..6 {
            assert!(rank[&0] > rank[&i]);
        }
        
        // All leaf nodes should have equal PageRank
        for i in 2..6 {
            assert!((rank[&1] - rank[&i]).abs() < 1e-6);
        }
    }
    
    #[test]
    fn test_pagerank_with_personalization() {
        let mut graph = Graph::new();
        graph.add_edge(1, 2, None);
        graph.add_edge(2, 3, None);
        graph.add_edge(3, 4, None);
        
        // Personalize towards node 1
        let mut personalization = HashMap::new();
        personalization.insert(1, 1.0);
        personalization.insert(2, 0.0);
        personalization.insert(3, 0.0);
        personalization.insert(4, 0.0);
        
        let rank = pagerank(&graph, 0.85, Some(personalization), 100, 1e-6).unwrap();
        
        // Node 1 should have highest PageRank due to personalization
        assert!(rank[&1] > rank[&2]);
        assert!(rank[&1] > rank[&3]);
        assert!(rank[&1] > rank[&4]);
    }
    
    #[test]
    fn test_hits_algorithm() {
        let mut graph = DiGraph::new();
        // Create a simple hub-authority structure
        graph.add_edge(1, 2, None);
        graph.add_edge(1, 3, None);
        graph.add_edge(4, 2, None);
        graph.add_edge(4, 3, None);
        
        let (hubs, authorities) = hits(&graph, 100, 1e-6, true).unwrap();
        
        // Nodes 1 and 4 should be good hubs (they point to authorities)
        assert!(hubs[&1] > authorities[&1]);
        assert!(hubs[&4] > authorities[&4]);
        
        // Nodes 2 and 3 should be good authorities (they are pointed to)
        assert!(authorities[&2] > hubs[&2]);
        assert!(authorities[&3] > hubs[&3]);
    }
}