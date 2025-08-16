//! Dijkstra's shortest path algorithm

use crate::graph::traits::GraphBase;
use crate::errors::{NetworkXError, Result};
use std::collections::{BinaryHeap, HashMap};
use std::cmp::Ordering;
use std::hash::Hash;

#[derive(Debug, Clone)]
struct State<N> {
    cost: f64,
    node: N,
}

impl<N: Clone + PartialEq> PartialEq for State<N> {
    fn eq(&self, other: &Self) -> bool {
        self.cost == other.cost && self.node == other.node
    }
}

impl<N: Clone + PartialEq> Eq for State<N> {}

impl<N: Clone + PartialEq> PartialOrd for State<N> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.cost.partial_cmp(&self.cost)
    }
}

impl<N: Clone + PartialEq + Eq> Ord for State<N> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

/// Dijkstra's algorithm for shortest paths
pub fn dijkstra_path<G, N>(
    graph: &G,
    source: N,
    target: N,
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
    let mut distances: HashMap<N, f64> = HashMap::new();
    let mut came_from: HashMap<N, N> = HashMap::new();
    
    distances.insert(source.clone(), 0.0);
    heap.push(State { cost: 0.0, node: source.clone() });
    
    while let Some(State { cost, node }) = heap.pop() {
        if node == target {
            // Reconstruct path
            let mut path = vec![target.clone()];
            let mut current = target.clone();
            
            while let Some(prev) = came_from.get(&current) {
                if *prev == source {
                    path.push(source.clone());
                    break;
                }
                path.push(prev.clone());
                current = prev.clone();
            }
            
            path.reverse();
            return Ok(Some((path, cost)));
        }
        
        if distances.contains_key(&node) && cost > distances[&node] {
            continue;
        }
        
        for neighbor in graph.neighbors(&node) {
            let edge_weight = if let Some(ref w) = weight_fn {
                w(&node, &neighbor)
            } else {
                1.0
            };
            
            if edge_weight < 0.0 {
                return Err(NetworkXError::InvalidInput(
                    "Dijkstra's algorithm doesn't support negative weights".to_string()
                ));
            }
            
            let next_cost = cost + edge_weight;
            
            if !distances.contains_key(&neighbor) || next_cost < distances[&neighbor] {
                distances.insert(neighbor.clone(), next_cost);
                came_from.insert(neighbor.clone(), node.clone());
                heap.push(State { cost: next_cost, node: neighbor });
            }
        }
    }
    
    Ok(None)
}

/// Get all shortest distances from source using Dijkstra
pub fn dijkstra_distances<G, N>(
    graph: &G,
    source: N,
    weight_fn: Option<impl Fn(&N, &N) -> f64>,
) -> Result<HashMap<N, f64>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq + Ord,
{
    if !graph.has_node(&source) {
        return Err(NetworkXError::NodeNotFound(format!("Source node not found")));
    }
    
    let mut heap = BinaryHeap::new();
    let mut distances: HashMap<N, f64> = HashMap::new();
    
    distances.insert(source.clone(), 0.0);
    heap.push(State { cost: 0.0, node: source });
    
    while let Some(State { cost, node }) = heap.pop() {
        if distances.contains_key(&node) && cost > distances[&node] {
            continue;
        }
        
        for neighbor in graph.neighbors(&node) {
            let edge_weight = if let Some(ref w) = weight_fn {
                w(&node, &neighbor)
            } else {
                1.0
            };
            
            if edge_weight < 0.0 {
                return Err(NetworkXError::InvalidInput(
                    "Dijkstra's algorithm doesn't support negative weights".to_string()
                ));
            }
            
            let next_cost = cost + edge_weight;
            
            if !distances.contains_key(&neighbor) || next_cost < distances[&neighbor] {
                distances.insert(neighbor.clone(), next_cost);
                heap.push(State { cost: next_cost, node: neighbor });
            }
        }
    }
    
    Ok(distances)
}

/// Get all shortest paths from source using Dijkstra
pub fn dijkstra_all_paths<G, N>(
    graph: &G,
    source: N,
    weight_fn: Option<impl Fn(&N, &N) -> f64>,
) -> Result<HashMap<N, Option<(Vec<N>, f64)>>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq + Ord,
{
    if !graph.has_node(&source) {
        return Err(NetworkXError::NodeNotFound(format!("Source node not found")));
    }
    
    let mut heap = BinaryHeap::new();
    let mut distances: HashMap<N, f64> = HashMap::new();
    let mut came_from: HashMap<N, N> = HashMap::new();
    
    distances.insert(source.clone(), 0.0);
    heap.push(State { cost: 0.0, node: source.clone() });
    
    while let Some(State { cost, node }) = heap.pop() {
        if distances.contains_key(&node) && cost > distances[&node] {
            continue;
        }
        
        for neighbor in graph.neighbors(&node) {
            let edge_weight = if let Some(ref w) = weight_fn {
                w(&node, &neighbor)
            } else {
                1.0
            };
            
            if edge_weight < 0.0 {
                return Err(NetworkXError::InvalidInput(
                    "Dijkstra's algorithm doesn't support negative weights".to_string()
                ));
            }
            
            let next_cost = cost + edge_weight;
            
            if !distances.contains_key(&neighbor) || next_cost < distances[&neighbor] {
                distances.insert(neighbor.clone(), next_cost);
                came_from.insert(neighbor.clone(), node.clone());
                heap.push(State { cost: next_cost, node: neighbor.clone() });
            }
        }
    }
    
    // Reconstruct all paths
    let mut paths = HashMap::new();
    for (target, &distance) in &distances {
        if *target == source {
            paths.insert(source.clone(), Some((vec![source.clone()], 0.0)));
        } else {
            let mut path = vec![target.clone()];
            let mut current = target.clone();
            
            while let Some(prev) = came_from.get(&current) {
                if *prev == source {
                    path.push(source.clone());
                    break;
                }
                path.push(prev.clone());
                current = prev.clone();
            }
            
            path.reverse();
            paths.insert(target.clone(), Some((path, distance)));
        }
    }
    
    Ok(paths)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::Graph;

    #[test]
    fn test_dijkstra_simple_path() {
        let mut graph = Graph::new();
        graph.add_edge(0, 1, Some(1.0));
        graph.add_edge(1, 2, Some(2.0));
        graph.add_edge(0, 2, Some(4.0));
        graph.add_edge(2, 3, Some(1.0));
        graph.add_edge(3, 4, Some(3.0));
        
        // With no weight function, uses edge weight = 1.0 for all edges
        let result = dijkstra_path(&graph, 0, 4, None::<fn(&i32, &i32) -> f64>).unwrap();
        assert!(result.is_some());
        let (path, cost) = result.unwrap();
        // Shortest path by hop count
        assert_eq!(path, vec![0, 2, 3, 4]);
        assert_eq!(cost, 3.0);
    }
    
    #[test]
    fn test_dijkstra_distances() {
        let mut graph = Graph::new();
        graph.add_edge(0, 1, Some(1.0));
        graph.add_edge(1, 2, Some(2.0));
        graph.add_edge(0, 2, Some(4.0));
        
        let distances = dijkstra_distances(&graph, 0, None::<fn(&i32, &i32) -> f64>).unwrap();
        assert_eq!(distances[&0], 0.0);
        assert_eq!(distances[&1], 1.0);
        assert_eq!(distances[&2], 1.0); // Direct edge from 0 to 2 (all edges weight=1)
    }
    
    #[test]
    fn test_dijkstra_no_path() {
        let mut graph = Graph::new();
        graph.add_node(0);
        graph.add_node(1);
        
        let result = dijkstra_path(&graph, 0, 1, None::<fn(&i32, &i32) -> f64>).unwrap();
        assert!(result.is_none());
    }
}