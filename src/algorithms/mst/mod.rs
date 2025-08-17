//! Minimum spanning tree algorithms

pub mod kruskal;
pub mod prim;
pub mod boruvka;

pub use kruskal::{kruskal_mst, kruskal_mst_edges};
pub use prim::{prim_mst, prim_mst_edges};
pub use boruvka::boruvka_mst;

use crate::graph::traits::GraphBase;
use crate::errors::Result;
use std::hash::Hash;

/// Edge representation for MST algorithms
#[derive(Debug, Clone)]
pub struct MSTEdge<N> {
    pub source: N,
    pub target: N,
    pub weight: f64,
}

impl<N: Clone> MSTEdge<N> {
    pub fn new(source: N, target: N, weight: f64) -> Self {
        MSTEdge { source, target, weight }
    }
}

/// Get the minimum spanning tree of a graph
pub fn minimum_spanning_tree<G, N>(graph: &G) -> Result<Vec<MSTEdge<N>>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    // Default to Kruskal's algorithm
    kruskal_mst(graph)
}

/// Get the total weight of the minimum spanning tree
pub fn minimum_spanning_tree_weight<G, N>(graph: &G) -> Result<f64>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let mst = minimum_spanning_tree(graph)?;
    Ok(mst.iter().map(|e| e.weight).sum())
}