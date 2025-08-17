//! Bron-Kerbosch algorithm for finding maximal cliques

use super::Clique;
use crate::graph::traits::GraphBase;
use crate::errors::Result;
use std::collections::HashSet;
use std::hash::Hash;

/// Find all maximal cliques using the Bron-Kerbosch algorithm
pub fn find_maximal_cliques<G, N>(graph: &G) -> Result<Vec<Clique<N>>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let mut cliques = Vec::new();
    let r = HashSet::new();
    let p: HashSet<N> = graph.nodes().collect();
    let x = HashSet::new();
    
    bron_kerbosch_recursive(graph, r, p, x, &mut cliques);
    
    Ok(cliques)
}

/// Bron-Kerbosch algorithm (without pivoting)
pub fn bron_kerbosch<G, N>(graph: &G) -> Result<Vec<Clique<N>>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    find_maximal_cliques(graph)
}

/// Recursive Bron-Kerbosch algorithm
fn bron_kerbosch_recursive<G, N>(
    graph: &G,
    r: HashSet<N>,
    mut p: HashSet<N>,
    mut x: HashSet<N>,
    cliques: &mut Vec<Clique<N>>,
) where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    if p.is_empty() && x.is_empty() {
        if !r.is_empty() {
            cliques.push(r);
        }
        return;
    }
    
    let p_copy = p.clone();
    for v in p_copy {
        let mut r_new = r.clone();
        r_new.insert(v.clone());
        
        let neighbors: HashSet<N> = graph.neighbors(&v).collect();
        let p_new: HashSet<N> = p.intersection(&neighbors).cloned().collect();
        let x_new: HashSet<N> = x.intersection(&neighbors).cloned().collect();
        
        bron_kerbosch_recursive(graph, r_new, p_new, x_new, cliques);
        
        p.remove(&v);
        x.insert(v);
    }
}

/// Bron-Kerbosch with pivoting for improved performance
pub fn bron_kerbosch_with_pivot<G, N>(graph: &G) -> Result<Vec<Clique<N>>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let mut cliques = Vec::new();
    let r = HashSet::new();
    let p: HashSet<N> = graph.nodes().collect();
    let x = HashSet::new();
    
    bron_kerbosch_pivot_recursive(graph, r, p, x, &mut cliques);
    
    Ok(cliques)
}

/// Recursive Bron-Kerbosch with pivoting
fn bron_kerbosch_pivot_recursive<G, N>(
    graph: &G,
    r: HashSet<N>,
    mut p: HashSet<N>,
    mut x: HashSet<N>,
    cliques: &mut Vec<Clique<N>>,
) where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    if p.is_empty() && x.is_empty() {
        if !r.is_empty() {
            cliques.push(r);
        }
        return;
    }
    
    // Choose pivot with maximum degree in P ∪ X
    let pivot = p.union(&x)
        .max_by_key(|v| {
            graph.neighbors(v)
                .filter(|n| p.contains(n))
                .count()
        })
        .cloned();
    
    if let Some(u) = pivot {
        let neighbors: HashSet<N> = graph.neighbors(&u).collect();
        let candidates: Vec<N> = p.difference(&neighbors).cloned().collect();
        
        for v in candidates {
            let mut r_new = r.clone();
            r_new.insert(v.clone());
            
            let v_neighbors: HashSet<N> = graph.neighbors(&v).collect();
            let p_new: HashSet<N> = p.intersection(&v_neighbors).cloned().collect();
            let x_new: HashSet<N> = x.intersection(&v_neighbors).cloned().collect();
            
            bron_kerbosch_pivot_recursive(graph, r_new, p_new, x_new, cliques);
            
            p.remove(&v);
            x.insert(v);
        }
    }
}

/// Find all cliques (convenience function)
pub fn find_cliques<G, N>(graph: &G) -> Result<Vec<Clique<N>>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    bron_kerbosch_with_pivot(graph)
}

/// Find all k-cliques (cliques of size k)
pub fn find_k_cliques<G, N>(graph: &G, k: usize) -> Result<Vec<Clique<N>>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let all_cliques = find_cliques(graph)?;
    Ok(all_cliques.into_iter()
        .filter(|clique| clique.len() == k)
        .collect())
}

/// Count the number of maximal cliques
pub fn count_maximal_cliques<G, N>(graph: &G) -> Result<usize>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let cliques = find_maximal_cliques(graph)?;
    Ok(cliques.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::Graph;
    
    #[test]
    fn test_find_cliques_triangle() {
        let mut graph = Graph::new();
        
        // Triangle is a 3-clique
        graph.add_edge(1, 2, None);
        graph.add_edge(2, 3, None);
        graph.add_edge(3, 1, None);
        
        let cliques = find_maximal_cliques(&graph).unwrap();
        
        assert_eq!(cliques.len(), 1);
        assert_eq!(cliques[0].len(), 3);
    }
    
    #[test]
    fn test_find_cliques_disconnected() {
        let mut graph = Graph::new();
        
        // Two triangles
        graph.add_edge(1, 2, None);
        graph.add_edge(2, 3, None);
        graph.add_edge(3, 1, None);
        
        graph.add_edge(4, 5, None);
        graph.add_edge(5, 6, None);
        graph.add_edge(6, 4, None);
        
        let cliques = find_maximal_cliques(&graph).unwrap();
        
        assert_eq!(cliques.len(), 2);
        for clique in &cliques {
            assert_eq!(clique.len(), 3);
        }
    }
    
    #[test]
    fn test_bron_kerbosch_with_pivot() {
        let mut graph = Graph::new();
        
        // Create a more complex graph
        for i in 1..=4 {
            for j in (i+1)..=4 {
                graph.add_edge(i, j, None);
            }
        }
        // Add another node connected to some
        graph.add_edge(5, 1, None);
        graph.add_edge(5, 2, None);
        
        let cliques = bron_kerbosch_with_pivot(&graph).unwrap();
        
        // Should find the 4-clique and a 3-clique
        let max_clique_size = cliques.iter().map(|c| c.len()).max().unwrap();
        assert_eq!(max_clique_size, 4);
    }
    
    #[test]
    fn test_find_k_cliques() {
        let mut graph = Graph::new();
        
        // Complete graph K4
        for i in 1..=4 {
            for j in (i+1)..=4 {
                graph.add_edge(i, j, None);
            }
        }
        
        let three_cliques = find_k_cliques(&graph, 3).unwrap();
        
        // K4 has C(4,3) = 4 three-cliques
        assert_eq!(three_cliques.len(), 4);
    }
}