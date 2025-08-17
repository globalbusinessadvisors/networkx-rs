//! Scale-free graph generators

use crate::graph::Graph;
use crate::graph::traits::{GraphBase, GraphMut};
use crate::errors::{NetworkXError, Result};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::collections::HashSet;

/// Generate a Barabási-Albert preferential attachment graph
///
/// A graph of n nodes is grown by attaching new nodes each with m edges
/// that are preferentially attached to existing nodes with high degree.
pub fn barabasi_albert(
    n: usize,
    m: usize,
    seed: Option<u64>,
) -> Result<Graph<usize>> {
    if m < 1 {
        return Err(NetworkXError::InvalidInput(
            "Number of edges to attach (m) must be at least 1".to_string()
        ));
    }
    
    if m >= n {
        return Err(NetworkXError::InvalidInput(
            format!("Number of edges to attach ({}) must be less than number of nodes ({})", m, n)
        ));
    }
    
    let mut rng = if let Some(s) = seed {
        ChaCha8Rng::seed_from_u64(s)
    } else {
        ChaCha8Rng::from_entropy()
    };
    
    let mut graph = Graph::new();
    
    // Start with a complete graph on m+1 nodes
    for i in 0..=m {
        graph.add_node(i);
    }
    
    for i in 0..=m {
        for j in (i + 1)..=m {
            graph.add_edge(i, j, None);
        }
    }
    
    // Keep track of node degrees for preferential attachment
    let mut repeated_nodes: Vec<usize> = Vec::new();
    for i in 0..=m {
        for _ in 0..m {
            repeated_nodes.push(i);
        }
    }
    
    // Add remaining nodes
    for source in (m + 1)..n {
        graph.add_node(source);
        
        // Select m unique targets with preferential attachment
        let mut targets = HashSet::new();
        while targets.len() < m {
            let idx = rng.gen_range(0..repeated_nodes.len());
            targets.insert(repeated_nodes[idx]);
        }
        
        // Add edges and update repeated_nodes
        for target in targets {
            graph.add_edge(source, target, None);
            repeated_nodes.push(source);
            repeated_nodes.push(target);
        }
    }
    
    Ok(graph)
}

/// Generate an extended Barabási-Albert graph
///
/// This variant allows for different attachment probabilities and
/// includes options for adding edges to existing nodes.
pub fn extended_barabasi_albert(
    n: usize,
    m: usize,
    p: f64,
    q: f64,
    seed: Option<u64>,
) -> Result<Graph<usize>> {
    if m < 1 {
        return Err(NetworkXError::InvalidInput(
            "Number of edges to attach (m) must be at least 1".to_string()
        ));
    }
    
    if p < 0.0 || p > 1.0 || q < 0.0 || q > 1.0 {
        return Err(NetworkXError::InvalidInput(
            "Probabilities p and q must be in [0,1]".to_string()
        ));
    }
    
    if p + q > 1.0 {
        return Err(NetworkXError::InvalidInput(
            "Sum of probabilities p + q must be at most 1".to_string()
        ));
    }
    
    let mut rng = if let Some(s) = seed {
        ChaCha8Rng::seed_from_u64(s)
    } else {
        ChaCha8Rng::from_entropy()
    };
    
    let mut graph = Graph::new();
    
    // Start with a single node
    graph.add_node(0);
    let mut node_count = 1;
    let mut repeated_nodes = vec![0];
    
    while node_count < n {
        let r = rng.gen::<f64>();
        
        if r < p {
            // Add m new edges to existing nodes
            if repeated_nodes.len() >= m {
                let mut targets = HashSet::new();
                while targets.len() < m {
                    let idx = rng.gen_range(0..repeated_nodes.len());
                    targets.insert(repeated_nodes[idx]);
                }
                
                for target in &targets {
                    if repeated_nodes.len() >= 2 {
                        let source_idx = rng.gen_range(0..repeated_nodes.len());
                        let source = repeated_nodes[source_idx];
                        if source != *target && !GraphBase::has_edge(&graph, &source, target) {
                            graph.add_edge(source, *target, None);
                            repeated_nodes.push(source);
                            repeated_nodes.push(*target);
                        }
                    }
                }
            }
        } else if r < p + q {
            // Rewire m edges
            let edges: Vec<_> = GraphBase::edges(&graph).collect();
            if edges.len() >= m {
                for _ in 0..m.min(edges.len()) {
                    let edge_idx = rng.gen_range(0..edges.len());
                    let (u, v, _) = &edges[edge_idx];
                    
                    // Remove old edge
                    GraphMut::remove_edge(&mut graph, u, v);
                    
                    // Add new edge with preferential attachment
                    if !repeated_nodes.is_empty() {
                        let new_target_idx = rng.gen_range(0..repeated_nodes.len());
                        let new_target = repeated_nodes[new_target_idx];
                        if !GraphBase::has_edge(&graph, u, &new_target) && *u != new_target {
                            graph.add_edge(u.clone(), new_target, None);
                        }
                    }
                }
            }
        } else {
            // Add new node with m edges (standard BA)
            graph.add_node(node_count);
            
            if repeated_nodes.len() >= m {
                let mut targets = HashSet::new();
                while targets.len() < m {
                    let idx = rng.gen_range(0..repeated_nodes.len());
                    targets.insert(repeated_nodes[idx]);
                }
                
                for target in targets {
                    graph.add_edge(node_count, target, None);
                    repeated_nodes.push(node_count);
                    repeated_nodes.push(target);
                }
            } else {
                // Connect to all existing nodes if fewer than m
                for i in 0..node_count {
                    graph.add_edge(node_count, i, None);
                    repeated_nodes.push(node_count);
                    repeated_nodes.push(i);
                }
            }
            
            node_count += 1;
        }
    }
    
    Ok(graph)
}

/// Generate a powerlaw cluster graph
///
/// This graph model combines preferential attachment with triangle formation.
pub fn powerlaw_cluster(
    n: usize,
    m: usize,
    p: f64,
    seed: Option<u64>,
) -> Result<Graph<usize>> {
    if m < 1 {
        return Err(NetworkXError::InvalidInput(
            "Number of edges to attach (m) must be at least 1".to_string()
        ));
    }
    
    if p < 0.0 || p > 1.0 {
        return Err(NetworkXError::InvalidInput(
            format!("Triangle formation probability must be in [0,1], got {}", p)
        ));
    }
    
    let mut rng = if let Some(s) = seed {
        ChaCha8Rng::seed_from_u64(s)
    } else {
        ChaCha8Rng::from_entropy()
    };
    
    let mut graph = Graph::new();
    
    // Start with a complete graph on m+1 nodes
    for i in 0..=m {
        graph.add_node(i);
    }
    
    for i in 0..=m {
        for j in (i + 1)..=m {
            graph.add_edge(i, j, None);
        }
    }
    
    // Keep track of node degrees for preferential attachment
    let mut repeated_nodes: Vec<usize> = Vec::new();
    for i in 0..=m {
        for _ in 0..m {
            repeated_nodes.push(i);
        }
    }
    
    // Add remaining nodes
    for source in (m + 1)..n {
        graph.add_node(source);
        
        // Select m unique targets with preferential attachment
        let mut targets: Vec<usize> = Vec::new();
        let mut target_set = HashSet::new();
        
        while targets.len() < m && !repeated_nodes.is_empty() {
            let idx = rng.gen_range(0..repeated_nodes.len());
            let target = repeated_nodes[idx];
            if target_set.insert(target) {
                targets.push(target);
            }
        }
        
        // Add edges
        for &target in &targets {
            graph.add_edge(source, target, None);
            repeated_nodes.push(source);
            repeated_nodes.push(target);
        }
        
        // Triangle formation step
        if p > 0.0 && targets.len() >= 2 {
            for i in 0..targets.len() {
                for j in (i + 1)..targets.len() {
                    if rng.gen::<f64>() < p && !GraphBase::has_edge(&graph, &targets[i], &targets[j]) {
                        graph.add_edge(targets[i], targets[j], None);
                        repeated_nodes.push(targets[i]);
                        repeated_nodes.push(targets[j]);
                    }
                }
            }
        }
    }
    
    Ok(graph)
}

/// Generate a dual Barabási-Albert graph
///
/// This creates two preferential attachment processes.
pub fn dual_barabasi_albert(
    n: usize,
    m1: usize,
    m2: usize,
    p: f64,
    seed: Option<u64>,
) -> Result<Graph<usize>> {
    if m1 < 1 || m2 < 1 {
        return Err(NetworkXError::InvalidInput(
            "Number of edges to attach must be at least 1".to_string()
        ));
    }
    
    if p < 0.0 || p > 1.0 {
        return Err(NetworkXError::InvalidInput(
            format!("Probability must be in [0,1], got {}", p)
        ));
    }
    
    let mut rng = if let Some(s) = seed {
        ChaCha8Rng::seed_from_u64(s)
    } else {
        ChaCha8Rng::from_entropy()
    };
    
    // Start with first BA process
    let mut graph = barabasi_albert(n / 2, m1, seed)?;
    
    // Second BA process
    let offset = n / 2;
    let mut repeated_nodes: Vec<usize> = Vec::new();
    
    // Initialize with existing nodes
    for node in 0..(n / 2) {
        for _ in 0..graph.degree(&node) {
            repeated_nodes.push(node);
        }
    }
    
    // Add remaining nodes with choice between m1 and m2
    for source in offset..n {
        graph.add_node(source);
        
        let m = if rng.gen::<f64>() < p { m1 } else { m2 };
        
        // Select targets
        let mut targets = HashSet::new();
        while targets.len() < m && !repeated_nodes.is_empty() {
            let idx = rng.gen_range(0..repeated_nodes.len());
            targets.insert(repeated_nodes[idx]);
        }
        
        // Add edges
        for target in targets {
            graph.add_edge(source, target, None);
            repeated_nodes.push(source);
            repeated_nodes.push(target);
        }
    }
    
    Ok(graph)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::traits::GraphBase;
    
    #[test]
    fn test_barabasi_albert() {
        let graph = barabasi_albert(20, 3, Some(42)).unwrap();
        assert_eq!(graph.node_count(), 20);
        
        // Check that later nodes tend to have degree at least m
        for node in 4..20 {
            assert!(graph.degree(&node) >= 3);
        }
        
        // Check total edges
        let expected_edges = 3 * 3 / 2 + 3 * (20 - 4); // Initial complete graph + new edges
        assert_eq!(graph.edge_count(), expected_edges);
    }
    
    #[test]
    fn test_barabasi_albert_invalid() {
        assert!(barabasi_albert(5, 0, None).is_err());
        assert!(barabasi_albert(5, 10, None).is_err());
    }
    
    #[test]
    fn test_powerlaw_cluster() {
        let graph = powerlaw_cluster(15, 3, 0.5, Some(42)).unwrap();
        assert_eq!(graph.node_count(), 15);
        
        // Graph should have more edges than basic BA due to triangle formation
        let ba_graph = barabasi_albert(15, 3, Some(42)).unwrap();
        assert!(graph.edge_count() >= ba_graph.edge_count());
    }
    
    #[test]
    fn test_extended_barabasi_albert() {
        let graph = extended_barabasi_albert(20, 2, 0.2, 0.2, Some(42)).unwrap();
        assert_eq!(graph.node_count(), 20);
        
        // Should have edges
        assert!(graph.edge_count() > 0);
    }
}