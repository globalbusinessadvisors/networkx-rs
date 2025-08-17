//! Dinic's algorithm for maximum flow

use super::MaxFlowResult;
use crate::graph::traits::GraphBase;
use crate::errors::Result;
use std::hash::Hash;

/// Compute maximum flow using Dinic's algorithm
pub fn dinic<G, N>(
    graph: &G,
    source: N,
    sink: N,
) -> Result<MaxFlowResult<N>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    // Dinic's algorithm uses level graphs and blocking flows
    todo!("Implement Dinic's algorithm")
}

/// Get just the flow value
pub fn dinic_flow<G, N>(
    graph: &G,
    source: N,
    sink: N,
) -> Result<f64>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let result = dinic(graph, source, sink)?;
    Ok(result.flow_value)
}