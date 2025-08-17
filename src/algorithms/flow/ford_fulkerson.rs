//! Ford-Fulkerson algorithm for maximum flow

use super::MaxFlowResult;
use crate::graph::traits::GraphBase;
use crate::errors::Result;
use std::hash::Hash;

/// Compute maximum flow using Ford-Fulkerson algorithm with DFS
pub fn ford_fulkerson<G, N>(
    graph: &G,
    source: N,
    sink: N,
) -> Result<MaxFlowResult<N>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    // Similar to Edmonds-Karp but uses DFS instead of BFS
    todo!("Implement Ford-Fulkerson with DFS")
}

/// Get just the flow value
pub fn ford_fulkerson_flow<G, N>(
    graph: &G,
    source: N,
    sink: N,
) -> Result<f64>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let result = ford_fulkerson(graph, source, sink)?;
    Ok(result.flow_value)
}