//! Push-relabel algorithm for maximum flow

use super::MaxFlowResult;
use crate::graph::traits::GraphBase;
use crate::errors::Result;
use std::hash::Hash;

/// Compute maximum flow using push-relabel algorithm
pub fn push_relabel<G, N>(
    graph: &G,
    source: N,
    sink: N,
) -> Result<MaxFlowResult<N>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    // Push-relabel is often faster than Ford-Fulkerson for dense graphs
    todo!("Implement push-relabel algorithm")
}

/// Get just the flow value
pub fn push_relabel_flow<G, N>(
    graph: &G,
    source: N,
    sink: N,
) -> Result<f64>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let result = push_relabel(graph, source, sink)?;
    Ok(result.flow_value)
}