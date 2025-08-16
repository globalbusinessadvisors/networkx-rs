//! Johnson's algorithm for all-pairs shortest paths

use crate::graph::traits::GraphBase;
use crate::graph::DiGraph;
use crate::errors::Result;
use crate::algorithms::paths::{bellman_ford, dijkstra};
use std::collections::HashMap;
use std::hash::Hash;

/// Johnson's algorithm for all-pairs shortest paths
///
/// More efficient than Floyd-Warshall for sparse graphs.
/// Handles negative weights but not negative cycles.
///
/// # Arguments
/// * `graph` - The graph to analyze
/// * `weight_fn` - Function to get edge weights (defaults to 1.0)
///
/// # Returns
/// * `Ok(distances)` - Map of (source, target) -> distance
/// * `Err` - If negative cycle detected
pub fn johnson<G, N>(
    graph: &G,
    weight_fn: Option<impl Fn(&N, &N) -> f64>,
) -> Result<HashMap<(N, N), f64>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq + Ord + Default,
{
    // Create a new graph with auxiliary node
    let mut aux_graph = DiGraph::new();
    let aux_node = N::default(); // Use default as auxiliary node
    
    // Add all original nodes and edges
    for node in graph.nodes() {
        aux_graph.add_node(node.clone());
    }
    
    for (u, v, w) in graph.edges() {
        let weight = if let Some(ref wf) = weight_fn {
            wf(&u, &v)
        } else {
            w
        };
        aux_graph.add_edge(u, v, Some(weight));
    }
    
    // Add auxiliary node with 0-weight edges to all nodes
    aux_graph.add_node(aux_node.clone());
    for node in graph.nodes() {
        aux_graph.add_edge(aux_node.clone(), node, Some(0.0));
    }
    
    // Run Bellman-Ford from auxiliary node
    let (h, _) = bellman_ford::bellman_ford(&aux_graph, aux_node.clone(), 
                                            Some(|u: &N, v: &N| {
        aux_graph.get_edge_weight(u, v).unwrap_or(1.0)
    }))?;
    
    // Run Dijkstra from each node with reweighted edges
    let mut distances = HashMap::new();
    let nodes: Vec<N> = graph.nodes().collect();
    
    for source in &nodes {
        // Reweight function for this source
        let reweight = |u: &N, v: &N| -> f64 {
            let original_weight = if let Some(ref wf) = weight_fn {
                wf(u, v)
            } else {
                graph.get_edge_weight(u, v).unwrap_or(1.0)
            };
            original_weight + h[u] - h[v]
        };
        
        let dist = dijkstra::dijkstra_distances(graph, source.clone(), Some(reweight))?;
        
        for (target, d) in dist {
            // Adjust distance back
            let actual_dist = d - h[source] + h[&target];
            distances.insert((source.clone(), target), actual_dist);
        }
    }
    
    Ok(distances)
}

/// Get shortest path between specific nodes using Johnson's algorithm
pub fn johnson_path<G, N>(
    graph: &G,
    source: N,
    target: N,
    weight_fn: Option<impl Fn(&N, &N) -> f64>,
) -> Result<Option<(Vec<N>, f64)>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq + Ord + Default,
{
    let distances = johnson(graph, weight_fn)?;
    
    if let Some(&dist) = distances.get(&(source.clone(), target.clone())) {
        if dist != f64::INFINITY {
            // TODO: Reconstruct actual path
            return Ok(Some((vec![source, target], dist)));
        }
    }
    
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::DiGraph;

    #[test]
    fn test_johnson_simple() {
        let mut graph = DiGraph::<i32>::new();
        graph.add_edge(0, 1, Some(3.0));
        graph.add_edge(0, 2, Some(8.0));
        graph.add_edge(1, 2, Some(2.0));
        graph.add_edge(2, 3, Some(1.0));
        
        let distances = johnson(&graph, None::<fn(&i32, &i32) -> f64>).unwrap();
        
        assert_eq!(distances[&(0, 1)], 3.0);
        assert_eq!(distances[&(0, 2)], 5.0);
        assert_eq!(distances[&(0, 3)], 6.0);
    }
    
    #[test]
    fn test_johnson_negative_weights() {
        let mut graph = DiGraph::<i32>::new();
        graph.add_edge(0, 1, Some(1.0));
        graph.add_edge(1, 2, Some(-2.0));
        graph.add_edge(2, 3, Some(3.0));
        
        let distances = johnson(&graph, None::<fn(&i32, &i32) -> f64>).unwrap();
        
        assert_eq!(distances[&(0, 1)], 1.0);
        assert_eq!(distances[&(0, 2)], -1.0);
        assert_eq!(distances[&(0, 3)], 2.0);
    }
}