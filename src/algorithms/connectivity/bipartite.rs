//! Bipartite graph algorithms

use crate::graph::traits::GraphBase;
use crate::errors::{NetworkXError, Result};
use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::Hash;

/// Check if a graph is bipartite
pub fn is_bipartite<G, N>(graph: &G) -> Result<bool>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    match bipartite_color(graph) {
        Ok(_) => Ok(true),
        Err(NetworkXError::AlgorithmError(_)) => Ok(false),
        Err(e) => Err(e),
    }
}

/// Color nodes of a bipartite graph with two colors
/// Returns a HashMap with node colors (0 or 1)
pub fn bipartite_color<G, N>(graph: &G) -> Result<HashMap<N, usize>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let mut color: HashMap<N, usize> = HashMap::new();
    
    for start_node in graph.nodes() {
        if !color.contains_key(&start_node) {
            // BFS to color the component
            let mut queue = VecDeque::new();
            queue.push_back(start_node.clone());
            color.insert(start_node.clone(), 0);
            
            while let Some(node) = queue.pop_front() {
                let node_color = color[&node];
                let next_color = 1 - node_color;
                
                for neighbor in graph.neighbors(&node) {
                    if let Some(&neighbor_color) = color.get(&neighbor) {
                        // Check for odd cycle
                        if neighbor_color == node_color {
                            return Err(NetworkXError::AlgorithmError(
                                "Graph is not bipartite - contains odd cycle".to_string()
                            ));
                        }
                    } else {
                        color.insert(neighbor.clone(), next_color);
                        queue.push_back(neighbor);
                    }
                }
            }
        }
    }
    
    Ok(color)
}

/// Get the two sets of nodes in a bipartite graph
pub fn bipartite_sets<G, N>(graph: &G) -> Result<(HashSet<N>, HashSet<N>)>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let coloring = bipartite_color(graph)?;
    
    let mut set0 = HashSet::new();
    let mut set1 = HashSet::new();
    
    for (node, color) in coloring {
        if color == 0 {
            set0.insert(node);
        } else {
            set1.insert(node);
        }
    }
    
    Ok((set0, set1))
}

/// Find maximum matching in a bipartite graph using the Hungarian algorithm
pub fn maximum_bipartite_matching<G, N>(
    graph: &G,
    left_nodes: &HashSet<N>,
    right_nodes: &HashSet<N>,
) -> Result<HashMap<N, N>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let mut matching: HashMap<N, N> = HashMap::new();
    let mut matched_right: HashSet<N> = HashSet::new();
    
    // Simple greedy matching - can be improved with augmenting paths
    for left_node in left_nodes {
        for neighbor in graph.neighbors(left_node) {
            if right_nodes.contains(&neighbor) && !matched_right.contains(&neighbor) {
                matching.insert(left_node.clone(), neighbor.clone());
                matched_right.insert(neighbor);
                break;
            }
        }
    }
    
    // Augmenting path improvement
    let mut improved = true;
    while improved {
        improved = false;
        
        for left_node in left_nodes {
            if !matching.contains_key(left_node) {
                let mut visited = HashSet::new();
                if find_augmenting_path(
                    graph,
                    left_node,
                    right_nodes,
                    &matching,
                    &mut visited,
                    &mut matching.clone(),
                ) {
                    improved = true;
                }
            }
        }
    }
    
    Ok(matching)
}

/// Find an augmenting path for bipartite matching
fn find_augmenting_path<G, N>(
    graph: &G,
    start: &N,
    right_nodes: &HashSet<N>,
    current_matching: &HashMap<N, N>,
    visited: &mut HashSet<N>,
    new_matching: &mut HashMap<N, N>,
) -> bool
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    visited.insert(start.clone());
    
    for neighbor in graph.neighbors(start) {
        if right_nodes.contains(&neighbor) && !visited.contains(&neighbor) {
            visited.insert(neighbor.clone());
            
            // Check if neighbor is unmatched or we can find augmenting path from its match
            let matched_left = current_matching.iter()
                .find(|(_, v)| **v == neighbor)
                .map(|(k, _)| k.clone());
            
            if matched_left.is_none() || 
               (matched_left.is_some() && 
                find_augmenting_path(
                    graph,
                    &matched_left.unwrap(),
                    right_nodes,
                    current_matching,
                    visited,
                    new_matching,
                )) {
                new_matching.insert(start.clone(), neighbor);
                return true;
            }
        }
    }
    
    false
}

/// Check if a graph is complete bipartite
pub fn is_complete_bipartite<G, N>(graph: &G) -> Result<bool>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let (set0, set1) = bipartite_sets(graph)?;
    
    // Check if all nodes in set0 are connected to all nodes in set1
    for node0 in &set0 {
        for node1 in &set1 {
            if !graph.has_edge(node0, node1) {
                return Ok(false);
            }
        }
    }
    
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::Graph;
    
    #[test]
    fn test_is_bipartite() {
        let mut graph = Graph::new();
        
        // Create a bipartite graph (K_{3,3})
        graph.add_edge(1, 4, None);
        graph.add_edge(1, 5, None);
        graph.add_edge(1, 6, None);
        graph.add_edge(2, 4, None);
        graph.add_edge(2, 5, None);
        graph.add_edge(2, 6, None);
        graph.add_edge(3, 4, None);
        graph.add_edge(3, 5, None);
        graph.add_edge(3, 6, None);
        
        assert!(is_bipartite(&graph).unwrap());
        
        // Add an edge that creates an odd cycle
        graph.add_edge(1, 2, None);
        
        assert!(!is_bipartite(&graph).unwrap());
    }
    
    #[test]
    fn test_bipartite_sets() {
        let mut graph = Graph::new();
        
        // Create a simple bipartite graph
        graph.add_edge(1, 2, None);
        graph.add_edge(1, 4, None);
        graph.add_edge(3, 2, None);
        graph.add_edge(3, 4, None);
        
        let (set0, set1) = bipartite_sets(&graph).unwrap();
        
        // Check that nodes 1,3 are in one set and 2,4 in the other
        assert_eq!(set0.len(), 2);
        assert_eq!(set1.len(), 2);
        
        // Verify bipartition
        let same_set_1_3 = (set0.contains(&1) && set0.contains(&3)) || 
                           (set1.contains(&1) && set1.contains(&3));
        let same_set_2_4 = (set0.contains(&2) && set0.contains(&4)) || 
                           (set1.contains(&2) && set1.contains(&4));
        
        assert!(same_set_1_3);
        assert!(same_set_2_4);
    }
    
    #[test]
    fn test_bipartite_color() {
        let mut graph = Graph::new();
        
        // Create a path graph (always bipartite)
        graph.add_edge(1, 2, None);
        graph.add_edge(2, 3, None);
        graph.add_edge(3, 4, None);
        graph.add_edge(4, 5, None);
        
        let coloring = bipartite_color(&graph).unwrap();
        
        // Adjacent nodes should have different colors
        assert_ne!(coloring[&1], coloring[&2]);
        assert_ne!(coloring[&2], coloring[&3]);
        assert_ne!(coloring[&3], coloring[&4]);
        assert_ne!(coloring[&4], coloring[&5]);
        
        // Alternating pattern
        assert_eq!(coloring[&1], coloring[&3]);
        assert_eq!(coloring[&2], coloring[&4]);
    }
}