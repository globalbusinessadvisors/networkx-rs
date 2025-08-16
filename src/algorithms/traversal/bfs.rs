//! Breadth-first search traversal

use crate::graph::traits::GraphBase;
use std::collections::{HashSet, VecDeque, HashMap};
use std::hash::Hash;

/// Breadth-first search edge generator
pub fn bfs_edges<G, N>(graph: &G, source: N) -> Vec<(N, N)>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let mut edges = Vec::new();
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    
    visited.insert(source.clone());
    queue.push_back(source);
    
    while let Some(current) = queue.pop_front() {
        for neighbor in graph.neighbors(&current) {
            if !visited.contains(&neighbor) {
                visited.insert(neighbor.clone());
                edges.push((current.clone(), neighbor.clone()));
                queue.push_back(neighbor);
            }
        }
    }
    
    edges
}

/// BFS tree from source
pub fn bfs_tree<G, N>(graph: &G, source: N) -> HashMap<N, Vec<N>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let mut tree: HashMap<N, Vec<N>> = HashMap::new();
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    
    visited.insert(source.clone());
    queue.push_back(source.clone());
    tree.insert(source, Vec::new());
    
    while let Some(current) = queue.pop_front() {
        let mut children = Vec::new();
        
        for neighbor in graph.neighbors(&current) {
            if !visited.contains(&neighbor) {
                visited.insert(neighbor.clone());
                queue.push_back(neighbor.clone());
                children.push(neighbor);
            }
        }
        
        if !children.is_empty() {
            tree.insert(current, children);
        }
    }
    
    tree
}

/// BFS predecessors
pub fn bfs_predecessors<G, N>(graph: &G, source: N) -> HashMap<N, N>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let mut predecessors = HashMap::new();
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    
    visited.insert(source.clone());
    queue.push_back(source);
    
    while let Some(current) = queue.pop_front() {
        for neighbor in graph.neighbors(&current) {
            if !visited.contains(&neighbor) {
                visited.insert(neighbor.clone());
                predecessors.insert(neighbor.clone(), current.clone());
                queue.push_back(neighbor);
            }
        }
    }
    
    predecessors
}

/// BFS successors
pub fn bfs_successors<G, N>(graph: &G, source: N) -> HashMap<N, Vec<N>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let mut successors: HashMap<N, Vec<N>> = HashMap::new();
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    
    visited.insert(source.clone());
    queue.push_back(source);
    
    while let Some(current) = queue.pop_front() {
        let mut current_successors = Vec::new();
        
        for neighbor in graph.neighbors(&current) {
            if !visited.contains(&neighbor) {
                visited.insert(neighbor.clone());
                current_successors.push(neighbor.clone());
                queue.push_back(neighbor);
            }
        }
        
        if !current_successors.is_empty() {
            successors.insert(current, current_successors);
        }
    }
    
    successors
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::Graph;
    
    #[test]
    fn test_bfs_edges() {
        let mut graph = Graph::new();
        graph.add_edge(0, 1, None);
        graph.add_edge(1, 2, None);
        graph.add_edge(0, 3, None);
        
        let edges = bfs_edges(&graph, 0);
        assert_eq!(edges.len(), 3);
    }
    
    #[test]
    fn test_bfs_tree() {
        let mut graph = Graph::new();
        graph.add_edge(0, 1, None);
        graph.add_edge(1, 2, None);
        graph.add_edge(0, 3, None);
        
        let tree = bfs_tree(&graph, 0);
        assert!(tree.contains_key(&0));
    }
}