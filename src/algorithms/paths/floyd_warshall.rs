//! Floyd-Warshall algorithm for all-pairs shortest paths

use crate::graph::traits::GraphBase;
use crate::errors::{NetworkXError, Result};
use std::collections::HashMap;
use std::hash::Hash;

/// Floyd-Warshall algorithm for all-pairs shortest paths
///
/// Computes shortest paths between all pairs of nodes
///
/// # Arguments
/// * `graph` - The graph to analyze
/// * `weight_fn` - Function to get edge weights (defaults to 1.0)
///
/// # Returns
/// * `Ok((distances, next))` - Distance matrix and next-hop matrix for path reconstruction
/// * `Err` - If negative cycle detected
pub fn floyd_warshall<G, N>(
    graph: &G,
    weight_fn: Option<impl Fn(&N, &N) -> f64>,
) -> Result<(HashMap<(N, N), f64>, HashMap<(N, N), N>)>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq + Ord,
{
    
    let nodes: Vec<N> = graph.nodes().collect();
    let mut dist: HashMap<(N, N), f64> = HashMap::new();
    let mut next: HashMap<(N, N), N> = HashMap::new();
    
    // Initialize distances
    for u in &nodes {
        for v in &nodes {
            if u == v {
                dist.insert((u.clone(), v.clone()), 0.0);
            } else if graph.has_edge(u, v) {
                let w = if let Some(ref wf) = weight_fn {
                    wf(u, v)
                } else {
                    graph.get_edge_weight(u, v).unwrap_or(1.0)
                };
                dist.insert((u.clone(), v.clone()), w);
                next.insert((u.clone(), v.clone()), v.clone());
            } else {
                dist.insert((u.clone(), v.clone()), f64::INFINITY);
            }
        }
    }
    
    // Main algorithm
    for k in &nodes {
        for i in &nodes {
            for j in &nodes {
                let dist_ik = dist[&(i.clone(), k.clone())];
                let dist_kj = dist[&(k.clone(), j.clone())];
                let dist_ij = dist[&(i.clone(), j.clone())];
                
                if dist_ik != f64::INFINITY && dist_kj != f64::INFINITY {
                    let new_dist = dist_ik + dist_kj;
                    if new_dist < dist_ij {
                        dist.insert((i.clone(), j.clone()), new_dist);
                        next.insert((i.clone(), j.clone()), 
                                   next[&(i.clone(), k.clone())].clone());
                    }
                }
            }
        }
    }
    
    // Check for negative cycles
    for u in &nodes {
        if dist[&(u.clone(), u.clone())] < 0.0 {
            return Err(NetworkXError::NegativeCycle(
                "Graph contains negative cycle".to_string()
            ));
        }
    }
    
    Ok((dist, next))
}

/// Reconstruct path from Floyd-Warshall result
pub fn reconstruct_path<N>(
    source: &N,
    target: &N,
    next: &HashMap<(N, N), N>,
) -> Option<Vec<N>>
where
    N: Clone + Hash + Eq,
{
    if !next.contains_key(&(source.clone(), target.clone())) {
        return None;
    }
    
    let mut path = vec![source.clone()];
    let mut current = source.clone();
    
    while current != *target {
        if let Some(next_node) = next.get(&(current.clone(), target.clone())) {
            current = next_node.clone();
            path.push(current.clone());
        } else {
            return None;
        }
    }
    
    Some(path)
}

/// Get all shortest paths from Floyd-Warshall result
pub fn all_pairs_shortest_paths<G, N>(
    graph: &G,
    weight_fn: Option<impl Fn(&N, &N) -> f64>,
) -> Result<HashMap<(N, N), Option<(Vec<N>, f64)>>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq + Ord,
{
    let (dist, next) = floyd_warshall(graph, weight_fn)?;
    let nodes: Vec<N> = graph.nodes().collect();
    let mut paths = HashMap::new();
    
    for u in &nodes {
        for v in &nodes {
            let distance = dist[&(u.clone(), v.clone())];
            if distance != f64::INFINITY {
                if let Some(path) = reconstruct_path(u, v, &next) {
                    paths.insert((u.clone(), v.clone()), Some((path, distance)));
                } else {
                    paths.insert((u.clone(), v.clone()), None);
                }
            } else {
                paths.insert((u.clone(), v.clone()), None);
            }
        }
    }
    
    Ok(paths)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::DiGraph;

    #[test]
    fn test_floyd_warshall_simple() {
        let mut graph = DiGraph::new();
        graph.add_edge(0, 1, Some(3.0));
        graph.add_edge(0, 2, Some(8.0));
        graph.add_edge(1, 2, Some(2.0));
        graph.add_edge(2, 3, Some(1.0));
        graph.add_edge(1, 3, Some(5.0));
        
        let (dist, _) = floyd_warshall(&graph, Some(|u: &i32, v: &i32| {
            graph.get_edge_weight(u, v).unwrap_or(1.0)
        })).unwrap();
        
        assert_eq!(dist[&(0, 0)], 0.0);
        assert_eq!(dist[&(0, 1)], 3.0);
        assert_eq!(dist[&(0, 2)], 5.0);  // Through node 1
        assert_eq!(dist[&(0, 3)], 6.0);  // Through nodes 1, 2
    }
    
    #[test]
    fn test_floyd_warshall_disconnected() {
        let mut graph = DiGraph::new();
        graph.add_node(0);
        graph.add_node(1);
        graph.add_edge(0, 2, Some(1.0));
        
        let (dist, _) = floyd_warshall(&graph, None::<fn(&i32, &i32) -> f64>).unwrap();
        
        assert_eq!(dist[&(0, 1)], f64::INFINITY);
        assert_eq!(dist[&(1, 0)], f64::INFINITY);
        assert_eq!(dist[&(0, 2)], 1.0);
    }
    
    #[test]
    fn test_path_reconstruction() {
        let mut graph = DiGraph::new();
        graph.add_edge(0, 1, Some(1.0));
        graph.add_edge(1, 2, Some(1.0));
        graph.add_edge(2, 3, Some(1.0));
        
        let (_, next) = floyd_warshall(&graph, None::<fn(&i32, &i32) -> f64>).unwrap();
        let path = reconstruct_path(&0, &3, &next).unwrap();
        
        assert_eq!(path, vec![0, 1, 2, 3]);
    }
}