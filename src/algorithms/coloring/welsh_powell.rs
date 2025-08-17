//! Welsh-Powell graph coloring algorithm

use super::ColoringResult;
use crate::graph::traits::GraphBase;
use crate::errors::Result;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

/// Welsh-Powell coloring algorithm
///
/// Colors nodes in order of decreasing degree, assigning the first
/// available color to each node.
pub fn welsh_powell_coloring<G, N>(graph: &G) -> Result<ColoringResult<N>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let mut nodes: Vec<N> = graph.nodes().collect();
    
    // Sort nodes by degree (descending)
    nodes.sort_by_key(|n| std::cmp::Reverse(graph.degree(n)));
    
    let mut coloring: HashMap<N, usize> = HashMap::new();
    let mut color_classes: Vec<HashSet<N>> = Vec::new();
    
    for node in nodes {
        // Find first color class where node can be added
        let mut placed = false;
        
        for (color, class) in color_classes.iter_mut().enumerate() {
            // Check if node is adjacent to any node in this color class
            let has_neighbor_in_class = class.iter()
                .any(|n| graph.has_edge(&node, n));
            
            if !has_neighbor_in_class {
                class.insert(node.clone());
                coloring.insert(node.clone(), color);
                placed = true;
                break;
            }
        }
        
        // If not placed, create new color class
        if !placed {
            let mut new_class = HashSet::new();
            new_class.insert(node.clone());
            let color = color_classes.len();
            coloring.insert(node, color);
            color_classes.push(new_class);
        }
    }
    
    Ok(ColoringResult::new(coloring))
}