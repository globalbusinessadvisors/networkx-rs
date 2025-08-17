//! Canonical labeling and automorphism detection

use crate::graph::traits::GraphBase;
use crate::errors::Result;
use std::hash::Hash;

/// Compute canonical labeling of a graph
pub fn canonical_labeling<G, N>(_graph: &G) -> Result<Vec<N>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    // Would implement nauty or similar algorithm
    todo!("Canonical labeling not yet implemented")
}

/// Check if a mapping is an automorphism
pub fn is_automorphism<G, N>(_graph: &G, _mapping: &std::collections::HashMap<N, N>) -> bool
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    todo!("Automorphism checking not yet implemented")
}