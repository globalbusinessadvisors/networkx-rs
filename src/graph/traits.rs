//! Graph trait definitions for common operations

use std::hash::Hash;

/// Base graph operations with generic node type
pub trait GraphBase {
    type NodeId: Clone + Hash + Eq;
    
    fn node_count(&self) -> usize;
    fn edge_count(&self) -> usize;
    fn has_node(&self, node: &Self::NodeId) -> bool;
    fn has_edge(&self, source: &Self::NodeId, target: &Self::NodeId) -> bool;
    fn nodes(&self) -> Box<dyn Iterator<Item = Self::NodeId> + '_>;
    fn edges(&self) -> Box<dyn Iterator<Item = (Self::NodeId, Self::NodeId, f64)> + '_>;
    fn neighbors(&self, node: &Self::NodeId) -> Box<dyn Iterator<Item = Self::NodeId> + '_>;
    fn degree(&self, node: &Self::NodeId) -> usize;
    fn get_edge_weight(&self, source: &Self::NodeId, target: &Self::NodeId) -> Option<f64>;
}

/// Mutable graph operations
pub trait GraphMut: GraphBase {
    fn add_node(&mut self, node: Self::NodeId) -> bool;
    fn add_edge(&mut self, source: Self::NodeId, target: Self::NodeId, weight: Option<f64>) -> bool;
    fn remove_edge(&mut self, source: &Self::NodeId, target: &Self::NodeId) -> bool;
    fn clear(&mut self);
}

/// Graph algorithms interface
pub trait GraphAlgorithms: GraphBase {
    fn is_directed(&self) -> bool;
    fn to_adjacency_matrix(&self) -> Vec<Vec<Option<f64>>>;
}