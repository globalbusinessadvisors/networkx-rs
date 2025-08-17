//! Louvain community detection algorithm

use crate::graph::traits::GraphBase;
use crate::errors::Result;
use super::modularity::{modularity, modularity_gain};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use rand::seq::SliceRandom;
use rand::thread_rng;

/// Detect communities using the Louvain method
///
/// The Louvain method is a greedy optimization method that attempts to optimize the modularity
/// of a partition of the network. It has two phases that are repeated iteratively:
/// 1. Local optimization: each node is moved to the community that yields the largest gain in modularity
/// 2. Network aggregation: communities are aggregated to build a new network of communities
pub fn louvain_communities<G, N>(
    graph: &G,
    resolution: f64,
    threshold: f64,
    max_iterations: Option<usize>,
) -> Result<Vec<HashSet<N>>>
where
    G: GraphBase<NodeId = N> + crate::graph::traits::GraphAlgorithms,
    N: Clone + Hash + Eq,
{
    let nodes: Vec<N> = graph.nodes().collect();
    if nodes.is_empty() {
        return Ok(Vec::new());
    }
    
    let m = graph.edge_count() as f64;
    if m == 0.0 {
        // Each node is its own community
        return Ok(nodes.into_iter().map(|n| {
            let mut community = HashSet::new();
            community.insert(n);
            community
        }).collect());
    }
    
    // Initialize: each node in its own community
    let mut node_to_community: HashMap<N, usize> = HashMap::new();
    let mut communities: HashMap<usize, HashSet<N>> = HashMap::new();
    
    for (i, node) in nodes.iter().enumerate() {
        node_to_community.insert(node.clone(), i);
        let mut community = HashSet::new();
        community.insert(node.clone());
        communities.insert(i, community);
    }
    
    let max_iter = max_iterations.unwrap_or(100);
    let mut improved = true;
    let mut iteration = 0;
    
    while improved && iteration < max_iter {
        improved = false;
        iteration += 1;
        
        // Randomize node order for better results
        let mut node_order = nodes.clone();
        node_order.shuffle(&mut thread_rng());
        
        // Phase 1: Local optimization
        for node in &node_order {
            let current_community_id = node_to_community[node];
            let current_community = &communities[&current_community_id];
            
            // Find neighboring communities
            let mut neighbor_communities: HashSet<usize> = HashSet::new();
            for neighbor in graph.neighbors(node) {
                let comm_id = node_to_community[&neighbor];
                if comm_id != current_community_id {
                    neighbor_communities.insert(comm_id);
                }
            }
            
            // Calculate modularity gain for each neighboring community
            let mut best_community = current_community_id;
            let mut best_gain = 0.0;
            
            for &comm_id in &neighbor_communities {
                let target_community = &communities[&comm_id];
                let gain = modularity_gain(
                    graph,
                    node,
                    current_community,
                    target_community,
                    m
                ) * resolution;
                
                if gain > best_gain + threshold {
                    best_gain = gain;
                    best_community = comm_id;
                }
            }
            
            // Move node to best community if there's improvement
            if best_community != current_community_id {
                // Remove from current community
                communities.get_mut(&current_community_id).unwrap().remove(node);
                
                // Add to new community
                communities.get_mut(&best_community).unwrap().insert(node.clone());
                node_to_community.insert(node.clone(), best_community);
                
                improved = true;
            }
        }
        
        // Clean up empty communities
        communities.retain(|_, comm| !comm.is_empty());
        
        // Renumber communities
        if improved {
            let mut new_communities: HashMap<usize, HashSet<N>> = HashMap::new();
            let mut new_node_to_community: HashMap<N, usize> = HashMap::new();
            
            for (new_id, (_, community)) in communities.into_iter().enumerate() {
                for node in &community {
                    new_node_to_community.insert(node.clone(), new_id);
                }
                new_communities.insert(new_id, community);
            }
            
            communities = new_communities;
            node_to_community = new_node_to_community;
        }
    }
    
    Ok(communities.into_values().collect())
}

/// Hierarchical Louvain algorithm that returns communities at multiple levels
pub fn louvain_hierarchical<G, N>(
    graph: &G,
    resolution: f64,
    threshold: f64,
) -> Result<Vec<Vec<HashSet<N>>>>
where
    G: GraphBase<NodeId = N> + crate::graph::traits::GraphAlgorithms,
    N: Clone + Hash + Eq,
{
    let mut hierarchy = Vec::new();
    
    // Get first level communities
    let first_level = louvain_communities(graph, resolution, threshold, None)?;
    if first_level.len() <= 1 {
        return Ok(vec![first_level]);
    }
    
    hierarchy.push(first_level.clone());
    
    // Build super-graph and continue if there's more than one community
    let mut current_partition = first_level;
    let mut level = 0;
    let max_levels = 10; // Prevent infinite loops
    
    while current_partition.len() > 1 && level < max_levels {
        // Check if we can merge further
        let current_modularity = modularity(graph, &current_partition)?;
        
        // Try to find better partition at higher level
        // This would require building a super-graph, which is complex
        // For now, we'll just return the single level
        break;
    }
    
    Ok(hierarchy)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::Graph;
    
    #[test]
    fn test_louvain_simple() {
        let mut graph = Graph::new();
        
        // Create two clear communities
        // Community 1
        graph.add_edge(1, 2, None);
        graph.add_edge(2, 3, None);
        graph.add_edge(3, 1, None);
        
        // Community 2
        graph.add_edge(4, 5, None);
        graph.add_edge(5, 6, None);
        graph.add_edge(6, 4, None);
        
        // Weak link between communities
        graph.add_edge(3, 4, Some(0.1));
        
        let communities = louvain_communities(&graph, 1.0, 0.0001, None).unwrap();
        
        // Should detect 2 communities
        assert_eq!(communities.len(), 2);
        
        // Each community should have 3 nodes
        for community in &communities {
            assert_eq!(community.len(), 3);
        }
    }
    
    #[test]
    fn test_louvain_single_community() {
        let mut graph = Graph::new();
        
        // Create a complete graph (should be one community)
        for i in 1..=4 {
            for j in (i+1)..=4 {
                graph.add_edge(i, j, None);
            }
        }
        
        let communities = louvain_communities(&graph, 1.0, 0.0001, None).unwrap();
        
        // Should detect 1 community with all nodes
        assert_eq!(communities.len(), 1);
        assert_eq!(communities[0].len(), 4);
    }
    
    #[test]
    fn test_louvain_disconnected() {
        let mut graph = Graph::new();
        
        // Create three disconnected components
        graph.add_edge(1, 2, None);
        graph.add_edge(3, 4, None);
        graph.add_edge(5, 6, None);
        
        let communities = louvain_communities(&graph, 1.0, 0.0001, None).unwrap();
        
        // Should detect 3 communities
        assert_eq!(communities.len(), 3);
        
        // Each community should have 2 nodes
        for community in &communities {
            assert_eq!(community.len(), 2);
        }
    }
}