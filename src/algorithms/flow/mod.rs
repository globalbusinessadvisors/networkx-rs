//! Flow algorithms for graphs

pub mod ford_fulkerson;
pub mod edmonds_karp;
pub mod dinic;
pub mod push_relabel;

pub use ford_fulkerson::{ford_fulkerson, ford_fulkerson_flow};
pub use edmonds_karp::{edmonds_karp, edmonds_karp_flow};
pub use dinic::{dinic, dinic_flow};
pub use push_relabel::{push_relabel, push_relabel_flow};

use crate::graph::traits::GraphBase;
use crate::errors::Result;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

/// Result of a maximum flow computation
#[derive(Debug, Clone)]
pub struct MaxFlowResult<N> {
    /// The maximum flow value
    pub flow_value: f64,
    /// The flow on each edge
    pub flow_dict: HashMap<(N, N), f64>,
}

/// Compute maximum flow using the default algorithm (Edmonds-Karp)
pub fn maximum_flow<G, N>(
    graph: &G,
    source: N,
    sink: N,
) -> Result<MaxFlowResult<N>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    edmonds_karp(graph, source, sink)
}

/// Compute the minimum cut corresponding to a maximum flow
pub fn minimum_cut<G, N>(
    graph: &G,
    source: N,
    sink: N,
) -> Result<(f64, HashSet<N>, HashSet<N>)>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    use std::collections::HashSet;
    
    let flow_result = maximum_flow(graph, source.clone(), sink.clone())?;
    
    // Find reachable nodes from source in residual graph
    let mut reachable = HashSet::new();
    let mut stack = vec![source.clone()];
    reachable.insert(source);
    
    while let Some(node) = stack.pop() {
        for neighbor in graph.neighbors(&node) {
            let capacity = graph.get_edge_weight(&node, &neighbor).unwrap_or(0.0);
            let flow = flow_result.flow_dict
                .get(&(node.clone(), neighbor.clone()))
                .copied()
                .unwrap_or(0.0);
            
            if capacity > flow && !reachable.contains(&neighbor) {
                reachable.insert(neighbor.clone());
                stack.push(neighbor);
            }
        }
    }
    
    // Non-reachable nodes form the other partition
    let mut non_reachable = HashSet::new();
    for node in graph.nodes() {
        if !reachable.contains(&node) {
            non_reachable.insert(node);
        }
    }
    
    Ok((flow_result.flow_value, reachable, non_reachable))
}