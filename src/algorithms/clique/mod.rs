//! Clique detection algorithms

pub mod bron_kerbosch;
pub mod max_clique;
pub mod enumerate;

pub use bron_kerbosch::{find_cliques, find_maximal_cliques, bron_kerbosch};
pub use max_clique::{max_clique, max_weight_clique, clique_number};
pub use enumerate::{enumerate_all_cliques, find_cliques_of_size, k_clique_communities};

use std::collections::HashSet;
use std::hash::Hash;

/// A clique in a graph (set of nodes)
pub type Clique<N> = HashSet<N>;

/// Check if a set of nodes forms a clique
pub fn is_clique<G, N>(graph: &G, nodes: &HashSet<N>) -> bool
where
    G: crate::graph::traits::GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    for node1 in nodes {
        for node2 in nodes {
            if node1 != node2 && !graph.has_edge(node1, node2) {
                return false;
            }
        }
    }
    true
}