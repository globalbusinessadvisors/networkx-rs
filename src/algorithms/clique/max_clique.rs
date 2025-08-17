//! Maximum clique algorithms

use super::{Clique, find_maximal_cliques};
use crate::graph::traits::GraphBase;
use crate::errors::Result;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

/// Find a maximum clique (clique with maximum number of nodes)
pub fn max_clique<G, N>(graph: &G) -> Result<Clique<N>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let cliques = find_maximal_cliques(graph)?;
    
    cliques.into_iter()
        .max_by_key(|c| c.len())
        .ok_or_else(|| crate::errors::NetworkXError::AlgorithmError(
            "No cliques found in graph".to_string()
        ))
}

/// Find the size of the maximum clique (clique number)
pub fn clique_number<G, N>(graph: &G) -> Result<usize>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let max = max_clique(graph)?;
    Ok(max.len())
}

/// Find maximum weight clique given node weights
pub fn max_weight_clique<G, N>(
    graph: &G,
    weights: &HashMap<N, f64>,
) -> Result<Clique<N>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let cliques = find_maximal_cliques(graph)?;
    
    cliques.into_iter()
        .max_by(|c1, c2| {
            let w1: f64 = c1.iter().map(|n| weights.get(n).copied().unwrap_or(1.0)).sum();
            let w2: f64 = c2.iter().map(|n| weights.get(n).copied().unwrap_or(1.0)).sum();
            w1.partial_cmp(&w2).unwrap()
        })
        .ok_or_else(|| crate::errors::NetworkXError::AlgorithmError(
            "No cliques found in graph".to_string()
        ))
}

/// Find all maximum cliques (all cliques of maximum size)
pub fn all_max_cliques<G, N>(graph: &G) -> Result<Vec<Clique<N>>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let cliques = find_maximal_cliques(graph)?;
    
    if cliques.is_empty() {
        return Ok(Vec::new());
    }
    
    let max_size = cliques.iter().map(|c| c.len()).max().unwrap();
    
    Ok(cliques.into_iter()
        .filter(|c| c.len() == max_size)
        .collect())
}

/// Approximate maximum clique using greedy algorithm
pub fn greedy_max_clique<G, N>(graph: &G) -> Result<Clique<N>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let mut clique = HashSet::new();
    let mut candidates: HashSet<N> = graph.nodes().collect();
    
    // Start with node of maximum degree
    if let Some(start) = candidates.iter()
        .max_by_key(|n| graph.degree(n))
        .cloned() 
    {
        clique.insert(start.clone());
        
        // Keep only neighbors as candidates
        let neighbors: HashSet<N> = graph.neighbors(&start).collect();
        candidates = candidates.intersection(&neighbors).cloned().collect();
        
        // Greedily add nodes that are connected to all current clique members
        while !candidates.is_empty() {
            // Find candidate connected to all clique members
            let next = candidates.iter()
                .find(|&c| {
                    clique.iter().all(|member| graph.has_edge(c, member))
                })
                .cloned();
            
            if let Some(node) = next {
                clique.insert(node.clone());
                
                // Update candidates to neighbors of new node
                let neighbors: HashSet<N> = graph.neighbors(&node).collect();
                candidates = candidates.intersection(&neighbors).cloned().collect();
            } else {
                break;
            }
        }
    }
    
    Ok(clique)
}

/// Find the k-core of a graph (maximal subgraph with minimum degree k)
pub fn k_core<G, N>(graph: &G, k: usize) -> Result<HashSet<N>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let mut remaining: HashSet<N> = graph.nodes().collect();
    let mut changed = true;
    
    while changed {
        changed = false;
        let to_remove: Vec<N> = remaining.iter()
            .filter(|n| {
                graph.neighbors(n)
                    .filter(|neighbor| remaining.contains(neighbor))
                    .count() < k
            })
            .cloned()
            .collect();
        
        if !to_remove.is_empty() {
            changed = true;
            for node in to_remove {
                remaining.remove(&node);
            }
        }
    }
    
    Ok(remaining)
}

/// Find the degeneracy and degeneracy ordering of a graph
pub fn degeneracy<G, N>(graph: &G) -> Result<(usize, Vec<N>)>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let mut remaining: HashSet<N> = graph.nodes().collect();
    let mut ordering = Vec::new();
    let mut max_core = 0;
    
    while !remaining.is_empty() {
        // Find node with minimum degree in remaining graph
        let (min_node, min_degree) = remaining.iter()
            .map(|n| {
                let degree = graph.neighbors(n)
                    .filter(|neighbor| remaining.contains(neighbor))
                    .count();
                (n.clone(), degree)
            })
            .min_by_key(|(_, d)| *d)
            .unwrap();
        
        max_core = max_core.max(min_degree);
        ordering.push(min_node.clone());
        remaining.remove(&min_node);
    }
    
    Ok((max_core, ordering))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::Graph;
    
    #[test]
    fn test_max_clique() {
        let mut graph = Graph::new();
        
        // Create K4 with an extra edge
        for i in 1..=4 {
            for j in (i+1)..=4 {
                graph.add_edge(i, j, None);
            }
        }
        graph.add_edge(4, 5, None);
        
        let max = max_clique(&graph).unwrap();
        assert_eq!(max.len(), 4);
        
        let clique_num = clique_number(&graph).unwrap();
        assert_eq!(clique_num, 4);
    }
    
    #[test]
    fn test_greedy_max_clique() {
        let mut graph = Graph::new();
        
        // Create a graph where greedy works well
        for i in 1..=3 {
            for j in (i+1)..=3 {
                graph.add_edge(i, j, None);
            }
        }
        
        let clique = greedy_max_clique(&graph).unwrap();
        assert_eq!(clique.len(), 3);
    }
    
    #[test]
    fn test_k_core() {
        let mut graph = Graph::new();
        
        // Create a graph with a 3-core
        for i in 1..=4 {
            for j in (i+1)..=4 {
                graph.add_edge(i, j, None);
            }
        }
        // Add some peripheral nodes
        graph.add_edge(5, 1, None);
        graph.add_edge(6, 2, None);
        
        let three_core = k_core(&graph, 3).unwrap();
        assert_eq!(three_core.len(), 4); // Nodes 1-4 form the 3-core
    }
    
    #[test]
    fn test_degeneracy() {
        let mut graph = Graph::new();
        
        // Path graph has degeneracy 1
        graph.add_edge(1, 2, None);
        graph.add_edge(2, 3, None);
        graph.add_edge(3, 4, None);
        
        let (degen, _ordering) = degeneracy(&graph).unwrap();
        assert_eq!(degen, 1);
        
        // Complete graph K4 has degeneracy 3
        let mut complete = Graph::new();
        for i in 1..=4 {
            for j in (i+1)..=4 {
                complete.add_edge(i, j, None);
            }
        }
        
        let (degen, _ordering) = degeneracy(&complete).unwrap();
        assert_eq!(degen, 3);
    }
}