//! Random graph generators

use crate::graph::Graph;
use crate::graph::traits::{GraphBase, GraphMut};
use crate::errors::{NetworkXError, Result};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::collections::HashSet;

/// Generate an Erdős-Rényi random graph G(n,p)
///
/// The G(n,p) model chooses each possible edge with probability p.
pub fn erdos_renyi(
    n: usize,
    p: f64,
    directed: bool,
    seed: Option<u64>,
) -> Result<Graph<usize>> {
    if n == 0 {
        return Ok(Graph::new());
    }
    
    if p < 0.0 || p > 1.0 {
        return Err(NetworkXError::InvalidInput(
            format!("Probability p must be in [0,1], got {}", p)
        ));
    }
    
    let mut rng = if let Some(s) = seed {
        ChaCha8Rng::seed_from_u64(s)
    } else {
        ChaCha8Rng::from_entropy()
    };
    
    let mut graph = Graph::new();
    
    // Add all nodes
    for i in 0..n {
        graph.add_node(i);
    }
    
    // Add edges with probability p
    if directed {
        for i in 0..n {
            for j in 0..n {
                if i != j && rng.gen::<f64>() < p {
                    graph.add_edge(i, j, None);
                }
            }
        }
    } else {
        for i in 0..n {
            for j in (i + 1)..n {
                if rng.gen::<f64>() < p {
                    graph.add_edge(i, j, None);
                }
            }
        }
    }
    
    Ok(graph)
}

/// Fast algorithm for generating random graphs G(n,p)
///
/// This is a faster O(n+m) expected time algorithm for sparse graphs.
pub fn fast_gnp_random_graph(
    n: usize,
    p: f64,
    directed: bool,
    seed: Option<u64>,
) -> Result<Graph<usize>> {
    if n == 0 {
        return Ok(Graph::new());
    }
    
    if p < 0.0 || p > 1.0 {
        return Err(NetworkXError::InvalidInput(
            format!("Probability p must be in [0,1], got {}", p)
        ));
    }
    
    let mut rng = if let Some(s) = seed {
        ChaCha8Rng::seed_from_u64(s)
    } else {
        ChaCha8Rng::from_entropy()
    };
    
    let mut graph = Graph::new();
    
    // Add all nodes
    for i in 0..n {
        graph.add_node(i);
    }
    
    // Handle edge cases
    if p == 0.0 {
        return Ok(graph);
    }
    if p == 1.0 {
        return complete_graph_internal(n, directed);
    }
    
    // Use fast algorithm for sparse graphs
    let lp = (1.0 - p).ln();
    
    if directed {
        for v in 0..n {
            let mut w = 0;
            while w < n {
                let lr = (1.0 - rng.gen::<f64>()).ln();
                w = w + 1 + ((lr / lp).floor() as usize);
                if w < n && v != w {
                    graph.add_edge(v, w, None);
                }
            }
        }
    } else {
        let mut v = 0;
        let mut w = 0;
        while v < n {
            let lr = (1.0 - rng.gen::<f64>()).ln();
            w = w + 1 + ((lr / lp).floor() as usize);
            while w >= n && v < n - 1 {
                v += 1;
                w = w - n + v + 1;
            }
            if v < n && w < n {
                graph.add_edge(v, w, None);
            }
        }
    }
    
    Ok(graph)
}

/// Generate a random graph G(n,m) with exactly m edges
///
/// The G(n,m) model chooses m edges uniformly at random from all possible edges.
pub fn gnm_random_graph(
    n: usize,
    m: usize,
    directed: bool,
    seed: Option<u64>,
) -> Result<Graph<usize>> {
    if n == 0 {
        return Ok(Graph::new());
    }
    
    let max_edges = if directed {
        n * (n - 1)
    } else {
        n * (n - 1) / 2
    };
    
    if m > max_edges {
        return Err(NetworkXError::InvalidInput(
            format!("Cannot create graph with {} nodes and {} edges (max: {})", n, m, max_edges)
        ));
    }
    
    let mut rng = if let Some(s) = seed {
        ChaCha8Rng::seed_from_u64(s)
    } else {
        ChaCha8Rng::from_entropy()
    };
    
    let mut graph = Graph::new();
    
    // Add all nodes
    for i in 0..n {
        graph.add_node(i);
    }
    
    if m == 0 {
        return Ok(graph);
    }
    
    // For dense graphs, start with complete graph and remove edges
    if m > max_edges / 2 {
        graph = complete_graph_internal(n, directed)?;
        let edges_to_remove = max_edges - m;
        let mut removed = 0;
        
        while removed < edges_to_remove {
            let u = rng.gen_range(0..n);
            let v = rng.gen_range(0..n);
            if u != v && GraphBase::has_edge(&graph, &u, &v) {
                GraphMut::remove_edge(&mut graph, &u, &v);
                removed += 1;
            }
        }
    } else {
        // For sparse graphs, add edges randomly
        let mut edges_added = HashSet::new();
        
        while edges_added.len() < m {
            let u = rng.gen_range(0..n);
            let v = rng.gen_range(0..n);
            
            if u != v {
                let edge = if !directed && u > v {
                    (v, u)
                } else {
                    (u, v)
                };
                
                if edges_added.insert(edge) {
                    GraphMut::add_edge(&mut graph, edge.0, edge.1, None);
                }
            }
        }
    }
    
    Ok(graph)
}

/// Generate a random regular graph
///
/// A regular graph is a graph where each vertex has the same degree.
pub fn random_regular_graph(
    d: usize,
    n: usize,
    seed: Option<u64>,
) -> Result<Graph<usize>> {
    if d >= n {
        return Err(NetworkXError::InvalidInput(
            format!("Degree {} must be less than number of nodes {}", d, n)
        ));
    }
    
    if (d * n) % 2 != 0 {
        return Err(NetworkXError::InvalidInput(
            "The product d*n must be even".to_string()
        ));
    }
    
    let mut rng = if let Some(s) = seed {
        ChaCha8Rng::seed_from_u64(s)
    } else {
        ChaCha8Rng::from_entropy()
    };
    
    // Create stubs (half-edges)
    let mut stubs: Vec<usize> = Vec::new();
    for node in 0..n {
        for _ in 0..d {
            stubs.push(node);
        }
    }
    
    // Repeatedly shuffle and check if valid
    let max_attempts = 100;
    for _ in 0..max_attempts {
        // Shuffle stubs
        for i in (1..stubs.len()).rev() {
            let j = rng.gen_range(0..=i);
            stubs.swap(i, j);
        }
        
        // Try to create graph
        let mut graph = Graph::new();
        for i in 0..n {
            graph.add_node(i);
        }
        
        let mut valid = true;
        for i in (0..stubs.len()).step_by(2) {
            let u = stubs[i];
            let v = stubs[i + 1];
            
            if u == v || graph.has_edge(&u, &v) {
                valid = false;
                break;
            }
            
            graph.add_edge(u, v, None);
        }
        
        if valid {
            return Ok(graph);
        }
    }
    
    Err(NetworkXError::ComputationError(
        "Failed to generate regular graph after maximum attempts".to_string()
    ))
}

/// Internal helper to create a complete graph
fn complete_graph_internal(n: usize, directed: bool) -> Result<Graph<usize>> {
    let mut graph = Graph::new();
    
    for i in 0..n {
        graph.add_node(i);
    }
    
    if directed {
        for i in 0..n {
            for j in 0..n {
                if i != j {
                    graph.add_edge(i, j, None);
                }
            }
        }
    } else {
        for i in 0..n {
            for j in (i + 1)..n {
                graph.add_edge(i, j, None);
            }
        }
    }
    
    Ok(graph)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::traits::GraphBase;
    
    #[test]
    fn test_erdos_renyi_empty() {
        let graph = erdos_renyi(10, 0.0, false, Some(42)).unwrap();
        assert_eq!(graph.node_count(), 10);
        assert_eq!(graph.edge_count(), 0);
    }
    
    #[test]
    fn test_erdos_renyi_complete() {
        let graph = erdos_renyi(5, 1.0, false, Some(42)).unwrap();
        assert_eq!(graph.node_count(), 5);
        assert_eq!(graph.edge_count(), 10); // 5 * 4 / 2
    }
    
    #[test]
    fn test_fast_gnp() {
        let graph = fast_gnp_random_graph(20, 0.1, false, Some(42)).unwrap();
        assert_eq!(graph.node_count(), 20);
        // With p=0.1, expect roughly 10% of possible edges
        let expected = (20 * 19 / 2) as f64 * 0.1;
        let actual = graph.edge_count() as f64;
        assert!((actual - expected).abs() < expected * 0.5); // Within 50% of expected
    }
    
    #[test]
    fn test_gnm_random() {
        let graph = gnm_random_graph(10, 15, false, Some(42)).unwrap();
        assert_eq!(graph.node_count(), 10);
        assert_eq!(graph.edge_count(), 15);
    }
    
    #[test]
    fn test_gnm_invalid() {
        let result = gnm_random_graph(5, 100, false, Some(42));
        assert!(result.is_err());
    }
    
    #[test]
    fn test_random_regular() {
        let graph = random_regular_graph(3, 10, Some(42)).unwrap();
        assert_eq!(graph.node_count(), 10);
        
        // Check that all nodes have degree 3
        for node in 0..10 {
            assert_eq!(graph.degree(&node), 3);
        }
    }
}