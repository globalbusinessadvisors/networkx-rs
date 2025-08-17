//! Small-world graph generators

use crate::graph::Graph;
use crate::graph::traits::{GraphBase, GraphMut};
use crate::errors::{NetworkXError, Result};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::collections::HashSet;

/// Generate a Watts-Strogatz small-world graph
///
/// The Watts-Strogatz model generates a graph with small-world properties,
/// including short average path lengths and high clustering.
///
/// Parameters:
/// - n: Number of nodes
/// - k: Each node is connected to k nearest neighbors in ring topology
/// - p: Probability of rewiring each edge
pub fn watts_strogatz(
    n: usize,
    k: usize,
    p: f64,
    seed: Option<u64>,
) -> Result<Graph<usize>> {
    if k >= n {
        return Err(NetworkXError::InvalidInput(
            format!("k ({}) must be less than n ({})", k, n)
        ));
    }
    
    if k % 2 != 0 {
        return Err(NetworkXError::InvalidInput(
            "k must be even".to_string()
        ));
    }
    
    if p < 0.0 || p > 1.0 {
        return Err(NetworkXError::InvalidInput(
            format!("Rewiring probability must be in [0,1], got {}", p)
        ));
    }
    
    let mut rng = if let Some(s) = seed {
        ChaCha8Rng::seed_from_u64(s)
    } else {
        ChaCha8Rng::from_entropy()
    };
    
    let mut graph = Graph::new();
    
    // Create nodes
    for i in 0..n {
        graph.add_node(i);
    }
    
    // Create ring lattice with k neighbors
    for i in 0..n {
        for j in 1..=(k / 2) {
            let neighbor = (i + j) % n;
            graph.add_edge(i, neighbor, None);
        }
    }
    
    // Rewiring step
    if p > 0.0 {
        // Collect all edges that could be rewired
        let edges: Vec<(usize, usize)> = GraphBase::edges(&graph)
            .map(|(u, v, _)| (u, v))
            .filter(|(u, v)| u < v) // Avoid duplicates in undirected graph
            .collect();
        
        for (u, v) in edges {
            if rng.gen::<f64>() < p {
                // Choose new target avoiding self-loops and duplicate edges
                let mut new_target = rng.gen_range(0..n);
                let mut attempts = 0;
                
                while (new_target == u || GraphBase::has_edge(&graph, &u, &new_target)) && attempts < 100 {
                    new_target = rng.gen_range(0..n);
                    attempts += 1;
                }
                
                if new_target != u && !GraphBase::has_edge(&graph, &u, &new_target) {
                    // Rewire the edge
                    GraphMut::remove_edge(&mut graph, &u, &v);
                    graph.add_edge(u, new_target, None);
                }
            }
        }
    }
    
    Ok(graph)
}

/// Generate a Newman-Watts-Strogatz small-world graph
///
/// This variant adds new edges instead of rewiring existing ones,
/// preserving the ring structure while adding shortcuts.
pub fn newman_watts_strogatz(
    n: usize,
    k: usize,
    p: f64,
    seed: Option<u64>,
) -> Result<Graph<usize>> {
    if k >= n {
        return Err(NetworkXError::InvalidInput(
            format!("k ({}) must be less than n ({})", k, n)
        ));
    }
    
    if k % 2 != 0 {
        return Err(NetworkXError::InvalidInput(
            "k must be even".to_string()
        ));
    }
    
    if p < 0.0 || p > 1.0 {
        return Err(NetworkXError::InvalidInput(
            format!("Shortcut probability must be in [0,1], got {}", p)
        ));
    }
    
    let mut rng = if let Some(s) = seed {
        ChaCha8Rng::seed_from_u64(s)
    } else {
        ChaCha8Rng::from_entropy()
    };
    
    let mut graph = Graph::new();
    
    // Create nodes
    for i in 0..n {
        graph.add_node(i);
    }
    
    // Create ring lattice with k neighbors
    for i in 0..n {
        for j in 1..=(k / 2) {
            let neighbor = (i + j) % n;
            graph.add_edge(i, neighbor, None);
        }
    }
    
    // Add shortcuts
    if p > 0.0 {
        for u in 0..n {
            for v in (u + 1)..n {
                if !GraphBase::has_edge(&graph, &u, &v) && rng.gen::<f64>() < p {
                    graph.add_edge(u, v, None);
                }
            }
        }
    }
    
    Ok(graph)
}

/// Generate a connected Watts-Strogatz graph
///
/// This variant ensures the graph remains connected after rewiring.
pub fn connected_watts_strogatz(
    n: usize,
    k: usize,
    p: f64,
    max_attempts: usize,
    seed: Option<u64>,
) -> Result<Graph<usize>> {
    if k >= n {
        return Err(NetworkXError::InvalidInput(
            format!("k ({}) must be less than n ({})", k, n)
        ));
    }
    
    if k % 2 != 0 {
        return Err(NetworkXError::InvalidInput(
            "k must be even".to_string()
        ));
    }
    
    if k < 2 {
        return Err(NetworkXError::InvalidInput(
            "k must be at least 2 to ensure connectivity".to_string()
        ));
    }
    
    for attempt in 0..max_attempts {
        let graph = watts_strogatz(n, k, p, seed.map(|s| s + attempt as u64))?;
        
        // Check if graph is connected using BFS
        if is_connected(&graph) {
            return Ok(graph);
        }
    }
    
    Err(NetworkXError::ComputationError(
        format!("Failed to generate connected graph after {} attempts", max_attempts)
    ))
}

/// Check if a graph is connected
fn is_connected(graph: &Graph<usize>) -> bool {
    let nodes: Vec<usize> = GraphBase::nodes(graph).collect();
    if nodes.is_empty() {
        return true;
    }
    
    let mut visited = HashSet::new();
    let mut stack = vec![nodes[0]];
    
    while let Some(node) = stack.pop() {
        if visited.insert(node) {
            for neighbor in GraphBase::neighbors(graph, &node) {
                if !visited.contains(&neighbor) {
                    stack.push(neighbor);
                }
            }
        }
    }
    
    visited.len() == nodes.len()
}

/// Generate a navigable small-world graph
///
/// This creates a graph with both local connections and long-range connections
/// distributed according to a power law.
pub fn navigable_small_world(
    n: usize,
    p: usize,
    q: usize,
    r: f64,
    dim: usize,
    seed: Option<u64>,
) -> Result<Graph<Vec<usize>>> {
    if dim == 0 {
        return Err(NetworkXError::InvalidInput(
            "Dimension must be at least 1".to_string()
        ));
    }
    
    let mut rng = if let Some(s) = seed {
        ChaCha8Rng::seed_from_u64(s)
    } else {
        ChaCha8Rng::from_entropy()
    };
    
    let mut graph = Graph::new();
    
    // Create lattice nodes
    let side_length = (n as f64).powf(1.0 / dim as f64).ceil() as usize;
    let mut nodes = Vec::new();
    
    fn generate_coords(index: usize, side_length: usize, dim: usize) -> Vec<usize> {
        let mut coords = vec![0; dim];
        let mut idx = index;
        for i in 0..dim {
            coords[i] = idx % side_length;
            idx /= side_length;
        }
        coords
    }
    
    // Add nodes
    for i in 0..n {
        let coords = generate_coords(i, side_length, dim);
        graph.add_node(coords.clone());
        nodes.push(coords);
    }
    
    // Add local connections (lattice edges within distance p)
    for i in 0..nodes.len() {
        for j in (i + 1)..nodes.len() {
            let dist = manhattan_distance(&nodes[i], &nodes[j]);
            if dist <= p {
                graph.add_edge(nodes[i].clone(), nodes[j].clone(), None);
            }
        }
    }
    
    // Add long-range connections
    for i in 0..nodes.len() {
        for _ in 0..q {
            // Select target according to probability ~ d^(-r)
            let mut probs = Vec::new();
            let mut total = 0.0;
            
            for j in 0..nodes.len() {
                if i != j {
                    let dist = manhattan_distance(&nodes[i], &nodes[j]) as f64;
                    if dist > 0.0 {
                        let prob = dist.powf(-r);
                        probs.push((j, prob));
                        total += prob;
                    }
                }
            }
            
            if total > 0.0 {
                // Sample from distribution
                let mut sample = rng.gen::<f64>() * total;
                for (j, prob) in probs {
                    sample -= prob;
                    if sample <= 0.0 {
                        graph.add_edge(nodes[i].clone(), nodes[j].clone(), None);
                        break;
                    }
                }
            }
        }
    }
    
    Ok(graph)
}

/// Calculate Manhattan distance between two points
fn manhattan_distance(a: &[usize], b: &[usize]) -> usize {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (*x as i32 - *y as i32).abs() as usize)
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::traits::GraphBase;
    
    #[test]
    fn test_watts_strogatz_ring() {
        // With p=0, should be a ring lattice
        let graph = watts_strogatz(10, 4, 0.0, Some(42)).unwrap();
        assert_eq!(graph.node_count(), 10);
        assert_eq!(graph.edge_count(), 20); // Each node has 4 edges, total 40/2
        
        // Each node should have degree 4
        for i in 0..10 {
            assert_eq!(graph.degree(&i), 4);
        }
    }
    
    #[test]
    fn test_watts_strogatz_rewired() {
        // With p=1, all edges should be rewired
        let graph = watts_strogatz(20, 4, 1.0, Some(42)).unwrap();
        assert_eq!(graph.node_count(), 20);
        assert_eq!(graph.edge_count(), 40); // Total edges preserved
    }
    
    #[test]
    fn test_newman_watts_strogatz() {
        let graph = newman_watts_strogatz(10, 4, 0.3, Some(42)).unwrap();
        assert_eq!(graph.node_count(), 10);
        // Should have at least the ring edges
        assert!(graph.edge_count() >= 20);
    }
    
    #[test]
    fn test_connected_watts_strogatz() {
        let graph = connected_watts_strogatz(15, 4, 0.3, 10, Some(42)).unwrap();
        assert_eq!(graph.node_count(), 15);
        assert!(is_connected(&graph));
    }
    
    #[test]
    fn test_watts_strogatz_invalid() {
        assert!(watts_strogatz(5, 10, 0.5, None).is_err()); // k >= n
        assert!(watts_strogatz(10, 3, 0.5, None).is_err()); // k odd
        assert!(watts_strogatz(10, 4, -0.1, None).is_err()); // p < 0
        assert!(watts_strogatz(10, 4, 1.1, None).is_err()); // p > 1
    }
    
    #[test]
    fn test_navigable_small_world() {
        let graph = navigable_small_world(16, 2, 1, 2.0, 2, Some(42)).unwrap();
        assert_eq!(graph.node_count(), 16);
        // Should have both local and long-range connections
        assert!(graph.edge_count() > 0);
    }
}