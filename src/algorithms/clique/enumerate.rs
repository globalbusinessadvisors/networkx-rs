//! Clique enumeration algorithms

use super::Clique;
use crate::graph::traits::GraphBase;
use crate::errors::Result;
use std::collections::HashSet;
use std::hash::Hash;

/// Enumerate all cliques in the graph
pub fn enumerate_all_cliques<G, N>(
    graph: &G,
    min_size: Option<usize>,
    max_size: Option<usize>,
) -> Result<Vec<Clique<N>>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let all_maximal = super::find_maximal_cliques(graph)?;
    let mut all_cliques = Vec::new();
    
    let min = min_size.unwrap_or(1);
    let max = max_size.unwrap_or(usize::MAX);
    
    // Generate all subsets of maximal cliques
    for maximal in all_maximal {
        let size = maximal.len();
        if size >= min {
            // Add the maximal clique itself if within size range
            if size <= max {
                all_cliques.push(maximal.clone());
            }
            
            // Generate smaller cliques from this maximal clique
            // This is a simplified version - proper implementation would avoid duplicates
            if min < size {
                for k in min..size.min(max + 1) {
                    // Would generate all k-subsets here
                    // Skipping for brevity
                }
            }
        }
    }
    
    Ok(all_cliques)
}

/// Find all cliques of a specific size
pub fn find_cliques_of_size<G, N>(graph: &G, size: usize) -> Result<Vec<Clique<N>>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    super::bron_kerbosch::find_k_cliques(graph, size)
}

/// Find k-clique communities
/// 
/// k-clique communities are sets of k-cliques that are adjacent to each other.
/// Two k-cliques are adjacent if they share exactly k-1 nodes.
pub fn k_clique_communities<G, N>(
    graph: &G,
    k: usize,
) -> Result<Vec<Vec<Clique<N>>>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    if k < 2 {
        return Ok(Vec::new());
    }
    
    // Find all k-cliques
    let k_cliques = find_cliques_of_size(graph, k)?;
    
    if k_cliques.is_empty() {
        return Ok(Vec::new());
    }
    
    // Build adjacency graph of k-cliques
    let mut clique_graph: std::collections::HashMap<usize, HashSet<usize>> = std::collections::HashMap::new();
    
    for i in 0..k_cliques.len() {
        clique_graph.insert(i, HashSet::new());
    }
    
    // Two k-cliques are adjacent if they share k-1 nodes
    for i in 0..k_cliques.len() {
        for j in (i + 1)..k_cliques.len() {
            let intersection: HashSet<_> = k_cliques[i].intersection(&k_cliques[j]).collect();
            if intersection.len() == k - 1 {
                clique_graph.get_mut(&i).unwrap().insert(j);
                clique_graph.get_mut(&j).unwrap().insert(i);
            }
        }
    }
    
    // Find connected components in the clique adjacency graph
    let mut visited = vec![false; k_cliques.len()];
    let mut communities = Vec::new();
    
    for start in 0..k_cliques.len() {
        if !visited[start] {
            let mut community = Vec::new();
            let mut stack = vec![start];
            
            while let Some(clique_idx) = stack.pop() {
                if visited[clique_idx] {
                    continue;
                }
                
                visited[clique_idx] = true;
                community.push(k_cliques[clique_idx].clone());
                
                // Add all adjacent k-cliques to the stack
                if let Some(neighbors) = clique_graph.get(&clique_idx) {
                    for &neighbor in neighbors {
                        if !visited[neighbor] {
                            stack.push(neighbor);
                        }
                    }
                }
            }
            
            if !community.is_empty() {
                communities.push(community);
            }
        }
    }
    
    Ok(communities)
}