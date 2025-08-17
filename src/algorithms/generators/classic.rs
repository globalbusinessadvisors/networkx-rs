//! Classic graph generators

use crate::graph::Graph;
use crate::errors::Result;

/// Generate a complete graph on n nodes
///
/// A complete graph is a graph where every pair of distinct vertices is connected by an edge.
pub fn complete_graph(n: usize) -> Result<Graph<usize>> {
    let mut graph = Graph::new();
    
    // Add all nodes
    for i in 0..n {
        graph.add_node(i);
    }
    
    // Add all possible edges
    for i in 0..n {
        for j in (i + 1)..n {
            graph.add_edge(i, j, None);
        }
    }
    
    Ok(graph)
}

/// Generate a cycle graph on n nodes
///
/// A cycle graph is a graph that consists of a single cycle.
pub fn cycle_graph(n: usize) -> Result<Graph<usize>> {
    let mut graph = Graph::new();
    
    if n == 0 {
        return Ok(graph);
    }
    
    // Add all nodes
    for i in 0..n {
        graph.add_node(i);
    }
    
    if n == 1 {
        return Ok(graph);
    }
    
    // Add edges to form a cycle
    for i in 0..n {
        let next = (i + 1) % n;
        graph.add_edge(i, next, None);
    }
    
    Ok(graph)
}

/// Generate a path graph on n nodes
///
/// A path graph is a graph that consists of a single path.
pub fn path_graph(n: usize) -> Result<Graph<usize>> {
    let mut graph = Graph::new();
    
    if n == 0 {
        return Ok(graph);
    }
    
    // Add all nodes
    for i in 0..n {
        graph.add_node(i);
    }
    
    // Add edges to form a path
    for i in 0..(n - 1) {
        graph.add_edge(i, i + 1, None);
    }
    
    Ok(graph)
}

/// Generate a star graph on n nodes
///
/// A star graph is a graph where one central node is connected to all other nodes.
pub fn star_graph(n: usize) -> Result<Graph<usize>> {
    let mut graph = Graph::new();
    
    if n == 0 {
        return Ok(graph);
    }
    
    // Add all nodes
    for i in 0..n {
        graph.add_node(i);
    }
    
    // Connect center (node 0) to all other nodes
    for i in 1..n {
        graph.add_edge(0, i, None);
    }
    
    Ok(graph)
}

/// Generate a wheel graph on n nodes
///
/// A wheel graph is a graph formed by connecting a single node to all nodes of a cycle.
pub fn wheel_graph(n: usize) -> Result<Graph<usize>> {
    let mut graph = Graph::new();
    
    if n == 0 {
        return Ok(graph);
    }
    
    // Add all nodes
    for i in 0..n {
        graph.add_node(i);
    }
    
    if n == 1 {
        return Ok(graph);
    }
    
    // Create the outer cycle (nodes 1 to n-1)
    for i in 1..n {
        let next = if i == n - 1 { 1 } else { i + 1 };
        graph.add_edge(i, next, None);
    }
    
    // Connect center (node 0) to all nodes in the cycle
    for i in 1..n {
        graph.add_edge(0, i, None);
    }
    
    Ok(graph)
}

/// Generate a grid graph with dimensions m x n
pub fn grid_graph(m: usize, n: usize) -> Result<Graph<(usize, usize)>> {
    let mut graph = Graph::new();
    
    // Add all nodes
    for i in 0..m {
        for j in 0..n {
            graph.add_node((i, j));
        }
    }
    
    // Add horizontal edges
    for i in 0..m {
        for j in 0..(n - 1) {
            graph.add_edge((i, j), (i, j + 1), None);
        }
    }
    
    // Add vertical edges
    for i in 0..(m - 1) {
        for j in 0..n {
            graph.add_edge((i, j), (i + 1, j), None);
        }
    }
    
    Ok(graph)
}

/// Generate a hypercube graph of dimension n
pub fn hypercube_graph(n: usize) -> Result<Graph<usize>> {
    let mut graph = Graph::new();
    let num_nodes = 1 << n; // 2^n nodes
    
    // Add all nodes
    for i in 0..num_nodes {
        graph.add_node(i);
    }
    
    // Add edges: two nodes are connected if their binary representations differ in exactly one bit
    for i in 0..num_nodes {
        for bit in 0..n {
            let neighbor = i ^ (1 << bit);
            if neighbor > i { // Avoid duplicate edges
                graph.add_edge(i, neighbor, None);
            }
        }
    }
    
    Ok(graph)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::traits::GraphBase;
    
    #[test]
    fn test_complete_graph() {
        let graph = complete_graph(5).unwrap();
        assert_eq!(graph.node_count(), 5);
        assert_eq!(graph.edge_count(), 10); // 5 * 4 / 2
        
        // Check that all pairs are connected
        for i in 0..5 {
            for j in 0..5 {
                if i != j {
                    assert!(graph.has_edge(&i, &j) || graph.has_edge(&j, &i));
                }
            }
        }
    }
    
    #[test]
    fn test_cycle_graph() {
        let graph = cycle_graph(6).unwrap();
        assert_eq!(graph.node_count(), 6);
        assert_eq!(graph.edge_count(), 6);
        
        // Check that each node has degree 2
        for i in 0..6 {
            assert_eq!(graph.degree(&i), 2);
        }
    }
    
    #[test]
    fn test_path_graph() {
        let graph = path_graph(5).unwrap();
        assert_eq!(graph.node_count(), 5);
        assert_eq!(graph.edge_count(), 4);
        
        // Check endpoints have degree 1, middle nodes have degree 2
        assert_eq!(graph.degree(&0), 1);
        assert_eq!(graph.degree(&4), 1);
        for i in 1..4 {
            assert_eq!(graph.degree(&i), 2);
        }
    }
    
    #[test]
    fn test_star_graph() {
        let graph = star_graph(6).unwrap();
        assert_eq!(graph.node_count(), 6);
        assert_eq!(graph.edge_count(), 5);
        
        // Check center has degree n-1, others have degree 1
        assert_eq!(graph.degree(&0), 5);
        for i in 1..6 {
            assert_eq!(graph.degree(&i), 1);
        }
    }
    
    #[test]
    fn test_wheel_graph() {
        let graph = wheel_graph(7).unwrap();
        assert_eq!(graph.node_count(), 7);
        assert_eq!(graph.edge_count(), 12); // 6 spokes + 6 cycle edges
        
        // Check center has degree n-1
        assert_eq!(graph.degree(&0), 6);
        // Check rim nodes have degree 3 (2 cycle + 1 spoke)
        for i in 1..7 {
            assert_eq!(graph.degree(&i), 3);
        }
    }
    
    #[test]
    fn test_grid_graph() {
        let graph = grid_graph(3, 4).unwrap();
        assert_eq!(graph.node_count(), 12);
        assert_eq!(graph.edge_count(), 17); // (3-1)*4 + 3*(4-1) = 8 + 9
        
        // Check corner nodes have degree 2
        assert_eq!(graph.degree(&(0, 0)), 2);
        assert_eq!(graph.degree(&(2, 3)), 2);
    }
    
    #[test]
    fn test_hypercube_graph() {
        let graph = hypercube_graph(3).unwrap();
        assert_eq!(graph.node_count(), 8); // 2^3
        assert_eq!(graph.edge_count(), 12); // 3 * 2^(3-1)
        
        // Each node should have degree 3
        for i in 0..8 {
            assert_eq!(graph.degree(&i), 3);
        }
    }
}