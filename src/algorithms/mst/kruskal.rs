//! Kruskal's algorithm for minimum spanning tree

use super::MSTEdge;
use crate::graph::traits::GraphBase;
use crate::errors::Result;
use std::collections::HashMap;
use std::hash::Hash;

/// Union-Find data structure for Kruskal's algorithm
struct UnionFind<N: Clone + Hash + Eq> {
    parent: HashMap<N, N>,
    rank: HashMap<N, usize>,
}

impl<N: Clone + Hash + Eq> UnionFind<N> {
    fn new() -> Self {
        UnionFind {
            parent: HashMap::new(),
            rank: HashMap::new(),
        }
    }
    
    fn make_set(&mut self, x: N) {
        if !self.parent.contains_key(&x) {
            self.parent.insert(x.clone(), x.clone());
            self.rank.insert(x, 0);
        }
    }
    
    fn find(&mut self, x: &N) -> N {
        if !self.parent.contains_key(x) {
            self.make_set(x.clone());
            return x.clone();
        }
        
        let parent = self.parent[x].clone();
        if parent != *x {
            let root = self.find(&parent);
            self.parent.insert(x.clone(), root.clone());
            root
        } else {
            x.clone()
        }
    }
    
    fn union(&mut self, x: &N, y: &N) -> bool {
        let root_x = self.find(x);
        let root_y = self.find(y);
        
        if root_x == root_y {
            return false; // Already in same set
        }
        
        let rank_x = self.rank[&root_x];
        let rank_y = self.rank[&root_y];
        
        if rank_x < rank_y {
            self.parent.insert(root_x, root_y);
        } else if rank_x > rank_y {
            self.parent.insert(root_y, root_x);
        } else {
            self.parent.insert(root_y, root_x.clone());
            self.rank.insert(root_x, rank_x + 1);
        }
        
        true
    }
}

/// Find minimum spanning tree using Kruskal's algorithm
pub fn kruskal_mst<G, N>(graph: &G) -> Result<Vec<MSTEdge<N>>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let mut edges: Vec<MSTEdge<N>> = Vec::new();
    
    // Collect all edges
    for (source, target, weight) in graph.edges() {
        edges.push(MSTEdge::new(source, target, weight));
    }
    
    // Sort edges by weight
    edges.sort_by(|a, b| a.weight.partial_cmp(&b.weight).unwrap());
    
    let mut mst = Vec::new();
    let mut uf = UnionFind::new();
    
    // Initialize all nodes in union-find
    for node in graph.nodes() {
        uf.make_set(node);
    }
    
    // Process edges in order of increasing weight
    for edge in edges {
        if uf.union(&edge.source, &edge.target) {
            mst.push(edge);
            
            // Early termination when we have n-1 edges
            if mst.len() == graph.node_count() - 1 {
                break;
            }
        }
    }
    
    Ok(mst)
}

/// Get just the edges of the minimum spanning tree
pub fn kruskal_mst_edges<G, N>(graph: &G) -> Result<Vec<(N, N, f64)>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let mst = kruskal_mst(graph)?;
    Ok(mst.into_iter()
        .map(|e| (e.source, e.target, e.weight))
        .collect())
}

/// Find maximum spanning tree using modified Kruskal's algorithm
pub fn kruskal_maximum_spanning_tree<G, N>(graph: &G) -> Result<Vec<MSTEdge<N>>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let mut edges: Vec<MSTEdge<N>> = Vec::new();
    
    // Collect all edges
    for (source, target, weight) in graph.edges() {
        edges.push(MSTEdge::new(source, target, weight));
    }
    
    // Sort edges by weight in descending order
    edges.sort_by(|a, b| b.weight.partial_cmp(&a.weight).unwrap());
    
    let mut mst = Vec::new();
    let mut uf = UnionFind::new();
    
    // Initialize all nodes in union-find
    for node in graph.nodes() {
        uf.make_set(node);
    }
    
    // Process edges in order of decreasing weight
    for edge in edges {
        if uf.union(&edge.source, &edge.target) {
            mst.push(edge);
            
            // Early termination when we have n-1 edges
            if mst.len() == graph.node_count() - 1 {
                break;
            }
        }
    }
    
    Ok(mst)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::Graph;
    
    #[test]
    fn test_kruskal_mst() {
        let mut graph = Graph::new();
        
        // Create a simple weighted graph
        graph.add_edge(1, 2, Some(4.0));
        graph.add_edge(1, 3, Some(2.0));
        graph.add_edge(2, 3, Some(1.0));
        graph.add_edge(2, 4, Some(5.0));
        graph.add_edge(3, 4, Some(3.0));
        
        let mst = kruskal_mst(&graph).unwrap();
        
        // MST should have n-1 edges
        assert_eq!(mst.len(), 3);
        
        // Total weight should be minimal (1 + 2 + 3 = 6)
        let total_weight: f64 = mst.iter().map(|e| e.weight).sum();
        assert_eq!(total_weight, 6.0);
    }
    
    #[test]
    fn test_union_find() {
        let mut uf = UnionFind::new();
        
        uf.make_set(1);
        uf.make_set(2);
        uf.make_set(3);
        
        assert_ne!(uf.find(&1), uf.find(&2));
        
        uf.union(&1, &2);
        assert_eq!(uf.find(&1), uf.find(&2));
        assert_ne!(uf.find(&1), uf.find(&3));
        
        uf.union(&2, &3);
        assert_eq!(uf.find(&1), uf.find(&3));
    }
    
    #[test]
    fn test_kruskal_disconnected() {
        let mut graph = Graph::new();
        
        // Component 1
        graph.add_edge(1, 2, Some(1.0));
        graph.add_edge(2, 3, Some(2.0));
        
        // Component 2 (disconnected)
        graph.add_edge(4, 5, Some(3.0));
        
        let mst = kruskal_mst(&graph).unwrap();
        
        // MST should include all edges since there are no cycles
        assert_eq!(mst.len(), 3);
        
        let total_weight: f64 = mst.iter().map(|e| e.weight).sum();
        assert_eq!(total_weight, 6.0);
    }
}