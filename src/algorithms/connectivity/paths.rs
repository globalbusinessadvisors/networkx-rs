//! Path connectivity algorithms

use crate::graph::traits::GraphBase;
use crate::errors::Result;
use std::collections::HashSet;
use std::hash::Hash;

/// Check if there is a path between two nodes
pub fn has_path<G, N>(graph: &G, source: N, target: N) -> Result<bool>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    use std::collections::{HashSet, VecDeque};
    
    if source == target {
        return Ok(true);
    }
    
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    
    queue.push_back(source);
    
    while let Some(node) = queue.pop_front() {
        if node == target {
            return Ok(true);
        }
        
        if visited.insert(node.clone()) {
            for neighbor in graph.neighbors(&node) {
                if !visited.contains(&neighbor) {
                    queue.push_back(neighbor);
                }
            }
        }
    }
    
    Ok(false)
}

/// Find node-disjoint paths between two nodes (basic implementation)
pub fn node_disjoint_paths<G, N>(
    graph: &G,
    source: N,
    target: N,
    k: Option<usize>,
) -> Result<Vec<Vec<N>>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    use std::collections::{HashSet, VecDeque, HashMap};
    
    let max_paths = k.unwrap_or(2);
    let mut paths = Vec::new();
    let mut used_nodes = HashSet::new();
    
    // Add source and target to allow them to be used in multiple paths
    used_nodes.insert(source.clone());
    used_nodes.insert(target.clone());
    
    for _ in 0..max_paths {
        // Find a path that doesn't use already used intermediate nodes
        if let Some(path) = find_path_avoiding_nodes(graph, &source, &target, &used_nodes)? {
            // Add intermediate nodes to used set
            for node in &path[1..path.len()-1] {
                used_nodes.insert(node.clone());
            }
            paths.push(path);
        } else {
            break;
        }
    }
    
    Ok(paths)
}

/// Helper function to find a path avoiding certain nodes
fn find_path_avoiding_nodes<G, N>(
    graph: &G,
    source: &N,
    target: &N,
    avoid_nodes: &HashSet<N>,
) -> Result<Option<Vec<N>>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    use std::collections::{HashSet, VecDeque, HashMap};
    
    if source == target {
        return Ok(Some(vec![source.clone()]));
    }
    
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    let mut parent: HashMap<N, N> = HashMap::new();
    
    queue.push_back(source.clone());
    visited.insert(source.clone());
    
    while let Some(node) = queue.pop_front() {
        if node == *target {
            // Reconstruct path
            let mut path = Vec::new();
            let mut current = target.clone();
            
            while current != *source {
                path.push(current.clone());
                current = parent[&current].clone();
            }
            path.push(source.clone());
            path.reverse();
            
            return Ok(Some(path));
        }
        
        for neighbor in graph.neighbors(&node) {
            if !visited.contains(&neighbor) && 
               (neighbor == *target || neighbor == *source || !avoid_nodes.contains(&neighbor)) {
                visited.insert(neighbor.clone());
                parent.insert(neighbor.clone(), node.clone());
                queue.push_back(neighbor);
            }
        }
    }
    
    Ok(None)
}

/// Find edge-disjoint paths between two nodes (basic implementation)
pub fn edge_disjoint_paths<G, N>(
    graph: &G,
    source: N,
    target: N,
    k: Option<usize>,
) -> Result<Vec<Vec<N>>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    use std::collections::{HashSet, VecDeque, HashMap};
    
    let max_paths = k.unwrap_or(2);
    let mut paths = Vec::new();
    let mut used_edges = HashSet::new();
    
    for _ in 0..max_paths {
        // Find a path that doesn't use already used edges
        if let Some(path) = find_path_avoiding_edges(graph, &source, &target, &used_edges)? {
            // Add edges to used set
            for i in 0..path.len()-1 {
                used_edges.insert((path[i].clone(), path[i+1].clone()));
                used_edges.insert((path[i+1].clone(), path[i].clone()));
            }
            paths.push(path);
        } else {
            break;
        }
    }
    
    Ok(paths)
}

/// Helper function to find a path avoiding certain edges
fn find_path_avoiding_edges<G, N>(
    graph: &G,
    source: &N,
    target: &N,
    avoid_edges: &HashSet<(N, N)>,
) -> Result<Option<Vec<N>>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    use std::collections::{HashSet, VecDeque, HashMap};
    
    if source == target {
        return Ok(Some(vec![source.clone()]));
    }
    
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    let mut parent: HashMap<N, N> = HashMap::new();
    
    queue.push_back(source.clone());
    visited.insert(source.clone());
    
    while let Some(node) = queue.pop_front() {
        if node == *target {
            // Reconstruct path
            let mut path = Vec::new();
            let mut current = target.clone();
            
            while current != *source {
                path.push(current.clone());
                current = parent[&current].clone();
            }
            path.push(source.clone());
            path.reverse();
            
            return Ok(Some(path));
        }
        
        for neighbor in graph.neighbors(&node) {
            if !visited.contains(&neighbor) && 
               !avoid_edges.contains(&(node.clone(), neighbor.clone())) {
                visited.insert(neighbor.clone());
                parent.insert(neighbor.clone(), node.clone());
                queue.push_back(neighbor);
            }
        }
    }
    
    Ok(None)
}