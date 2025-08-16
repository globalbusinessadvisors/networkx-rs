//! Core graph data structures optimized for performance

use ahash::AHashMap;

pub mod traits;
pub mod generic;

pub use generic::{Graph, DiGraph};
pub use traits::{GraphBase, GraphMut, GraphAlgorithms};

/// Node identifier type
pub type Node = usize;

/// Edge weight type
pub type Weight = f64;

/// Edge representation
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Edge {
    pub source: Node,
    pub target: Node,
    pub weight: Weight,
}

impl Edge {
    pub fn new(source: Node, target: Node, weight: Weight) -> Self {
        Edge { source, target, weight }
    }
    
    pub fn unweighted(source: Node, target: Node) -> Self {
        Edge { source, target, weight: 1.0 }
    }
}

/// Common graph storage using adjacency list with fast lookup
#[derive(Debug, Clone)]
pub struct AdjacencyList {
    /// Node to neighbors mapping with weights
    adj: Vec<AHashMap<Node, Weight>>,
    /// Number of edges
    edge_count: usize,
    /// Node attributes storage
    node_attrs: Vec<AHashMap<String, String>>,
}

impl AdjacencyList {
    pub fn new() -> Self {
        AdjacencyList {
            adj: Vec::new(),
            edge_count: 0,
            node_attrs: Vec::new(),
        }
    }
    
    pub fn with_capacity(nodes: usize) -> Self {
        AdjacencyList {
            adj: Vec::with_capacity(nodes),
            edge_count: 0,
            node_attrs: Vec::with_capacity(nodes),
        }
    }
    
    pub fn add_node(&mut self) -> Node {
        let node = self.adj.len();
        self.adj.push(AHashMap::new());
        self.node_attrs.push(AHashMap::new());
        node
    }
    
    pub fn add_edge(&mut self, source: Node, target: Node, weight: Weight) -> bool {
        if source >= self.adj.len() || target >= self.adj.len() {
            return false;
        }
        
        let is_new = self.adj[source].insert(target, weight).is_none();
        if is_new {
            self.edge_count += 1;
        }
        is_new
    }
    
    pub fn has_edge(&self, source: Node, target: Node) -> bool {
        source < self.adj.len() && self.adj[source].contains_key(&target)
    }
    
    pub fn edge_weight(&self, source: Node, target: Node) -> Option<Weight> {
        self.adj.get(source)?.get(&target).copied()
    }
    
    pub fn neighbors(&self, node: Node) -> impl Iterator<Item = (Node, Weight)> + '_ {
        self.adj
            .get(node)
            .into_iter()
            .flat_map(|neighbors| neighbors.iter().map(|(&n, &w)| (n, w)))
    }
    
    pub fn node_count(&self) -> usize {
        self.adj.len()
    }
    
    pub fn edge_count(&self) -> usize {
        self.edge_count
    }
    
    pub fn degree(&self, node: Node) -> usize {
        self.adj.get(node).map_or(0, |n| n.len())
    }
    
    pub fn nodes(&self) -> impl Iterator<Item = Node> {
        0..self.adj.len()
    }
    
    pub fn edges(&self) -> impl Iterator<Item = Edge> + '_ {
        self.adj.iter().enumerate().flat_map(|(source, neighbors)| {
            neighbors.iter().map(move |(&target, &weight)| {
                Edge::new(source, target, weight)
            })
        })
    }
}