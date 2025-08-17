//! Modularity calculation for community detection

use crate::graph::traits::GraphBase;
use crate::errors::Result;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

/// Calculate the modularity of a graph given a community partition
///
/// Modularity is a measure of the structure of networks or graphs which measures the strength
/// of division of a network into modules (communities). Networks with high modularity have
/// dense connections between the nodes within modules but sparse connections between nodes
/// in different modules.
pub fn modularity<G, N>(
    graph: &G,
    communities: &[HashSet<N>],
) -> Result<f64>
where
    G: GraphBase<NodeId = N> + crate::graph::traits::GraphAlgorithms,
    N: Clone + Hash + Eq,
{
    if communities.is_empty() {
        return Ok(0.0);
    }
    
    let m = graph.edge_count() as f64;
    if m == 0.0 {
        return Ok(0.0);
    }
    
    // For undirected graphs, we need to account for each edge twice
    let norm = if graph.is_directed() {
        1.0 / m
    } else {
        1.0 / (2.0 * m)
    };
    
    let mut modularity_value = 0.0;
    
    // Calculate degree for each node
    let degrees: HashMap<N, f64> = graph.nodes()
        .map(|node| {
            let degree = graph.neighbors(&node).count() as f64;
            (node, degree)
        })
        .collect();
    
    // Calculate modularity for each community
    for community in communities {
        let mut internal_edges = 0.0;
        let mut total_degree = 0.0;
        
        for node in community {
            total_degree += degrees.get(node).unwrap_or(&0.0);
            
            // Count edges within the community
            for neighbor in graph.neighbors(node) {
                if community.contains(&neighbor) {
                    internal_edges += graph.get_edge_weight(node, &neighbor).unwrap_or(1.0);
                }
            }
        }
        
        // For undirected graphs, internal edges are counted twice
        if !graph.is_directed() {
            internal_edges /= 2.0;
        }
        
        let expected_edges = if graph.is_directed() {
            total_degree * total_degree / m
        } else {
            total_degree * total_degree / (4.0 * m)
        };
        
        modularity_value += internal_edges * norm - expected_edges * norm * norm;
    }
    
    Ok(modularity_value)
}

/// Create the modularity matrix for the graph
///
/// The modularity matrix B is defined as B_ij = A_ij - k_i * k_j / (2m)
/// where A is the adjacency matrix, k_i is the degree of node i, and m is the number of edges.
pub fn modularity_matrix<G, N>(
    graph: &G,
) -> Result<HashMap<(N, N), f64>>
where
    G: GraphBase<NodeId = N> + crate::graph::traits::GraphAlgorithms,
    N: Clone + Hash + Eq,
{
    let nodes: Vec<N> = graph.nodes().collect();
    let m = graph.edge_count() as f64;
    
    if m == 0.0 {
        return Ok(HashMap::new());
    }
    
    let norm = if graph.is_directed() {
        m
    } else {
        2.0 * m
    };
    
    // Calculate degree for each node
    let degrees: HashMap<N, f64> = nodes
        .iter()
        .map(|node| {
            let degree = graph.neighbors(node).count() as f64;
            (node.clone(), degree)
        })
        .collect();
    
    let mut matrix = HashMap::new();
    
    for i in &nodes {
        for j in &nodes {
            let a_ij = if graph.has_edge(i, j) {
                graph.get_edge_weight(i, j).unwrap_or(1.0)
            } else {
                0.0
            };
            
            let expected = degrees[i] * degrees[j] / norm;
            matrix.insert((i.clone(), j.clone()), a_ij - expected);
        }
    }
    
    Ok(matrix)
}

/// Calculate the change in modularity when moving a node to a different community
pub fn modularity_gain<G, N>(
    graph: &G,
    node: &N,
    from_community: &HashSet<N>,
    to_community: &HashSet<N>,
    m: f64,
) -> f64
where
    G: GraphBase<NodeId = N> + crate::graph::traits::GraphAlgorithms,
    N: Clone + Hash + Eq,
{
    let mut k_in_from = 0.0;
    let mut k_in_to = 0.0;
    let mut k_node = 0.0;
    
    // Calculate edges from node to each community
    for neighbor in graph.neighbors(node) {
        let weight = graph.get_edge_weight(node, &neighbor).unwrap_or(1.0);
        k_node += weight;
        
        if from_community.contains(&neighbor) && &neighbor != node {
            k_in_from += weight;
        }
        if to_community.contains(&neighbor) {
            k_in_to += weight;
        }
    }
    
    // Calculate sum of degrees in each community
    let sigma_from: f64 = from_community
        .iter()
        .filter(|&n| n != node)
        .map(|n| graph.degree(n) as f64)
        .sum();
    
    let sigma_to: f64 = to_community
        .iter()
        .map(|n| graph.degree(n) as f64)
        .sum();
    
    let norm = if graph.is_directed() {
        1.0 / m
    } else {
        1.0 / (2.0 * m)
    };
    
    // Calculate modularity gain
    let gain = (k_in_to - k_in_from) * norm - 
               k_node * (sigma_to - sigma_from + k_node) * norm * norm;
    
    gain
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::Graph;
    
    #[test]
    fn test_modularity_single_community() {
        let mut graph = Graph::new();
        graph.add_edge(1, 2, None);
        graph.add_edge(2, 3, None);
        graph.add_edge(3, 1, None);
        
        let mut community = HashSet::new();
        community.insert(1);
        community.insert(2);
        community.insert(3);
        
        let communities = vec![community];
        let mod_value = modularity(&graph, &communities).unwrap();
        
        // Single community should have modularity of 0
        assert!(mod_value.abs() < 0.01);
    }
    
    #[test]
    fn test_modularity_two_communities() {
        let mut graph = Graph::new();
        // First community
        graph.add_edge(1, 2, None);
        graph.add_edge(2, 3, None);
        graph.add_edge(3, 1, None);
        
        // Second community
        graph.add_edge(4, 5, None);
        graph.add_edge(5, 6, None);
        graph.add_edge(6, 4, None);
        
        // Bridge between communities
        graph.add_edge(3, 4, None);
        
        let mut community1 = HashSet::new();
        community1.insert(1);
        community1.insert(2);
        community1.insert(3);
        
        let mut community2 = HashSet::new();
        community2.insert(4);
        community2.insert(5);
        community2.insert(6);
        
        let communities = vec![community1, community2];
        let mod_value = modularity(&graph, &communities).unwrap();
        
        // Two well-separated communities should have positive modularity
        assert!(mod_value > 0.0);
    }
}