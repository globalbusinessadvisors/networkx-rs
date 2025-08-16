//! Generic graph implementations that work with any node type

use super::traits::{GraphBase, GraphMut, GraphAlgorithms};
use ahash::AHashMap;
use std::hash::Hash;
use std::collections::HashSet;

/// Generic undirected graph
#[derive(Debug, Clone)]
pub struct Graph<N = i32> 
where
    N: Clone + Hash + Eq,
{
    adj: AHashMap<N, AHashMap<N, f64>>,
    nodes: HashSet<N>,
}

impl<N> Graph<N> 
where
    N: Clone + Hash + Eq,
{
    pub fn new() -> Self {
        Graph {
            adj: AHashMap::new(),
            nodes: HashSet::new(),
        }
    }
    
    pub fn add_node(&mut self, node: N) -> bool {
        self.nodes.insert(node.clone());
        self.adj.entry(node).or_insert_with(AHashMap::new);
        true
    }
    
    pub fn add_edge(&mut self, source: N, target: N, weight: Option<f64>) -> bool {
        let w = weight.unwrap_or(1.0);
        
        // Ensure nodes exist
        self.add_node(source.clone());
        self.add_node(target.clone());
        
        // Add edges in both directions for undirected graph
        self.adj.get_mut(&source).unwrap().insert(target.clone(), w);
        self.adj.get_mut(&target).unwrap().insert(source.clone(), w);
        
        true
    }
}

impl<N> Default for Graph<N>
where
    N: Clone + Hash + Eq,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<N> GraphBase for Graph<N>
where
    N: Clone + Hash + Eq,
{
    type NodeId = N;
    
    fn node_count(&self) -> usize {
        self.nodes.len()
    }
    
    fn edge_count(&self) -> usize {
        self.adj.values()
            .map(|neighbors| neighbors.len())
            .sum::<usize>() / 2  // Undirected edges are counted twice
    }
    
    fn has_node(&self, node: &Self::NodeId) -> bool {
        self.nodes.contains(node)
    }
    
    fn has_edge(&self, source: &Self::NodeId, target: &Self::NodeId) -> bool {
        self.adj.get(source)
            .map(|neighbors| neighbors.contains_key(target))
            .unwrap_or(false)
    }
    
    fn nodes(&self) -> Box<dyn Iterator<Item = Self::NodeId> + '_> {
        Box::new(self.nodes.iter().cloned())
    }
    
    fn edges(&self) -> Box<dyn Iterator<Item = (Self::NodeId, Self::NodeId, f64)> + '_> {
        Box::new(
            self.adj.iter()
                .flat_map(|(source, neighbors)| {
                    neighbors.iter().map(move |(target, &weight)| {
                        (source.clone(), target.clone(), weight)
                    })
                })
                // Return all edges for undirected graph (handled by caller if needed)
        )
    }
    
    fn neighbors(&self, node: &Self::NodeId) -> Box<dyn Iterator<Item = Self::NodeId> + '_> {
        if let Some(neighbors) = self.adj.get(node) {
            Box::new(neighbors.keys().cloned())
        } else {
            Box::new(std::iter::empty())
        }
    }
    
    fn degree(&self, node: &Self::NodeId) -> usize {
        self.adj.get(node).map(|n| n.len()).unwrap_or(0)
    }
    
    fn get_edge_weight(&self, source: &Self::NodeId, target: &Self::NodeId) -> Option<f64> {
        self.adj.get(source)?.get(target).copied()
    }
}

impl<N> GraphMut for Graph<N>
where
    N: Clone + Hash + Eq,
{
    fn add_node(&mut self, node: Self::NodeId) -> bool {
        self.add_node(node)
    }
    
    fn add_edge(&mut self, source: Self::NodeId, target: Self::NodeId, weight: Option<f64>) -> bool {
        self.add_edge(source, target, weight)
    }
    
    fn remove_edge(&mut self, source: &Self::NodeId, target: &Self::NodeId) -> bool {
        let mut removed = false;
        
        if let Some(neighbors) = self.adj.get_mut(source) {
            removed |= neighbors.remove(target).is_some();
        }
        
        if let Some(neighbors) = self.adj.get_mut(target) {
            removed |= neighbors.remove(source).is_some();
        }
        
        removed
    }
    
    fn clear(&mut self) {
        self.adj.clear();
        self.nodes.clear();
    }
}

impl<N> GraphAlgorithms for Graph<N>
where
    N: Clone + Hash + Eq,
{
    fn is_directed(&self) -> bool {
        false
    }
    
    fn to_adjacency_matrix(&self) -> Vec<Vec<Option<f64>>> {
        let nodes: Vec<_> = self.nodes.iter().cloned().collect();
        let n = nodes.len();
        let mut matrix = vec![vec![None; n]; n];
        
        let node_to_idx: AHashMap<_, _> = nodes.iter()
            .enumerate()
            .map(|(i, n)| (n.clone(), i))
            .collect();
        
        for (source, neighbors) in &self.adj {
            let i = node_to_idx[source];
            for (target, &weight) in neighbors {
                let j = node_to_idx[target];
                matrix[i][j] = Some(weight);
            }
        }
        
        matrix
    }
}

/// Generic directed graph
#[derive(Debug, Clone)]
pub struct DiGraph<N = i32>
where
    N: Clone + Hash + Eq,
{
    adj: AHashMap<N, AHashMap<N, f64>>,
    reverse_adj: AHashMap<N, HashSet<N>>,
    nodes: HashSet<N>,
}

impl<N> DiGraph<N>
where
    N: Clone + Hash + Eq,
{
    pub fn new() -> Self {
        DiGraph {
            adj: AHashMap::new(),
            reverse_adj: AHashMap::new(),
            nodes: HashSet::new(),
        }
    }
    
    pub fn add_node(&mut self, node: N) -> bool {
        self.nodes.insert(node.clone());
        self.adj.entry(node.clone()).or_insert_with(AHashMap::new);
        self.reverse_adj.entry(node).or_insert_with(HashSet::new);
        true
    }
    
    pub fn add_edge(&mut self, source: N, target: N, weight: Option<f64>) -> bool {
        let w = weight.unwrap_or(1.0);
        
        // Ensure nodes exist
        self.add_node(source.clone());
        self.add_node(target.clone());
        
        // Add directed edge
        self.adj.get_mut(&source).unwrap().insert(target.clone(), w);
        self.reverse_adj.get_mut(&target).unwrap().insert(source.clone());
        
        true
    }
    
    pub fn get_edge_weight(&self, source: &N, target: &N) -> Option<f64> {
        self.adj.get(source)?.get(target).copied()
    }
}

impl<N> Default for DiGraph<N>
where
    N: Clone + Hash + Eq,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<N> GraphBase for DiGraph<N>
where
    N: Clone + Hash + Eq,
{
    type NodeId = N;
    
    fn node_count(&self) -> usize {
        self.nodes.len()
    }
    
    fn edge_count(&self) -> usize {
        self.adj.values()
            .map(|neighbors| neighbors.len())
            .sum()
    }
    
    fn has_node(&self, node: &Self::NodeId) -> bool {
        self.nodes.contains(node)
    }
    
    fn has_edge(&self, source: &Self::NodeId, target: &Self::NodeId) -> bool {
        self.adj.get(source)
            .map(|neighbors| neighbors.contains_key(target))
            .unwrap_or(false)
    }
    
    fn nodes(&self) -> Box<dyn Iterator<Item = Self::NodeId> + '_> {
        Box::new(self.nodes.iter().cloned())
    }
    
    fn edges(&self) -> Box<dyn Iterator<Item = (Self::NodeId, Self::NodeId, f64)> + '_> {
        Box::new(
            self.adj.iter()
                .flat_map(|(source, neighbors)| {
                    neighbors.iter().map(move |(target, &weight)| {
                        (source.clone(), target.clone(), weight)
                    })
                })
        )
    }
    
    fn neighbors(&self, node: &Self::NodeId) -> Box<dyn Iterator<Item = Self::NodeId> + '_> {
        if let Some(neighbors) = self.adj.get(node) {
            Box::new(neighbors.keys().cloned())
        } else {
            Box::new(std::iter::empty())
        }
    }
    
    fn degree(&self, node: &Self::NodeId) -> usize {
        self.adj.get(node).map(|n| n.len()).unwrap_or(0)
    }
    
    fn get_edge_weight(&self, source: &Self::NodeId, target: &Self::NodeId) -> Option<f64> {
        self.adj.get(source)?.get(target).copied()
    }
}

impl<N> GraphMut for DiGraph<N>
where
    N: Clone + Hash + Eq,
{
    fn add_node(&mut self, node: Self::NodeId) -> bool {
        self.add_node(node)
    }
    
    fn add_edge(&mut self, source: Self::NodeId, target: Self::NodeId, weight: Option<f64>) -> bool {
        self.add_edge(source, target, weight)
    }
    
    fn remove_edge(&mut self, source: &Self::NodeId, target: &Self::NodeId) -> bool {
        let mut removed = false;
        
        if let Some(neighbors) = self.adj.get_mut(source) {
            removed = neighbors.remove(target).is_some();
        }
        
        if removed {
            if let Some(reverse) = self.reverse_adj.get_mut(target) {
                reverse.remove(source);
            }
        }
        
        removed
    }
    
    fn clear(&mut self) {
        self.adj.clear();
        self.reverse_adj.clear();
        self.nodes.clear();
    }
}

impl<N> GraphAlgorithms for DiGraph<N>
where
    N: Clone + Hash + Eq,
{
    fn is_directed(&self) -> bool {
        true
    }
    
    fn to_adjacency_matrix(&self) -> Vec<Vec<Option<f64>>> {
        let nodes: Vec<_> = self.nodes.iter().cloned().collect();
        let n = nodes.len();
        let mut matrix = vec![vec![None; n]; n];
        
        let node_to_idx: AHashMap<_, _> = nodes.iter()
            .enumerate()
            .map(|(i, n)| (n.clone(), i))
            .collect();
        
        for (source, neighbors) in &self.adj {
            let i = node_to_idx[source];
            for (target, &weight) in neighbors {
                let j = node_to_idx[target];
                matrix[i][j] = Some(weight);
            }
        }
        
        matrix
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_graph_creation() {
        let mut g = Graph::new();
        g.add_edge(1, 2, Some(1.5));
        g.add_edge(2, 3, None);
        
        assert_eq!(g.node_count(), 3);
        assert_eq!(g.edge_count(), 2);
        assert!(g.has_edge(&1, &2));
        assert!(g.has_edge(&2, &1)); // Undirected
        assert_eq!(g.get_edge_weight(&1, &2), Some(1.5));
    }
    
    #[test]
    fn test_digraph_creation() {
        let mut g = DiGraph::new();
        g.add_edge(1, 2, Some(1.5));
        g.add_edge(2, 3, None);
        
        assert_eq!(g.node_count(), 3);
        assert_eq!(g.edge_count(), 2);
        assert!(g.has_edge(&1, &2));
        assert!(!g.has_edge(&2, &1)); // Directed
        assert_eq!(g.get_edge_weight(&1, &2), Some(1.5));
    }
}