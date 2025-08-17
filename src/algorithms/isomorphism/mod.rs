//! Graph isomorphism algorithms

pub mod vf2;
pub mod canonical;

pub use vf2::{is_isomorphic, find_isomorphism, find_all_isomorphisms, VF2State};
pub use canonical::{canonical_labeling, is_automorphism};

use crate::graph::traits::GraphBase;
use std::collections::HashMap;
use std::hash::Hash;

/// A graph isomorphism (bijection between node sets)
pub type Isomorphism<N> = HashMap<N, N>;

/// Check if two graphs have the same structure
pub fn graphs_equal<G1, G2, N1, N2>(g1: &G1, g2: &G2) -> bool
where
    G1: GraphBase<NodeId = N1>,
    G2: GraphBase<NodeId = N2>,
    N1: Clone + Hash + Eq,
    N2: Clone + Hash + Eq,
{
    // Quick checks
    if g1.node_count() != g2.node_count() {
        return false;
    }
    
    if g1.edge_count() != g2.edge_count() {
        return false;
    }
    
    // Check degree sequence
    let mut deg1: Vec<usize> = g1.nodes().map(|n| g1.degree(&n)).collect();
    let mut deg2: Vec<usize> = g2.nodes().map(|n| g2.degree(&n)).collect();
    
    deg1.sort_unstable();
    deg2.sort_unstable();
    
    deg1 == deg2
}