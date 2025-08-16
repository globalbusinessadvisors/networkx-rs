//! Depth-first search traversal

use crate::graph::traits::GraphBase;
use std::collections::HashSet;
use std::hash::Hash;

/// Depth-first search traversal
pub fn dfs_edges<G, N>(graph: &G, source: N) -> Vec<(N, N)>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let mut edges = Vec::new();
    let mut visited = HashSet::new();
    let mut stack = vec![(source.clone(), None)];
    
    while let Some((node, parent)) = stack.pop() {
        if visited.contains(&node) {
            continue;
        }
        
        visited.insert(node.clone());
        
        if let Some(p) = parent {
            edges.push((p, node.clone()));
        }
        
        for neighbor in graph.neighbors(&node) {
            if !visited.contains(&neighbor) {
                stack.push((neighbor, Some(node.clone())));
            }
        }
    }
    
    edges
}

/// DFS tree from source
pub fn dfs_tree<G, N>(graph: &G, source: N) -> Vec<N>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let mut nodes = Vec::new();
    let mut visited = HashSet::new();
    let mut stack = vec![source.clone()];
    
    while let Some(node) = stack.pop() {
        if visited.contains(&node) {
            continue;
        }
        
        visited.insert(node.clone());
        nodes.push(node.clone());
        
        for neighbor in graph.neighbors(&node) {
            if !visited.contains(&neighbor) {
                stack.push(neighbor);
            }
        }
    }
    
    nodes
}

/// DFS predecessors
pub fn dfs_predecessors<G, N>(graph: &G, source: N) -> Vec<(N, N)>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let mut predecessors = Vec::new();
    let mut visited = HashSet::new();
    let mut stack = vec![(source.clone(), None)];
    
    while let Some((node, parent)) = stack.pop() {
        if visited.contains(&node) {
            continue;
        }
        
        visited.insert(node.clone());
        
        if let Some(p) = parent {
            predecessors.push((node.clone(), p));
        }
        
        for neighbor in graph.neighbors(&node) {
            if !visited.contains(&neighbor) {
                stack.push((neighbor, Some(node.clone())));
            }
        }
    }
    
    predecessors
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::Graph;
    
    #[test]
    fn test_dfs_tree() {
        let mut graph = Graph::new();
        graph.add_edge(0, 1, None);
        graph.add_edge(1, 2, None);
        graph.add_edge(0, 3, None);
        
        let tree = dfs_tree(&graph, 0);
        assert!(tree.contains(&0));
        assert!(tree.contains(&1));
        assert!(tree.contains(&2));
        assert!(tree.contains(&3));
    }
}