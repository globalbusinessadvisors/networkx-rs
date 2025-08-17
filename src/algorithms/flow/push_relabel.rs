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
    use std::collections::{HashMap, VecDeque};
    
    let nodes: Vec<N> = graph.nodes().collect();
    let n = nodes.len();
    
    if n == 0 {
        return Ok(MaxFlowResult {
            flow_value: 0.0,
            flow_dict: HashMap::new(),
        });
    }
    
    // Initialize data structures
    let mut flow_dict: HashMap<(N, N), f64> = HashMap::new();
    let mut excess: HashMap<N, f64> = HashMap::new();
    let mut height: HashMap<N, usize> = HashMap::new();
    
    // Initialize flow and excess
    for node in &nodes {
        excess.insert(node.clone(), 0.0);
        height.insert(node.clone(), 0);
        
        for neighbor in graph.neighbors(node) {
            flow_dict.insert((node.clone(), neighbor), 0.0);
        }
    }
    
    // Set source height to n
    height.insert(source.clone(), n);
    
    // Initialize preflow from source
    for neighbor in graph.neighbors(&source) {
        let capacity = graph.get_edge_weight(&source, &neighbor).unwrap_or(0.0);
        if capacity > 0.0 {
            flow_dict.insert((source.clone(), neighbor.clone()), capacity);
            flow_dict.insert((neighbor.clone(), source.clone()), -capacity);
            *excess.get_mut(&neighbor).unwrap() += capacity;
        }
    }
    
    // Main push-relabel loop
    let mut queue: VecDeque<N> = nodes.iter()
        .filter(|&node| *node != source && *node != sink && excess[node] > 0.0)
        .cloned()
        .collect();
    
    while let Some(current) = queue.pop_front() {
        if excess[&current] > 0.0 && current != source && current != sink {
            let old_height = height[&current];
            discharge(graph, &current, &mut flow_dict, &mut excess, &mut height);
            
            if height[&current] > old_height {
                // Re-add to front of queue if height increased
                queue.push_front(current);
            } else {
                // Check if any neighbors need to be added to queue
                for neighbor in graph.neighbors(&current) {
                    if neighbor != source && neighbor != sink && excess[&neighbor] > 0.0 {
                        queue.push_back(neighbor);
                    }
                }
            }
        }
    }
    
    // Clean up flow dictionary to only include positive flows
    let cleaned_flow: HashMap<(N, N), f64> = flow_dict.into_iter()
        .filter(|(_, flow)| *flow > 0.0)
        .collect();
    
    let flow_value = -excess[&source];
    
    Ok(MaxFlowResult {
        flow_value,
        flow_dict: cleaned_flow,
    })
}

/// Discharge operation for push-relabel
fn discharge<G, N>(
    graph: &G,
    node: &N,
    flow_dict: &mut std::collections::HashMap<(N, N), f64>,
    excess: &mut std::collections::HashMap<N, f64>,
    height: &mut std::collections::HashMap<N, usize>,
) where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    while excess[node] > 0.0 {
        let mut pushed = false;
        
        // Try to push to an admissible neighbor
        for neighbor in graph.neighbors(node) {
            if height[node] == height[&neighbor] + 1 {
                let capacity = graph.get_edge_weight(node, &neighbor).unwrap_or(0.0);
                let current_flow = flow_dict.get(&(node.clone(), neighbor.clone())).copied().unwrap_or(0.0);
                let residual_capacity = capacity - current_flow;
                
                if residual_capacity > 0.0 {
                    // Push flow
                    let push_amount = excess[node].min(residual_capacity);
                    
                    let old_flow = flow_dict.get(&(node.clone(), neighbor.clone())).copied().unwrap_or(0.0);
                    flow_dict.insert((node.clone(), neighbor.clone()), old_flow + push_amount);
                    
                    let reverse_flow = flow_dict.get(&(neighbor.clone(), node.clone())).copied().unwrap_or(0.0);
                    flow_dict.insert((neighbor.clone(), node.clone()), reverse_flow - push_amount);
                    
                    *excess.get_mut(node).unwrap() -= push_amount;
                    *excess.get_mut(&neighbor).unwrap() += push_amount;
                    
                    pushed = true;
                    
                    if excess[node] == 0.0 {
                        break;
                    }
                }
            }
        }
        
        if !pushed {
            // Relabel: increase height
            let mut min_height = usize::MAX;
            for neighbor in graph.neighbors(node) {
                let capacity = graph.get_edge_weight(node, &neighbor).unwrap_or(0.0);
                let current_flow = flow_dict.get(&(node.clone(), neighbor.clone())).copied().unwrap_or(0.0);
                let residual_capacity = capacity - current_flow;
                
                if residual_capacity > 0.0 {
                    min_height = min_height.min(height[&neighbor]);
                }
            }
            
            if min_height != usize::MAX {
                height.insert(node.clone(), min_height + 1);
            } else {
                break; // No valid neighbors
            }
        }
    }
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