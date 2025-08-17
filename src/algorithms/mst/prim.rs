//! Prim's algorithm for minimum spanning tree

use super::MSTEdge;
use crate::graph::traits::GraphBase;
use crate::errors::Result;
use std::collections::{BinaryHeap, HashSet, HashMap};
use std::cmp::Ordering;
use std::hash::Hash;

/// Priority queue entry for Prim's algorithm
#[derive(Clone)]
struct PrimEntry<N> {
    node: N,
    parent: Option<N>,
    weight: f64,
}

impl<N: Clone> PrimEntry<N> {
    fn new(node: N, parent: Option<N>, weight: f64) -> Self {
        PrimEntry { node, parent, weight }
    }
}

impl<N: Clone> PartialEq for PrimEntry<N> {
    fn eq(&self, other: &Self) -> bool {
        self.weight == other.weight
    }
}

impl<N: Clone> Eq for PrimEntry<N> {}

impl<N: Clone> PartialOrd for PrimEntry<N> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // Reverse order for min-heap behavior
        other.weight.partial_cmp(&self.weight)
    }
}

impl<N: Clone> Ord for PrimEntry<N> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

/// Find minimum spanning tree using Prim's algorithm
pub fn prim_mst<G, N>(graph: &G) -> Result<Vec<MSTEdge<N>>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let nodes: Vec<N> = graph.nodes().collect();
    if nodes.is_empty() {
        return Ok(Vec::new());
    }
    
    let mut mst = Vec::new();
    let mut visited = HashSet::new();
    let mut heap = BinaryHeap::new();
    
    // Start from the first node
    let start = nodes[0].clone();
    visited.insert(start.clone());
    
    // Add all edges from the start node to the heap
    for neighbor in graph.neighbors(&start) {
        let weight = graph.get_edge_weight(&start, &neighbor).unwrap_or(1.0);
        heap.push(PrimEntry::new(neighbor, Some(start.clone()), weight));
    }
    
    // Process edges in order of increasing weight
    while let Some(entry) = heap.pop() {
        if visited.contains(&entry.node) {
            continue;
        }
        
        visited.insert(entry.node.clone());
        
        // Add edge to MST
        if let Some(parent) = entry.parent {
            mst.push(MSTEdge::new(parent, entry.node.clone(), entry.weight));
        }
        
        // Add new edges from the newly visited node
        for neighbor in graph.neighbors(&entry.node) {
            if !visited.contains(&neighbor) {
                let weight = graph.get_edge_weight(&entry.node, &neighbor).unwrap_or(1.0);
                heap.push(PrimEntry::new(neighbor, Some(entry.node.clone()), weight));
            }
        }
        
        // Early termination
        if mst.len() == nodes.len() - 1 {
            break;
        }
    }
    
    Ok(mst)
}

/// Get just the edges of the minimum spanning tree using Prim's algorithm
pub fn prim_mst_edges<G, N>(graph: &G) -> Result<Vec<(N, N, f64)>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let mst = prim_mst(graph)?;
    Ok(mst.into_iter()
        .map(|e| (e.source, e.target, e.weight))
        .collect())
}

/// Find minimum spanning tree starting from a specific node
pub fn prim_mst_from<G, N>(graph: &G, start: N) -> Result<Vec<MSTEdge<N>>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let mut mst = Vec::new();
    let mut visited = HashSet::new();
    let mut heap = BinaryHeap::new();
    
    // Start from the specified node
    visited.insert(start.clone());
    
    // Add all edges from the start node to the heap
    for neighbor in graph.neighbors(&start) {
        let weight = graph.get_edge_weight(&start, &neighbor).unwrap_or(1.0);
        heap.push(PrimEntry::new(neighbor, Some(start.clone()), weight));
    }
    
    // Process edges in order of increasing weight
    while let Some(entry) = heap.pop() {
        if visited.contains(&entry.node) {
            continue;
        }
        
        visited.insert(entry.node.clone());
        
        // Add edge to MST
        if let Some(parent) = entry.parent {
            mst.push(MSTEdge::new(parent, entry.node.clone(), entry.weight));
        }
        
        // Add new edges from the newly visited node
        for neighbor in graph.neighbors(&entry.node) {
            if !visited.contains(&neighbor) {
                let weight = graph.get_edge_weight(&entry.node, &neighbor).unwrap_or(1.0);
                heap.push(PrimEntry::new(neighbor, Some(entry.node.clone()), weight));
            }
        }
    }
    
    Ok(mst)
}

/// Find minimum spanning forest using Prim's algorithm
/// (handles disconnected graphs)
pub fn prim_mst_forest<G, N>(graph: &G) -> Result<Vec<Vec<MSTEdge<N>>>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let mut forest = Vec::new();
    let mut global_visited = HashSet::new();
    
    for node in graph.nodes() {
        if !global_visited.contains(&node) {
            let mut tree = Vec::new();
            let mut visited = HashSet::new();
            let mut heap = BinaryHeap::new();
            
            visited.insert(node.clone());
            global_visited.insert(node.clone());
            
            for neighbor in graph.neighbors(&node) {
                let weight = graph.get_edge_weight(&node, &neighbor).unwrap_or(1.0);
                heap.push(PrimEntry::new(neighbor, Some(node.clone()), weight));
            }
            
            while let Some(entry) = heap.pop() {
                if visited.contains(&entry.node) {
                    continue;
                }
                
                visited.insert(entry.node.clone());
                global_visited.insert(entry.node.clone());
                
                if let Some(parent) = entry.parent {
                    tree.push(MSTEdge::new(parent, entry.node.clone(), entry.weight));
                }
                
                for neighbor in graph.neighbors(&entry.node) {
                    if !visited.contains(&neighbor) {
                        let weight = graph.get_edge_weight(&entry.node, &neighbor).unwrap_or(1.0);
                        heap.push(PrimEntry::new(neighbor, Some(entry.node.clone()), weight));
                    }
                }
            }
            
            if !tree.is_empty() {
                forest.push(tree);
            }
        }
    }
    
    Ok(forest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::Graph;
    use crate::graph::traits::GraphBase;
    
    #[test]
    fn test_prim_mst() {
        let mut graph = Graph::new();
        
        // Create a simple weighted graph
        graph.add_edge(1, 2, Some(4.0));
        graph.add_edge(1, 3, Some(2.0));
        graph.add_edge(2, 3, Some(1.0));
        graph.add_edge(2, 4, Some(5.0));
        graph.add_edge(3, 4, Some(3.0));
        
        let mst = prim_mst(&graph).unwrap();
        
        // MST should have n-1 edges
        assert_eq!(mst.len(), 3);
        
        // Total weight should be minimal (1 + 2 + 3 = 6)
        let total_weight: f64 = mst.iter().map(|e| e.weight).sum();
        assert_eq!(total_weight, 6.0);
    }
    
    #[test]
    fn test_prim_mst_from() {
        let mut graph = Graph::new();
        
        graph.add_edge(1, 2, Some(2.0));
        graph.add_edge(1, 3, Some(3.0));
        graph.add_edge(2, 3, Some(1.0));
        graph.add_edge(2, 4, Some(4.0));
        
        let mst = prim_mst_from(&graph, 2).unwrap();
        
        assert_eq!(mst.len(), 3);
        
        let total_weight: f64 = mst.iter().map(|e| e.weight).sum();
        assert_eq!(total_weight, 7.0); // 1 + 2 + 4
    }
    
    #[test]
    fn test_prim_mst_forest() {
        let mut graph = Graph::new();
        
        // Component 1
        graph.add_edge(1, 2, Some(1.0));
        graph.add_edge(2, 3, Some(2.0));
        
        // Component 2
        graph.add_edge(4, 5, Some(3.0));
        graph.add_edge(5, 6, Some(4.0));
        
        let forest = prim_mst_forest(&graph).unwrap();
        
        // Should have 2 trees
        assert_eq!(forest.len(), 2);
        
        // Each tree should have the correct number of edges
        assert_eq!(forest[0].len(), 2);
        assert_eq!(forest[1].len(), 2);
    }
}