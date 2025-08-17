//! Eigenvector centrality algorithm

use crate::graph::traits::GraphBase;
use crate::errors::{NetworkXError, Result};
use std::collections::HashMap;
use std::hash::Hash;

/// Calculate eigenvector centrality for all nodes in the graph
///
/// Eigenvector centrality assigns relative scores to all nodes in the network based on
/// the concept that connections to high-scoring nodes contribute more to the score of
/// the node in question than equal connections to low-scoring nodes.
pub fn eigenvector_centrality<G, N>(
    graph: &G,
    max_iter: usize,
    tolerance: f64,
    normalized: bool,
) -> Result<HashMap<N, f64>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let nodes: Vec<N> = graph.nodes().collect();
    let n = nodes.len();
    
    if n == 0 {
        return Ok(HashMap::new());
    }
    
    // Initialize eigenvector with equal values
    let mut x: HashMap<N, f64> = HashMap::new();
    let init_value = 1.0 / (n as f64).sqrt();
    for node in &nodes {
        x.insert(node.clone(), init_value);
    }
    
    // Power iteration method
    for iteration in 0..max_iter {
        let x_prev = x.clone();
        let mut x_new: HashMap<N, f64> = HashMap::new();
        
        // Calculate new values
        for node in &nodes {
            let mut sum = 0.0;
            for neighbor in graph.neighbors(node) {
                let weight = graph.get_edge_weight(&neighbor, node).unwrap_or(1.0);
                sum += x_prev[&neighbor] * weight;
            }
            x_new.insert(node.clone(), sum);
        }
        
        // Normalize the vector
        let norm: f64 = x_new.values().map(|v| v * v).sum::<f64>().sqrt();
        if norm == 0.0 {
            return Err(NetworkXError::ComputationError(
                "Eigenvector centrality failed: zero norm".to_string()
            ));
        }
        
        for value in x_new.values_mut() {
            *value /= norm;
        }
        
        // Check for convergence
        let mut converged = true;
        for node in &nodes {
            if (x_new[node] - x_prev[node]).abs() > tolerance {
                converged = false;
                break;
            }
        }
        
        x = x_new;
        
        if converged {
            break;
        }
        
        if iteration == max_iter - 1 {
            return Err(NetworkXError::ComputationError(
                format!("Eigenvector centrality failed to converge after {} iterations", max_iter)
            ));
        }
    }
    
    // Normalize to [0, 1] range if requested
    if normalized && !x.is_empty() {
        let max_value = x.values().cloned().fold(0.0, f64::max);
        if max_value > 0.0 {
            for value in x.values_mut() {
                *value /= max_value;
            }
        }
    }
    
    Ok(x)
}

/// Calculate Katz centrality for all nodes in the graph
///
/// Katz centrality is a generalization of eigenvector centrality that can handle
/// directed graphs and includes a damping parameter alpha.
pub fn katz_centrality<G, N>(
    graph: &G,
    alpha: f64,
    beta: f64,
    max_iter: usize,
    tolerance: f64,
    normalized: bool,
) -> Result<HashMap<N, f64>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let nodes: Vec<N> = graph.nodes().collect();
    let n = nodes.len();
    
    if n == 0 {
        return Ok(HashMap::new());
    }
    
    // Check that alpha is valid (should be less than 1/lambda_max)
    // For simplicity, we use a conservative bound
    let max_degree = nodes.iter()
        .map(|node| graph.degree(node))
        .max()
        .unwrap_or(0) as f64;
    
    if alpha * max_degree >= 1.0 {
        return Err(NetworkXError::InvalidInput(
            format!("Alpha ({}) is too large for this graph (max degree: {})", alpha, max_degree)
        ));
    }
    
    // Initialize centrality values
    let mut x: HashMap<N, f64> = HashMap::new();
    for node in &nodes {
        x.insert(node.clone(), 0.0);
    }
    
    // Power iteration with personalization vector beta
    for iteration in 0..max_iter {
        let x_prev = x.clone();
        
        for node in &nodes {
            let mut sum = beta;
            for neighbor in graph.neighbors(node) {
                let weight = graph.get_edge_weight(&neighbor, node).unwrap_or(1.0);
                sum += alpha * x_prev[&neighbor] * weight;
            }
            x.insert(node.clone(), sum);
        }
        
        // Check for convergence
        let mut converged = true;
        for node in &nodes {
            if (x[node] - x_prev[node]).abs() > tolerance {
                converged = false;
                break;
            }
        }
        
        if converged {
            break;
        }
        
        if iteration == max_iter - 1 {
            return Err(NetworkXError::ComputationError(
                format!("Katz centrality failed to converge after {} iterations", max_iter)
            ));
        }
    }
    
    // Normalize if requested
    if normalized && !x.is_empty() {
        let norm: f64 = x.values().map(|v| v * v).sum::<f64>().sqrt();
        if norm > 0.0 {
            for value in x.values_mut() {
                *value /= norm;
            }
        }
    }
    
    Ok(x)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::Graph;
    
    #[test]
    fn test_eigenvector_centrality_star() {
        let mut graph = Graph::new();
        // Create a star graph with center at node 0
        for i in 1..5 {
            graph.add_edge(0, i, None);
        }
        
        let centrality = eigenvector_centrality(&graph, 100, 1e-6, true).unwrap();
        
        // Center node should have maximum centrality
        assert_eq!(centrality[&0], 1.0); // Normalized, so max is 1.0
        
        // All leaf nodes should have equal centrality
        for i in 2..5 {
            assert!((centrality[&1] - centrality[&i]).abs() < 1e-6);
        }
    }
    
    #[test]
    fn test_eigenvector_centrality_path() {
        let mut graph = Graph::new();
        // Create a path graph: 1-2-3-4
        graph.add_edge(1, 2, None);
        graph.add_edge(2, 3, None);
        graph.add_edge(3, 4, None);
        
        let centrality = eigenvector_centrality(&graph, 100, 1e-6, false).unwrap();
        
        // Middle nodes should have higher centrality
        assert!(centrality[&2] > centrality[&1]);
        assert!(centrality[&3] > centrality[&4]);
    }
    
    #[test]
    fn test_katz_centrality() {
        let mut graph = Graph::new();
        // Create a simple cycle
        graph.add_edge(1, 2, None);
        graph.add_edge(2, 3, None);
        graph.add_edge(3, 4, None);
        graph.add_edge(4, 1, None);
        
        let centrality = katz_centrality(&graph, 0.1, 1.0, 100, 1e-6, false).unwrap();
        
        // In a cycle, all nodes should have equal Katz centrality
        let values: Vec<f64> = centrality.values().cloned().collect();
        for i in 1..values.len() {
            assert!((values[0] - values[i]).abs() < 1e-6);
        }
    }
}