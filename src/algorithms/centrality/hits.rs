//! HITS (Hyperlink-Induced Topic Search) algorithm for authority and hub scores

use crate::graph::traits::GraphBase;
use crate::errors::Result;
use std::collections::HashMap;
use std::hash::Hash;

/// Result of HITS algorithm containing both hub and authority scores
#[derive(Debug, Clone)]
pub struct HITSResult<N> {
    /// Hub scores for each node
    pub hubs: HashMap<N, f64>,
    /// Authority scores for each node
    pub authorities: HashMap<N, f64>,
}

/// Compute HITS (Hyperlink-Induced Topic Search) scores
/// 
/// The HITS algorithm assigns two scores to each node:
/// - Hub score: measures how many good authorities a node points to
/// - Authority score: measures how many good hubs point to a node
/// 
/// These scores are computed iteratively until convergence.
pub fn hits<G, N>(
    graph: &G,
    max_iter: Option<usize>,
    tolerance: Option<f64>,
    normalize: Option<bool>,
) -> Result<HITSResult<N>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let max_iter = max_iter.unwrap_or(100);
    let tolerance = tolerance.unwrap_or(1e-8);
    let normalize = normalize.unwrap_or(true);
    
    let nodes: Vec<N> = graph.nodes().collect();
    let n = nodes.len();
    
    if n == 0 {
        return Ok(HITSResult {
            hubs: HashMap::new(),
            authorities: HashMap::new(),
        });
    }
    
    // Initialize all scores to 1.0
    let mut hubs: HashMap<N, f64> = nodes.iter()
        .map(|n| (n.clone(), 1.0))
        .collect();
    let mut authorities: HashMap<N, f64> = nodes.iter()
        .map(|n| (n.clone(), 1.0))
        .collect();
    
    for _ in 0..max_iter {
        let old_hubs = hubs.clone();
        let old_authorities = authorities.clone();
        
        // Update authority scores: authority(v) = sum of hub scores of nodes pointing to v
        for node in &nodes {
            let mut auth_score = 0.0;
            for neighbor in graph.neighbors(node) {
                if graph.has_edge(&neighbor, node) {
                    auth_score += old_hubs.get(&neighbor).copied().unwrap_or(0.0);
                }
            }
            authorities.insert(node.clone(), auth_score);
        }
        
        // Update hub scores: hub(v) = sum of authority scores of nodes v points to
        for node in &nodes {
            let mut hub_score = 0.0;
            for neighbor in graph.neighbors(node) {
                if graph.has_edge(node, &neighbor) {
                    hub_score += authorities.get(&neighbor).copied().unwrap_or(0.0);
                }
            }
            hubs.insert(node.clone(), hub_score);
        }
        
        // Normalize scores
        if normalize {
            let hub_norm = (hubs.values().map(|x| x * x).sum::<f64>()).sqrt();
            let auth_norm = (authorities.values().map(|x| x * x).sum::<f64>()).sqrt();
            
            if hub_norm > 0.0 {
                for value in hubs.values_mut() {
                    *value /= hub_norm;
                }
            }
            
            if auth_norm > 0.0 {
                for value in authorities.values_mut() {
                    *value /= auth_norm;
                }
            }
        }
        
        // Check for convergence
        let hub_change: f64 = hubs.iter()
            .map(|(k, v)| (v - old_hubs.get(k).unwrap_or(&0.0)).abs())
            .fold(0.0, f64::max);
            
        let auth_change: f64 = authorities.iter()
            .map(|(k, v)| (v - old_authorities.get(k).unwrap_or(&0.0)).abs())
            .fold(0.0, f64::max);
        
        if hub_change < tolerance && auth_change < tolerance {
            break;
        }
    }
    
    Ok(HITSResult {
        hubs,
        authorities,
    })
}

/// Get only the hub scores from HITS algorithm
pub fn hub_scores<G, N>(
    graph: &G,
    max_iter: Option<usize>,
    tolerance: Option<f64>,
    normalize: Option<bool>,
) -> Result<HashMap<N, f64>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let result = hits(graph, max_iter, tolerance, normalize)?;
    Ok(result.hubs)
}

/// Get only the authority scores from HITS algorithm
pub fn authority_scores<G, N>(
    graph: &G,
    max_iter: Option<usize>,
    tolerance: Option<f64>,
    normalize: Option<bool>,
) -> Result<HashMap<N, f64>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let result = hits(graph, max_iter, tolerance, normalize)?;
    Ok(result.authorities)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::Graph;

    #[test]
    fn test_hits_simple() {
        let mut graph = Graph::new();
        graph.add_edge(1, 2, Some(1.0));
        graph.add_edge(1, 3, Some(1.0));
        graph.add_edge(2, 3, Some(1.0));
        
        let result = hits(&graph, Some(100), Some(1e-8), Some(true)).unwrap();
        
        // Basic sanity checks
        assert_eq!(result.hubs.len(), 3);
        assert_eq!(result.authorities.len(), 3);
        assert!(result.hubs.values().all(|&v| v >= 0.0));
        assert!(result.authorities.values().all(|&v| v >= 0.0));
    }
    
    #[test]
    fn test_hits_normalized() {
        let mut graph = Graph::new();
        graph.add_edge(1, 2, Some(1.0));
        graph.add_edge(2, 3, Some(1.0));
        
        let result = hits(&graph, Some(100), Some(1e-8), Some(true)).unwrap();
        
        // Check normalization
        let hub_norm: f64 = result.hubs.values().map(|x| x * x).sum::<f64>().sqrt();
        let auth_norm: f64 = result.authorities.values().map(|x| x * x).sum::<f64>().sqrt();
        
        assert!((hub_norm - 1.0).abs() < 1e-10);
        assert!((auth_norm - 1.0).abs() < 1e-10);
    }
    
    #[test]
    fn test_hub_and_authority_scores() {
        let mut graph = Graph::new();
        graph.add_edge(1, 2, Some(1.0));
        graph.add_edge(2, 3, Some(1.0));
        
        let hubs = hub_scores(&graph, Some(100), Some(1e-8), Some(true)).unwrap();
        let authorities = authority_scores(&graph, Some(100), Some(1e-8), Some(true)).unwrap();
        
        assert_eq!(hubs.len(), 3);
        assert_eq!(authorities.len(), 3);
    }
}