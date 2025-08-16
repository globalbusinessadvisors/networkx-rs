//! K-shortest paths algorithms (Yen's algorithm)

use crate::graph::traits::GraphBase;
use crate::algorithms::paths::dijkstra;
use crate::errors::{NetworkXError, Result};
use std::collections::{BinaryHeap, HashSet};
use std::hash::Hash;
use std::cmp::Ordering;

#[derive(Debug, Clone)]
struct PathCandidate<N> {
    path: Vec<N>,
    cost: f64,
}

impl<N: Clone> PartialEq for PathCandidate<N> {
    fn eq(&self, other: &Self) -> bool {
        self.cost == other.cost
    }
}

impl<N: Clone> Eq for PathCandidate<N> {}

impl<N: Clone> PartialOrd for PathCandidate<N> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.cost.partial_cmp(&other.cost)
    }
}

impl<N: Clone> Ord for PathCandidate<N> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

/// Yen's algorithm for k-shortest paths
///
/// Finds the k shortest paths between source and target
///
/// # Arguments
/// * `graph` - The graph to search
/// * `source` - Starting node
/// * `target` - Target node
/// * `k` - Number of shortest paths to find
/// * `weight_fn` - Function to get edge weights (defaults to 1.0)
///
/// # Returns
/// * `Ok(paths)` - Vector of (path, cost) tuples, sorted by cost
pub fn k_shortest_paths<G, N>(
    graph: &G,
    source: N,
    target: N,
    k: usize,
    weight_fn: Option<impl Fn(&N, &N) -> f64 + Clone>,
) -> Result<Vec<(Vec<N>, f64)>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq + Ord,
{
    if k == 0 {
        return Ok(vec![]);
    }
    
    if !graph.has_node(&source) {
        return Err(NetworkXError::NodeNotFound(format!("Source node not found")));
    }
    if !graph.has_node(&target) {
        return Err(NetworkXError::NodeNotFound(format!("Target node not found")));
    }
    
    let weight: Box<dyn Fn(&N, &N) -> f64> = if let Some(w) = weight_fn.clone() {
        Box::new(w)
    } else {
        Box::new(|_, _| 1.0)
    };
    
    // Find the shortest path first
    let first_path = dijkstra::dijkstra_path(graph, source.clone(), target.clone(), 
                                            Some(|u: &N, v: &N| weight(u, v)))?;
    
    if first_path.is_none() {
        return Ok(vec![]);
    }
    
    let (first_path, first_cost) = first_path.unwrap();
    let mut shortest_paths = vec![(first_path.clone(), first_cost)];
    
    if k == 1 {
        return Ok(shortest_paths);
    }
    
    let mut candidates = BinaryHeap::new();
    let mut explored_paths: HashSet<Vec<N>> = HashSet::new();
    explored_paths.insert(first_path.clone());
    
    // Generate alternative paths
    for path_idx in 0..shortest_paths.len() {
        if shortest_paths.len() >= k {
            break;
        }
        
        let (current_path, _) = &shortest_paths[path_idx];
        
        for i in 0..current_path.len() - 1 {
            let spur_node = &current_path[i];
            let root_path = &current_path[0..=i];
            
            // Create a modified graph for finding alternative paths
            let mut removed_edges = Vec::new();
            let mut removed_nodes = HashSet::new();
            
            // Remove edges that are part of previous shortest paths
            for (prev_path, _) in &shortest_paths {
                if prev_path.len() > i && &prev_path[0..=i] == root_path {
                    if i + 1 < prev_path.len() {
                        removed_edges.push((prev_path[i].clone(), prev_path[i + 1].clone()));
                    }
                }
            }
            
            // Remove nodes in root path except spur node
            for j in 0..i {
                removed_nodes.insert(current_path[j].clone());
            }
            
            // Find shortest path from spur node to target in modified graph
            let spur_path = find_path_avoiding(
                graph,
                spur_node.clone(),
                target.clone(),
                &removed_edges,
                &removed_nodes,
                &*weight
            );
            
            if let Some((mut spur_path, _spur_cost)) = spur_path {
                // Construct the full alternative path
                let mut total_path = root_path[0..i].to_vec();
                total_path.append(&mut spur_path);
                
                if !explored_paths.contains(&total_path) {
                    // Calculate total cost
                    let mut total_cost = 0.0;
                    for j in 0..total_path.len() - 1 {
                        total_cost += weight(&total_path[j], &total_path[j + 1]);
                    }
                    
                    candidates.push(PathCandidate {
                        path: total_path.clone(),
                        cost: total_cost,
                    });
                    explored_paths.insert(total_path);
                }
            }
        }
        
        // Add the best candidate to shortest paths
        if let Some(candidate) = candidates.pop() {
            shortest_paths.push((candidate.path, candidate.cost));
        }
    }
    
    // Sort by cost and return up to k paths
    shortest_paths.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal));
    shortest_paths.truncate(k);
    
    Ok(shortest_paths)
}

/// Find path avoiding certain edges and nodes
fn find_path_avoiding<G, N>(
    graph: &G,
    source: N,
    target: N,
    avoided_edges: &[(N, N)],
    avoided_nodes: &HashSet<N>,
    weight: &dyn Fn(&N, &N) -> f64,
) -> Option<(Vec<N>, f64)>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq + Ord,
{
    // Modified weight function that returns infinity for avoided edges
    let modified_weight = |u: &N, v: &N| -> f64 {
        if avoided_nodes.contains(u) || avoided_nodes.contains(v) {
            return f64::INFINITY;
        }
        for (avoided_u, avoided_v) in avoided_edges {
            if u == avoided_u && v == avoided_v {
                return f64::INFINITY;
            }
        }
        weight(u, v)
    };
    
    dijkstra::dijkstra_path(graph, source, target, Some(modified_weight)).ok()?
}

/// Get k shortest simple paths (no repeated nodes)
pub fn k_shortest_simple_paths<G, N>(
    graph: &G,
    source: N,
    target: N,
    k: usize,
    weight_fn: Option<impl Fn(&N, &N) -> f64 + Clone>,
) -> Result<Vec<(Vec<N>, f64)>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq + Ord,
{
    let mut all_paths = k_shortest_paths(graph, source, target, k * 2, weight_fn)?;
    
    // Filter to only simple paths (no repeated nodes)
    all_paths.retain(|(path, _)| {
        let mut seen = HashSet::new();
        for node in path {
            if !seen.insert(node.clone()) {
                return false;
            }
        }
        true
    });
    
    all_paths.truncate(k);
    Ok(all_paths)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::Graph;

    #[test]
    fn test_k_shortest_paths_simple() {
        let mut graph = Graph::new();
        graph.add_edge(0, 1, Some(1.0));
        graph.add_edge(0, 2, Some(4.0));
        graph.add_edge(1, 2, Some(2.0));
        graph.add_edge(1, 3, Some(5.0));
        graph.add_edge(2, 3, Some(1.0));
        
        let paths = k_shortest_paths(&graph, 0, 3, 3, None::<fn(&i32, &i32) -> f64>).unwrap();
        
        assert!(!paths.is_empty());
        assert!(paths.len() <= 3);
        
        // Verify paths are sorted by cost
        for i in 1..paths.len() {
            assert!(paths[i - 1].1 <= paths[i].1);
        }
    }
    
    #[test]
    fn test_k_shortest_no_path() {
        let mut graph = Graph::new();
        graph.add_node(0);
        graph.add_node(1);
        
        let paths = k_shortest_paths(&graph, 0, 1, 5, None::<fn(&i32, &i32) -> f64>).unwrap();
        assert!(paths.is_empty());
    }
    
    #[test]
    fn test_k_shortest_single_path() {
        let mut graph = Graph::new();
        graph.add_edge(0, 1, Some(1.0));
        graph.add_edge(1, 2, Some(1.0));
        
        let paths = k_shortest_paths(&graph, 0, 2, 3, None::<fn(&i32, &i32) -> f64>).unwrap();
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].0, vec![0, 1, 2]);
    }
}