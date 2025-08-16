//! A* search algorithm for shortest paths with heuristics

use crate::graph::traits::GraphBase;
use crate::errors::{NetworkXError, Result};
use std::collections::{BinaryHeap, HashMap};
use std::cmp::Ordering;
use std::hash::Hash;

#[derive(Debug, Clone)]
struct State<N> {
    cost: f64,
    node: N,
    f_score: f64,
}

impl<N: Clone + PartialEq> PartialEq for State<N> {
    fn eq(&self, other: &Self) -> bool {
        self.f_score == other.f_score && self.node == other.node
    }
}

impl<N: Clone + PartialEq> Eq for State<N> {}

impl<N: Clone + PartialEq> PartialOrd for State<N> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.f_score.partial_cmp(&self.f_score)
    }
}

impl<N: Clone + PartialEq + Eq> Ord for State<N> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

/// A* search algorithm for finding shortest path with heuristic
///
/// # Arguments
/// * `graph` - The graph to search
/// * `source` - Starting node
/// * `target` - Target node
/// * `heuristic` - Heuristic function estimating distance to target
/// * `weight_fn` - Function to get edge weights (defaults to 1.0)
///
/// # Returns
/// * `Ok(Some((path, cost)))` - Path and total cost if found
/// * `Ok(None)` - No path exists
/// * `Err` - If source or target not in graph
pub fn astar_path<G, N>(
    graph: &G,
    source: N,
    target: N,
    heuristic: impl Fn(&N) -> f64,
    weight_fn: Option<impl Fn(&N, &N) -> f64>,
) -> Result<Option<(Vec<N>, f64)>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq + Ord,
{
    if !graph.has_node(&source) {
        return Err(NetworkXError::NodeNotFound(format!("Source node not found")));
    }
    if !graph.has_node(&target) {
        return Err(NetworkXError::NodeNotFound(format!("Target node not found")));
    }

    
    let mut heap = BinaryHeap::new();
    let mut g_scores: HashMap<N, f64> = HashMap::new();
    let mut came_from: HashMap<N, N> = HashMap::new();
    let mut closed_set: HashMap<N, bool> = HashMap::new();
    
    g_scores.insert(source.clone(), 0.0);
    heap.push(State {
        cost: 0.0,
        node: source.clone(),
        f_score: heuristic(&source),
    });
    
    while let Some(State { cost, node, .. }) = heap.pop() {
        if node == target {
            // Reconstruct path
            let mut path = vec![target.clone()];
            let mut current = target;
            
            while let Some(prev) = came_from.get(&current) {
                path.push(prev.clone());
                current = prev.clone();
            }
            
            path.reverse();
            return Ok(Some((path, cost)));
        }
        
        if closed_set.contains_key(&node) {
            continue;
        }
        closed_set.insert(node.clone(), true);
        
        for neighbor in graph.neighbors(&node) {
            if closed_set.contains_key(&neighbor) {
                continue;
            }
            
            let tentative_g = cost + if let Some(ref w) = weight_fn {
                w(&node, &neighbor)
            } else {
                1.0
            };
            
            if !g_scores.contains_key(&neighbor) || tentative_g < g_scores[&neighbor] {
                g_scores.insert(neighbor.clone(), tentative_g);
                came_from.insert(neighbor.clone(), node.clone());
                
                heap.push(State {
                    cost: tentative_g,
                    node: neighbor.clone(),
                    f_score: tentative_g + heuristic(&neighbor),
                });
            }
        }
    }
    
    Ok(None)
}

/// A* search algorithm returning distance only
pub fn astar_distance<G, N>(
    graph: &G,
    source: N,
    target: N,
    heuristic: impl Fn(&N) -> f64,
    weight_fn: Option<impl Fn(&N, &N) -> f64>,
) -> Result<Option<f64>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq + Ord,
{
    astar_path(graph, source, target, heuristic, weight_fn)
        .map(|result| result.map(|(_, cost)| cost))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::Graph;

    #[test]
    fn test_astar_simple() {
        let mut graph = Graph::new();
        graph.add_edge(0, 1, Some(1.0));
        graph.add_edge(1, 2, Some(2.0));
        graph.add_edge(0, 2, Some(4.0));
        
        // Simple heuristic - always 0
        let result = astar_path(&graph, 0, 2, |_| 0.0, None::<fn(&i32, &i32) -> f64>).unwrap();
        assert!(result.is_some());
        let (path, cost) = result.unwrap();
        assert_eq!(path, vec![0, 2]); // Direct path (all edges weight=1)
        assert_eq!(cost, 1.0);
    }
    
    #[test]
    fn test_astar_no_path() {
        let mut graph = Graph::new();
        graph.add_node(0);
        graph.add_node(1);
        
        let result = astar_path(&graph, 0, 1, |_| 0.0, None::<fn(&i32, &i32) -> f64>).unwrap();
        assert!(result.is_none());
    }
}