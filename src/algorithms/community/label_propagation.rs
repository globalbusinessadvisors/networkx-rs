//! Label Propagation Algorithm for community detection

use crate::graph::traits::GraphBase;
use crate::errors::Result;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use rand::seq::SliceRandom;
use rand::thread_rng;

/// Detect communities using Label Propagation Algorithm (LPA)
///
/// The Label Propagation Algorithm is a fast algorithm for finding communities in a graph.
/// It works by propagating labels throughout the network and forming communities based on
/// this propagation process. Each node adopts the label that most of its neighbors have.
pub fn label_propagation_communities<G, N>(
    graph: &G,
    max_iterations: Option<usize>,
) -> Result<Vec<HashSet<N>>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let nodes: Vec<N> = graph.nodes().collect();
    if nodes.is_empty() {
        return Ok(Vec::new());
    }
    
    // Initialize: each node gets a unique label
    let mut labels: HashMap<N, usize> = HashMap::new();
    for (i, node) in nodes.iter().enumerate() {
        labels.insert(node.clone(), i);
    }
    
    let max_iter = max_iterations.unwrap_or(100);
    let mut changed = true;
    let mut iteration = 0;
    
    while changed && iteration < max_iter {
        changed = false;
        iteration += 1;
        
        // Random order for asynchronous updating
        let mut node_order = nodes.clone();
        node_order.shuffle(&mut thread_rng());
        
        for node in &node_order {
            // Count labels of neighbors
            let mut label_counts: HashMap<usize, f64> = HashMap::new();
            
            for neighbor in graph.neighbors(node) {
                if let Some(&neighbor_label) = labels.get(&neighbor) {
                    let weight = graph.get_edge_weight(node, &neighbor).unwrap_or(1.0);
                    *label_counts.entry(neighbor_label).or_insert(0.0) += weight;
                }
            }
            
            if label_counts.is_empty() {
                continue;
            }
            
            // Find the most frequent label(s)
            let max_count = label_counts.values().cloned().fold(0.0, f64::max);
            let max_labels: Vec<usize> = label_counts
                .iter()
                .filter(|(_, &count)| count == max_count)
                .map(|(&label, _)| label)
                .collect();
            
            // Choose one of the max labels (randomly if there's a tie)
            let current_label = labels[node];
            let new_label = if max_labels.contains(&current_label) {
                // Keep current label if it's among the most frequent
                current_label
            } else {
                // Otherwise, randomly choose from the most frequent labels
                *max_labels.choose(&mut thread_rng()).unwrap()
            };
            
            if new_label != current_label {
                labels.insert(node.clone(), new_label);
                changed = true;
            }
        }
    }
    
    // Group nodes by their labels to form communities
    let mut communities: HashMap<usize, HashSet<N>> = HashMap::new();
    for (node, label) in labels {
        communities.entry(label).or_insert_with(HashSet::new).insert(node);
    }
    
    Ok(communities.into_values().collect())
}

/// Asynchronous Label Propagation Algorithm
///
/// This variant updates all nodes simultaneously in each iteration,
/// which can lead to different (sometimes better) results.
pub fn async_label_propagation<G, N>(
    graph: &G,
    max_iterations: Option<usize>,
) -> Result<Vec<HashSet<N>>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let nodes: Vec<N> = graph.nodes().collect();
    if nodes.is_empty() {
        return Ok(Vec::new());
    }
    
    // Initialize: each node gets a unique label
    let mut labels: HashMap<N, usize> = HashMap::new();
    for (i, node) in nodes.iter().enumerate() {
        labels.insert(node.clone(), i);
    }
    
    let max_iter = max_iterations.unwrap_or(100);
    let mut changed = true;
    let mut iteration = 0;
    
    while changed && iteration < max_iter {
        changed = false;
        iteration += 1;
        
        // Calculate new labels for all nodes simultaneously
        let mut new_labels: HashMap<N, usize> = HashMap::new();
        
        for node in &nodes {
            // Count labels of neighbors
            let mut label_counts: HashMap<usize, f64> = HashMap::new();
            
            for neighbor in graph.neighbors(node) {
                if let Some(&neighbor_label) = labels.get(&neighbor) {
                    let weight = graph.get_edge_weight(node, &neighbor).unwrap_or(1.0);
                    *label_counts.entry(neighbor_label).or_insert(0.0) += weight;
                }
            }
            
            if label_counts.is_empty() {
                new_labels.insert(node.clone(), labels[node]);
                continue;
            }
            
            // Find the most frequent label
            let max_count = label_counts.values().cloned().fold(0.0, f64::max);
            let max_labels: Vec<usize> = label_counts
                .iter()
                .filter(|(_, &count)| count == max_count)
                .map(|(&label, _)| label)
                .collect();
            
            // Choose the smallest label among ties (for determinism)
            let new_label = *max_labels.iter().min().unwrap();
            new_labels.insert(node.clone(), new_label);
            
            if new_label != labels[node] {
                changed = true;
            }
        }
        
        labels = new_labels;
    }
    
    // Group nodes by their labels to form communities
    let mut communities: HashMap<usize, HashSet<N>> = HashMap::new();
    for (node, label) in labels {
        communities.entry(label).or_insert_with(HashSet::new).insert(node);
    }
    
    Ok(communities.into_values().collect())
}

/// Semi-synchronous Label Propagation Algorithm
///
/// This variant combines aspects of synchronous and asynchronous updates,
/// updating nodes in small batches.
pub fn semi_sync_label_propagation<G, N>(
    graph: &G,
    batch_size: usize,
    max_iterations: Option<usize>,
) -> Result<Vec<HashSet<N>>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let nodes: Vec<N> = graph.nodes().collect();
    if nodes.is_empty() {
        return Ok(Vec::new());
    }
    
    // Initialize: each node gets a unique label
    let mut labels: HashMap<N, usize> = HashMap::new();
    for (i, node) in nodes.iter().enumerate() {
        labels.insert(node.clone(), i);
    }
    
    let max_iter = max_iterations.unwrap_or(100);
    let mut changed = true;
    let mut iteration = 0;
    
    while changed && iteration < max_iter {
        changed = false;
        iteration += 1;
        
        // Random order for batch processing
        let mut node_order = nodes.clone();
        node_order.shuffle(&mut thread_rng());
        
        // Process nodes in batches
        for batch in node_order.chunks(batch_size) {
            let mut batch_updates: Vec<(N, usize)> = Vec::new();
            
            for node in batch {
                // Count labels of neighbors
                let mut label_counts: HashMap<usize, f64> = HashMap::new();
                
                for neighbor in graph.neighbors(node) {
                    if let Some(&neighbor_label) = labels.get(&neighbor) {
                        let weight = graph.get_edge_weight(node, &neighbor).unwrap_or(1.0);
                        *label_counts.entry(neighbor_label).or_insert(0.0) += weight;
                    }
                }
                
                if label_counts.is_empty() {
                    continue;
                }
                
                // Find the most frequent label
                let max_count = label_counts.values().cloned().fold(0.0, f64::max);
                let max_labels: Vec<usize> = label_counts
                    .iter()
                    .filter(|(_, &count)| count == max_count)
                    .map(|(&label, _)| label)
                    .collect();
                
                let current_label = labels[node];
                let new_label = if max_labels.contains(&current_label) {
                    current_label
                } else {
                    *max_labels.choose(&mut thread_rng()).unwrap()
                };
                
                if new_label != current_label {
                    batch_updates.push((node.clone(), new_label));
                    changed = true;
                }
            }
            
            // Apply batch updates
            for (node, label) in batch_updates {
                labels.insert(node, label);
            }
        }
    }
    
    // Group nodes by their labels to form communities
    let mut communities: HashMap<usize, HashSet<N>> = HashMap::new();
    for (node, label) in labels {
        communities.entry(label).or_insert_with(HashSet::new).insert(node);
    }
    
    Ok(communities.into_values().collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::Graph;
    
    #[test]
    fn test_label_propagation_simple() {
        let mut graph = Graph::new();
        
        // Create two clear communities
        // Community 1: triangle
        graph.add_edge(1, 2, None);
        graph.add_edge(2, 3, None);
        graph.add_edge(3, 1, None);
        
        // Community 2: triangle
        graph.add_edge(4, 5, None);
        graph.add_edge(5, 6, None);
        graph.add_edge(6, 4, None);
        
        // Weak link between communities
        graph.add_edge(3, 4, Some(0.1));
        
        let communities = label_propagation_communities(&graph, Some(50)).unwrap();
        
        // Should typically detect 2 communities (though LPA can be non-deterministic)
        assert!(communities.len() >= 1 && communities.len() <= 2);
        
        // Total nodes should be 6
        let total_nodes: usize = communities.iter().map(|c| c.len()).sum();
        assert_eq!(total_nodes, 6);
    }
    
    #[test]
    fn test_async_label_propagation() {
        let mut graph = Graph::new();
        
        // Create a simple path graph
        graph.add_edge(1, 2, None);
        graph.add_edge(2, 3, None);
        graph.add_edge(3, 4, None);
        graph.add_edge(4, 5, None);
        
        let communities = async_label_propagation(&graph, Some(50)).unwrap();
        
        // Path graph might form 1-2 communities
        assert!(communities.len() >= 1 && communities.len() <= 5);
        
        // Total nodes should be 5
        let total_nodes: usize = communities.iter().map(|c| c.len()).sum();
        assert_eq!(total_nodes, 5);
    }
    
    #[test]
    fn test_semi_sync_label_propagation() {
        let mut graph = Graph::new();
        
        // Create a star graph
        for i in 1..=5 {
            graph.add_edge(0, i, None);
        }
        
        let communities = semi_sync_label_propagation(&graph, 2, Some(50)).unwrap();
        
        // Star graph should typically form 1 community
        assert_eq!(communities.len(), 1);
        assert_eq!(communities[0].len(), 6);
    }
    
    #[test]
    fn test_label_propagation_disconnected() {
        let mut graph = Graph::new();
        
        // Create disconnected components
        graph.add_edge(1, 2, None);
        graph.add_edge(3, 4, None);
        graph.add_edge(5, 6, None);
        
        let communities = label_propagation_communities(&graph, Some(10)).unwrap();
        
        // Should detect 3 communities
        assert_eq!(communities.len(), 3);
        
        // Each community should have 2 nodes
        for community in &communities {
            assert_eq!(community.len(), 2);
        }
    }
}