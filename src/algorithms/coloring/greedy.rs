//! Greedy graph coloring algorithms

use super::ColoringResult;
use crate::graph::traits::GraphBase;
use crate::errors::Result;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

/// Color a graph using a greedy algorithm
///
/// The greedy algorithm assigns colors to nodes in order, using the
/// smallest available color that doesn't conflict with neighbors.
pub fn greedy_color<G, N>(
    graph: &G,
    strategy: ColoringStrategy,
) -> Result<ColoringResult<N>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let nodes = order_nodes(graph, strategy);
    let mut coloring: HashMap<N, usize> = HashMap::new();
    
    for node in nodes {
        // Find colors used by neighbors
        let mut neighbor_colors = HashSet::new();
        for neighbor in graph.neighbors(&node) {
            if let Some(&color) = coloring.get(&neighbor) {
                neighbor_colors.insert(color);
            }
        }
        
        // Find the smallest available color
        let mut color = 0;
        while neighbor_colors.contains(&color) {
            color += 1;
        }
        
        coloring.insert(node, color);
    }
    
    Ok(ColoringResult::new(coloring))
}

/// Strategy for ordering nodes in greedy coloring
#[derive(Debug, Clone, Copy)]
pub enum ColoringStrategy {
    /// Color nodes in the order they appear
    Default,
    /// Color nodes with highest degree first
    LargestFirst,
    /// Color nodes with lowest degree first
    SmallestLast,
    /// Random order (requires seed)
    Random(u64),
    /// Color nodes in order of saturation degree
    Saturation,
}

/// Order nodes according to the given strategy
fn order_nodes<G, N>(graph: &G, strategy: ColoringStrategy) -> Vec<N>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let mut nodes: Vec<N> = graph.nodes().collect();
    
    match strategy {
        ColoringStrategy::Default => nodes,
        ColoringStrategy::LargestFirst => {
            nodes.sort_by_key(|n| std::cmp::Reverse(graph.degree(n)));
            nodes
        }
        ColoringStrategy::SmallestLast => {
            // Build the ordering by repeatedly removing minimum degree nodes
            let mut result = Vec::new();
            let mut remaining: HashSet<N> = nodes.into_iter().collect();
            let mut temp_graph_degrees: HashMap<N, usize> = HashMap::new();
            
            // Initialize degrees
            for node in &remaining {
                let degree = graph.neighbors(node)
                    .filter(|n| remaining.contains(n))
                    .count();
                temp_graph_degrees.insert(node.clone(), degree);
            }
            
            while !remaining.is_empty() {
                // Find node with minimum degree
                let min_node = remaining
                    .iter()
                    .min_by_key(|n| temp_graph_degrees[n])
                    .cloned()
                    .unwrap();
                
                // Remove it and update degrees
                remaining.remove(&min_node);
                for neighbor in graph.neighbors(&min_node) {
                    if let Some(degree) = temp_graph_degrees.get_mut(&neighbor) {
                        *degree = degree.saturating_sub(1);
                    }
                }
                
                result.push(min_node);
            }
            
            result.reverse(); // We want smallest last, not first
            result
        }
        ColoringStrategy::Random(seed) => {
            use rand::{SeedableRng, seq::SliceRandom};
            use rand_chacha::ChaCha8Rng;
            
            let mut rng = ChaCha8Rng::seed_from_u64(seed);
            nodes.shuffle(&mut rng);
            nodes
        }
        ColoringStrategy::Saturation => {
            // This is actually DSATUR, implemented separately
            nodes
        }
    }
}

/// Greedy coloring with interchange
///
/// After initial greedy coloring, try to reduce colors by interchanging
pub fn greedy_color_with_interchange<G, N>(
    graph: &G,
    max_iterations: usize,
) -> Result<ColoringResult<N>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let mut result = greedy_color(graph, ColoringStrategy::LargestFirst)?;
    
    for _ in 0..max_iterations {
        let improved = try_interchange(graph, &mut result);
        if !improved {
            break;
        }
    }
    
    Ok(result)
}

/// Try to improve coloring by interchanging colors
fn try_interchange<G, N>(graph: &G, result: &mut ColoringResult<N>) -> bool
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    // Try to eliminate the highest color
    let max_color = result.num_colors - 1;
    if max_color == 0 {
        return false;
    }
    
    // Find nodes with the highest color
    let nodes_with_max: Vec<N> = result.coloring
        .iter()
        .filter(|(_, &c)| c == max_color)
        .map(|(n, _)| n.clone())
        .collect();
    
    for node in nodes_with_max {
        // Try to recolor this node with a lower color
        let mut neighbor_colors = HashSet::new();
        for neighbor in graph.neighbors(&node) {
            if let Some(&color) = result.coloring.get(&neighbor) {
                neighbor_colors.insert(color);
            }
        }
        
        for new_color in 0..max_color {
            if !neighbor_colors.contains(&new_color) {
                result.coloring.insert(node, new_color);
                result.num_colors = result.coloring.values().max().copied().unwrap_or(0) + 1;
                return true;
            }
        }
    }
    
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::Graph;
    
    #[test]
    fn test_greedy_color_simple() {
        let mut graph = Graph::new();
        
        // Create a triangle
        graph.add_edge(1, 2, None);
        graph.add_edge(2, 3, None);
        graph.add_edge(3, 1, None);
        
        let result = greedy_color(&graph, ColoringStrategy::Default).unwrap();
        
        // Triangle needs 3 colors
        assert_eq!(result.num_colors, 3);
        assert!(result.is_valid(&graph));
    }
    
    #[test]
    fn test_greedy_color_bipartite() {
        let mut graph = Graph::new();
        
        // Create a bipartite graph
        graph.add_edge(1, 2, None);
        graph.add_edge(1, 4, None);
        graph.add_edge(3, 2, None);
        graph.add_edge(3, 4, None);
        
        let result = greedy_color(&graph, ColoringStrategy::LargestFirst).unwrap();
        
        // Bipartite graph needs only 2 colors
        assert_eq!(result.num_colors, 2);
        assert!(result.is_valid(&graph));
    }
    
    #[test]
    fn test_coloring_strategies() {
        let mut graph = Graph::new();
        
        // Create a more complex graph
        for i in 1..=5 {
            for j in (i+1)..=5 {
                if (i + j) % 2 == 0 {
                    graph.add_edge(i, j, None);
                }
            }
        }
        
        let default = greedy_color(&graph, ColoringStrategy::Default).unwrap();
        let largest = greedy_color(&graph, ColoringStrategy::LargestFirst).unwrap();
        let smallest = greedy_color(&graph, ColoringStrategy::SmallestLast).unwrap();
        
        // All should produce valid colorings
        assert!(default.is_valid(&graph));
        assert!(largest.is_valid(&graph));
        assert!(smallest.is_valid(&graph));
    }
}