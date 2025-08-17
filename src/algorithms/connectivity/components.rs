//! Connected components algorithms

use crate::graph::traits::{GraphBase, GraphAlgorithms};
use crate::errors::Result;
use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::Hash;

/// Find all connected components in an undirected graph
pub fn connected_components<G, N>(graph: &G) -> Result<Vec<HashSet<N>>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let mut visited = HashSet::new();
    let mut components = Vec::new();
    
    for node in graph.nodes() {
        if !visited.contains(&node) {
            let mut component = HashSet::new();
            let mut stack = vec![node.clone()];
            
            while let Some(current) = stack.pop() {
                if visited.insert(current.clone()) {
                    component.insert(current.clone());
                    for neighbor in graph.neighbors(&current) {
                        if !visited.contains(&neighbor) {
                            stack.push(neighbor);
                        }
                    }
                }
            }
            
            components.push(component);
        }
    }
    
    Ok(components)
}

/// Find strongly connected components in a directed graph using Tarjan's algorithm
pub fn strongly_connected_components<G, N>(graph: &G) -> Result<Vec<HashSet<N>>>
where
    G: GraphBase<NodeId = N> + GraphAlgorithms,
    N: Clone + Hash + Eq,
{
    if !graph.is_directed() {
        return connected_components(graph);
    }
    
    let nodes: Vec<N> = graph.nodes().collect();
    let mut index_counter = 0;
    let mut stack = Vec::new();
    let mut lowlinks = HashMap::new();
    let mut index = HashMap::new();
    let mut on_stack = HashSet::new();
    let mut components = Vec::new();
    
    for node in &nodes {
        if !index.contains_key(node) {
            strongconnect(
                graph,
                node.clone(),
                &mut index_counter,
                &mut stack,
                &mut lowlinks,
                &mut index,
                &mut on_stack,
                &mut components,
            );
        }
    }
    
    Ok(components)
}

/// Helper function for Tarjan's algorithm
fn strongconnect<G, N>(
    graph: &G,
    v: N,
    index_counter: &mut usize,
    stack: &mut Vec<N>,
    lowlinks: &mut HashMap<N, usize>,
    index: &mut HashMap<N, usize>,
    on_stack: &mut HashSet<N>,
    components: &mut Vec<HashSet<N>>,
) where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    index.insert(v.clone(), *index_counter);
    lowlinks.insert(v.clone(), *index_counter);
    *index_counter += 1;
    stack.push(v.clone());
    on_stack.insert(v.clone());
    
    for w in graph.neighbors(&v) {
        if !index.contains_key(&w) {
            strongconnect(
                graph,
                w.clone(),
                index_counter,
                stack,
                lowlinks,
                index,
                on_stack,
                components,
            );
            let w_lowlink = lowlinks[&w];
            let v_lowlink = lowlinks[&v];
            lowlinks.insert(v.clone(), v_lowlink.min(w_lowlink));
        } else if on_stack.contains(&w) {
            let w_index = index[&w];
            let v_lowlink = lowlinks[&v];
            lowlinks.insert(v.clone(), v_lowlink.min(w_index));
        }
    }
    
    if lowlinks[&v] == index[&v] {
        let mut component = HashSet::new();
        loop {
            let w = stack.pop().unwrap();
            on_stack.remove(&w);
            component.insert(w.clone());
            if w == v {
                break;
            }
        }
        components.push(component);
    }
}

/// Find weakly connected components in a directed graph
pub fn weakly_connected_components<G, N>(graph: &G) -> Result<Vec<HashSet<N>>>
where
    G: GraphBase<NodeId = N> + GraphAlgorithms,
    N: Clone + Hash + Eq,
{
    if !graph.is_directed() {
        return connected_components(graph);
    }
    
    // Treat directed graph as undirected for weak connectivity
    let mut visited = HashSet::new();
    let mut components = Vec::new();
    
    for node in graph.nodes() {
        if !visited.contains(&node) {
            let mut component = HashSet::new();
            let mut queue = VecDeque::new();
            queue.push_back(node.clone());
            visited.insert(node.clone());
            
            while let Some(current) = queue.pop_front() {
                component.insert(current.clone());
                
                // Check both outgoing and incoming edges
                for neighbor in graph.neighbors(&current) {
                    if visited.insert(neighbor.clone()) {
                        queue.push_back(neighbor);
                    }
                }
                
                // Also check for nodes that point to current
                for other in graph.nodes() {
                    if graph.has_edge(&other, &current) && visited.insert(other.clone()) {
                        queue.push_back(other);
                    }
                }
            }
            
            components.push(component);
        }
    }
    
    Ok(components)
}

/// Check if the graph is connected
pub fn is_connected<G, N>(graph: &G) -> Result<bool>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let components = connected_components(graph)?;
    Ok(components.len() <= 1)
}

/// Check if a directed graph is strongly connected
pub fn is_strongly_connected<G, N>(graph: &G) -> Result<bool>
where
    G: GraphBase<NodeId = N> + GraphAlgorithms,
    N: Clone + Hash + Eq,
{
    if !graph.is_directed() {
        return is_connected(graph);
    }
    
    let components = strongly_connected_components(graph)?;
    Ok(components.len() == 1)
}

/// Get the number of connected components
pub fn number_connected_components<G, N>(graph: &G) -> Result<usize>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let components = connected_components(graph)?;
    Ok(components.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{Graph, DiGraph};
    
    #[test]
    fn test_connected_components() {
        let mut graph = Graph::new();
        
        // Component 1
        graph.add_edge(1, 2, None);
        graph.add_edge(2, 3, None);
        
        // Component 2
        graph.add_edge(4, 5, None);
        
        // Isolated node
        graph.add_node(6);
        
        let components = connected_components(&graph).unwrap();
        assert_eq!(components.len(), 3);
        
        // Check component sizes
        let mut sizes: Vec<usize> = components.iter().map(|c| c.len()).collect();
        sizes.sort();
        assert_eq!(sizes, vec![1, 1, 3]);
    }
    
    #[test]
    fn test_strongly_connected_components() {
        let mut graph = DiGraph::new();
        
        // SCC 1: 1 -> 2 -> 3 -> 1
        graph.add_edge(1, 2, None);
        graph.add_edge(2, 3, None);
        graph.add_edge(3, 1, None);
        
        // SCC 2: 4 <-> 5
        graph.add_edge(4, 5, None);
        graph.add_edge(5, 4, None);
        
        // Bridge
        graph.add_edge(3, 4, None);
        
        let sccs = strongly_connected_components(&graph).unwrap();
        assert_eq!(sccs.len(), 2);
        
        // Check SCC sizes
        let mut sizes: Vec<usize> = sccs.iter().map(|c| c.len()).collect();
        sizes.sort();
        assert_eq!(sizes, vec![2, 3]);
    }
    
    #[test]
    fn test_is_connected() {
        let mut graph = Graph::new();
        
        // Connected graph
        graph.add_edge(1, 2, None);
        graph.add_edge(2, 3, None);
        graph.add_edge(3, 4, None);
        
        assert!(is_connected(&graph).unwrap());
        
        // Add disconnected component
        graph.add_edge(5, 6, None);
        
        assert!(!is_connected(&graph).unwrap());
    }
}