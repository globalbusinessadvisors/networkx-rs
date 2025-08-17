//! DSATUR (Degree of Saturation) coloring algorithm

use super::ColoringResult;
use crate::graph::traits::GraphBase;
use crate::errors::Result;
use std::collections::{HashMap, HashSet, BinaryHeap};
use std::cmp::Ordering;
use std::hash::Hash;

/// Node with saturation degree for priority queue
#[derive(Clone)]
struct SaturatedNode<N> {
    node: N,
    saturation: usize,
    degree: usize,
}

impl<N: Clone> SaturatedNode<N> {
    fn new(node: N, saturation: usize, degree: usize) -> Self {
        SaturatedNode { node, saturation, degree }
    }
}

impl<N: Clone> PartialEq for SaturatedNode<N> {
    fn eq(&self, other: &Self) -> bool {
        self.saturation == other.saturation && self.degree == other.degree
    }
}

impl<N: Clone> Eq for SaturatedNode<N> {}

impl<N: Clone> PartialOrd for SaturatedNode<N> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<N: Clone> Ord for SaturatedNode<N> {
    fn cmp(&self, other: &Self) -> Ordering {
        // First by saturation (higher is better)
        match self.saturation.cmp(&other.saturation) {
            Ordering::Equal => {
                // Then by degree (higher is better)
                self.degree.cmp(&other.degree)
            }
            other => other
        }
    }
}

/// DSATUR coloring algorithm
///
/// DSATUR (Degree of Saturation) is an exact algorithm for graph coloring
/// that colors nodes in order of their saturation degree - the number of
/// different colors used by their neighbors.
pub fn dsatur_coloring<G, N>(graph: &G) -> Result<ColoringResult<N>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let mut coloring: HashMap<N, usize> = HashMap::new();
    let mut saturation: HashMap<N, HashSet<usize>> = HashMap::new();
    let mut uncolored: HashSet<N> = graph.nodes().collect();
    
    if uncolored.is_empty() {
        return Ok(ColoringResult::new(coloring));
    }
    
    // Initialize saturation degrees
    for node in &uncolored {
        saturation.insert(node.clone(), HashSet::new());
    }
    
    // Start with node of maximum degree
    let first_node = uncolored
        .iter()
        .max_by_key(|n| graph.degree(n))
        .cloned()
        .unwrap();
    
    coloring.insert(first_node.clone(), 0);
    uncolored.remove(&first_node);
    
    // Update saturation of neighbors
    for neighbor in graph.neighbors(&first_node) {
        if let Some(sat) = saturation.get_mut(&neighbor) {
            sat.insert(0);
        }
    }
    
    // Color remaining nodes
    while !uncolored.is_empty() {
        // Find node with maximum saturation degree
        let next_node = uncolored
            .iter()
            .max_by(|a, b| {
                let sat_a = saturation[a].len();
                let sat_b = saturation[b].len();
                
                match sat_a.cmp(&sat_b) {
                    Ordering::Equal => {
                        // Tie-break by degree
                        graph.degree(a).cmp(&graph.degree(b))
                    }
                    other => other
                }
            })
            .cloned()
            .unwrap();
        
        // Find smallest available color
        let neighbor_colors: HashSet<usize> = graph.neighbors(&next_node)
            .filter_map(|n| coloring.get(&n).copied())
            .collect();
        
        let mut color = 0;
        while neighbor_colors.contains(&color) {
            color += 1;
        }
        
        coloring.insert(next_node.clone(), color);
        uncolored.remove(&next_node);
        
        // Update saturation of uncolored neighbors
        for neighbor in graph.neighbors(&next_node) {
            if uncolored.contains(&neighbor) {
                if let Some(sat) = saturation.get_mut(&neighbor) {
                    sat.insert(color);
                }
            }
        }
    }
    
    Ok(ColoringResult::new(coloring))
}

/// DSATUR with backtracking for exact chromatic number
pub fn dsatur_exact<G, N>(
    graph: &G,
    upper_bound: Option<usize>,
) -> Result<ColoringResult<N>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    // Start with heuristic solution
    let heuristic = dsatur_coloring(graph)?;
    let mut best = heuristic.num_colors;
    let mut best_coloring = heuristic.coloring;
    
    let upper = upper_bound.unwrap_or(best);
    
    // Try to find better colorings with backtracking
    for k in 2..best.min(upper) {
        if let Some(coloring) = try_k_coloring(graph, k) {
            best = k;
            best_coloring = coloring;
        } else {
            // k colors not possible, so k-1 is optimal
            break;
        }
    }
    
    Ok(ColoringResult::new(best_coloring))
}

/// Try to find a k-coloring using backtracking
fn try_k_coloring<G, N>(graph: &G, k: usize) -> Option<HashMap<N, usize>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let nodes: Vec<N> = graph.nodes().collect();
    let mut coloring = HashMap::new();
    
    if backtrack(graph, &nodes, 0, k, &mut coloring) {
        Some(coloring)
    } else {
        None
    }
}

/// Backtracking helper for k-coloring
fn backtrack<G, N>(
    graph: &G,
    nodes: &[N],
    idx: usize,
    k: usize,
    coloring: &mut HashMap<N, usize>,
) -> bool
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    if idx == nodes.len() {
        return true;
    }
    
    let node = &nodes[idx];
    
    // Find colors used by neighbors
    let neighbor_colors: HashSet<usize> = graph.neighbors(node)
        .filter_map(|n| coloring.get(&n).copied())
        .collect();
    
    // Try each color
    for color in 0..k {
        if !neighbor_colors.contains(&color) {
            coloring.insert(node.clone(), color);
            
            if backtrack(graph, nodes, idx + 1, k, coloring) {
                return true;
            }
            
            coloring.remove(node);
        }
    }
    
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::Graph;
    
    #[test]
    fn test_dsatur_triangle() {
        let mut graph = Graph::new();
        
        // Triangle needs 3 colors
        graph.add_edge(1, 2, None);
        graph.add_edge(2, 3, None);
        graph.add_edge(3, 1, None);
        
        let result = dsatur_coloring(&graph).unwrap();
        
        assert_eq!(result.num_colors, 3);
        assert!(result.is_valid(&graph));
    }
    
    #[test]
    fn test_dsatur_bipartite() {
        let mut graph = Graph::new();
        
        // Bipartite graph needs 2 colors
        graph.add_edge(1, 2, None);
        graph.add_edge(1, 4, None);
        graph.add_edge(3, 2, None);
        graph.add_edge(3, 4, None);
        
        let result = dsatur_coloring(&graph).unwrap();
        
        assert_eq!(result.num_colors, 2);
        assert!(result.is_valid(&graph));
    }
    
    #[test]
    fn test_dsatur_petersen() {
        let mut graph = Graph::new();
        
        // Create Petersen graph (chromatic number = 3)
        // Outer pentagon
        for i in 0..5 {
            graph.add_edge(i, (i + 1) % 5, None);
        }
        
        // Inner pentagram
        for i in 0..5 {
            graph.add_edge(i + 5, ((i + 2) % 5) + 5, None);
        }
        
        // Connections between outer and inner
        for i in 0..5 {
            graph.add_edge(i, i + 5, None);
        }
        
        let result = dsatur_coloring(&graph).unwrap();
        
        assert_eq!(result.num_colors, 3);
        assert!(result.is_valid(&graph));
    }
}