//! Katz centrality algorithm

use crate::graph::traits::GraphBase;
use crate::errors::Result;
use std::collections::HashMap;
use std::hash::Hash;

/// Compute Katz centrality for all nodes in the graph
/// 
/// Katz centrality measures the influence of a node by considering
/// the number of walks of all lengths starting from that node,
/// with longer walks given exponentially decreasing weights.
/// 
/// The centrality is calculated as:
/// x = α * A * x + β
/// where α is the attenuation factor and β is the bias vector
pub fn katz_centrality<G, N>(
    graph: &G,
    alpha: Option<f64>,
    beta: Option<f64>,
    max_iter: Option<usize>,
    tolerance: Option<f64>,
) -> Result<HashMap<N, f64>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let alpha = alpha.unwrap_or(0.1);
    let beta = beta.unwrap_or(1.0);
    let max_iter = max_iter.unwrap_or(1000);
    let tolerance = tolerance.unwrap_or(1e-6);
    
    let nodes: Vec<N> = graph.nodes().collect();
    let n = nodes.len();
    
    if n == 0 {
        return Ok(HashMap::new());
    }
    
    // Initialize centrality values
    let mut centrality: HashMap<N, f64> = nodes.iter()
        .map(|n| (n.clone(), beta))
        .collect();
    
    // Power iteration
    for _ in 0..max_iter {
        let mut new_centrality = HashMap::new();
        let mut max_change: f64 = 0.0;
        
        for node in &nodes {
            let mut sum = 0.0;
            
            // Sum over all predecessors
            for neighbor in graph.neighbors(node) {
                if graph.has_edge(&neighbor, node) {
                    sum += centrality.get(&neighbor).copied().unwrap_or(0.0);
                }
            }
            
            let new_value = alpha * sum + beta;
            let old_value = centrality.get(node).copied().unwrap_or(0.0);
            max_change = max_change.max((new_value - old_value).abs());
            
            new_centrality.insert(node.clone(), new_value);
        }
        
        centrality = new_centrality;
        
        // Check for convergence
        if max_change < tolerance {
            break;
        }
    }
    
    Ok(centrality)
}

/// Compute normalized Katz centrality
pub fn katz_centrality_normalized<G, N>(
    graph: &G,
    alpha: Option<f64>,
    beta: Option<f64>,
    max_iter: Option<usize>,
    tolerance: Option<f64>,
) -> Result<HashMap<N, f64>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let mut centrality = katz_centrality(graph, alpha, beta, max_iter, tolerance)?;
    
    // Normalize by dividing by the sum
    let sum: f64 = centrality.values().sum();
    if sum > 0.0 {
        for value in centrality.values_mut() {
            *value /= sum;
        }
    }
    
    Ok(centrality)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::Graph;

    #[test]
    fn test_katz_centrality_simple() {
        let mut graph = Graph::new();
        graph.add_edge(1, 2, Some(1.0));
        graph.add_edge(2, 3, Some(1.0));
        graph.add_edge(1, 3, Some(1.0));
        
        let centrality = katz_centrality(&graph, Some(0.1), Some(1.0), Some(100), Some(1e-6)).unwrap();
        
        // Basic sanity checks
        assert_eq!(centrality.len(), 3);
        assert!(centrality.values().all(|&v| v > 0.0));
    }
    
    #[test]
    fn test_katz_centrality_normalized() {
        let mut graph = Graph::new();
        graph.add_edge(1, 2, Some(1.0));
        graph.add_edge(2, 3, Some(1.0));
        
        let centrality = katz_centrality_normalized(&graph, Some(0.1), Some(1.0), Some(100), Some(1e-6)).unwrap();
        
        let sum: f64 = centrality.values().sum();
        assert!((sum - 1.0).abs() < 1e-10);
    }
}