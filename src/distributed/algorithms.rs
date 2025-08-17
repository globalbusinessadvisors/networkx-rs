//! Distributed graph algorithms

use super::partition::{GraphPartitioner, PartitionStrategy, Partition};
use crate::graph::traits::GraphBase;
use crate::errors::Result;
use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::Hash;

/// Distributed PageRank using message passing
pub async fn distributed_pagerank<G, N>(
    graph: &G,
    num_workers: usize,
    alpha: Option<f64>,
    max_iter: Option<usize>,
    tolerance: Option<f64>,
) -> Result<HashMap<N, f64>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq + Send + Sync,
{
    let alpha = alpha.unwrap_or(0.85);
    let max_iter = max_iter.unwrap_or(100);
    let tolerance = tolerance.unwrap_or(1e-6);
    
    if num_workers <= 1 {
        // Fall back to single-threaded PageRank
        return crate::algorithms::centrality::pagerank(graph, Some(alpha), Some(max_iter), Some(tolerance));
    }
    
    // Partition the graph
    let partitioner = GraphPartitioner::new(PartitionStrategy::EdgeCut, num_workers);
    let partitions = partitioner.partition(graph)?;
    
    let nodes: Vec<N> = graph.nodes().collect();
    let n = nodes.len();
    
    if n == 0 {
        return Ok(HashMap::new());
    }
    
    // Initialize PageRank values
    let mut pagerank: HashMap<N, f64> = nodes.iter()
        .map(|node| (node.clone(), 1.0 / n as f64))
        .collect();
    
    // Simulate distributed computation
    for _iter in 0..max_iter {
        let old_pagerank = pagerank.clone();
        let mut new_pagerank: HashMap<N, f64> = HashMap::new();
        
        // Process each partition (simulating workers)
        for partition in &partitions {
            let partition_result = process_pagerank_partition(
                &partition,
                &old_pagerank,
                alpha,
                n,
            );
            
            // Merge results
            for (node, value) in partition_result {
                *new_pagerank.entry(node).or_insert(0.0) += value;
            }
        }
        
        // Check convergence
        let mut diff = 0.0;
        for node in &nodes {
            let old_val = old_pagerank.get(node).copied().unwrap_or(0.0);
            let new_val = new_pagerank.get(node).copied().unwrap_or(0.0);
            diff += (old_val - new_val).abs();
        }
        
        if diff < tolerance {
            break;
        }
        
        pagerank = new_pagerank;
    }
    
    Ok(pagerank)
}

/// Process PageRank for a single partition
fn process_pagerank_partition<N>(
    partition: &Partition<N>,
    current_pagerank: &HashMap<N, f64>,
    alpha: f64,
    total_nodes: usize,
) -> HashMap<N, f64>
where
    N: Clone + Hash + Eq,
{
    let mut result = HashMap::new();
    let teleport = (1.0 - alpha) / total_nodes as f64;
    
    // Process internal edges
    for (u, v, _weight) in &partition.internal_edges {
        let u_rank = current_pagerank.get(u).copied().unwrap_or(0.0);
        let u_degree = partition.internal_edges.iter()
            .filter(|(source, _, _)| source == u)
            .count() as f64;
        
        if u_degree > 0.0 {
            *result.entry(v.clone()).or_insert(teleport) += alpha * u_rank / u_degree;
        }
    }
    
    // Process cut edges (communication between partitions)
    for (u, v, _weight) in &partition.cut_edges {
        if partition.nodes.contains(v) {
            let u_rank = current_pagerank.get(u).copied().unwrap_or(0.0);
            // Simplified: assume uniform out-degree
            *result.entry(v.clone()).or_insert(teleport) += alpha * u_rank;
        }
    }
    
    // Ensure all nodes in partition have values
    for node in &partition.nodes {
        result.entry(node.clone()).or_insert(teleport);
    }
    
    result
}

/// Distributed BFS using frontier synchronization
pub async fn distributed_bfs<G, N>(
    graph: &G,
    source: N,
    num_workers: usize,
) -> Result<HashMap<N, usize>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq + Send + Sync,
{
    if num_workers <= 1 {
        // Fall back to single-threaded BFS
        return crate::algorithms::traversal::bfs(graph, source);
    }
    
    // Partition the graph
    let partitioner = GraphPartitioner::new(PartitionStrategy::Hash, num_workers);
    let partitions = partitioner.partition(graph)?;
    
    // Find which partition contains the source
    let mut source_partition = None;
    for (i, partition) in partitions.iter().enumerate() {
        if partition.nodes.contains(&source) {
            source_partition = Some(i);
            break;
        }
    }
    
    let source_partition = source_partition.ok_or_else(|| {
        crate::errors::NetworkXError::NodeNotFound("Source node not found in any partition".to_string())
    })?;
    
    let mut distances: HashMap<N, usize> = HashMap::new();
    let mut current_frontier: HashSet<N> = HashSet::new();
    let mut level = 0usize;
    
    // Initialize with source
    distances.insert(source.clone(), 0);
    current_frontier.insert(source);
    
    // Distributed BFS level by level
    while !current_frontier.is_empty() {
        let mut next_frontier: HashSet<N> = HashSet::new();
        
        // Process each partition in parallel (simulated)
        for (i, partition) in partitions.iter().enumerate() {
            let partition_frontier = process_bfs_partition(
                partition,
                &current_frontier,
                &distances,
                level + 1,
            );
            
            for node in partition_frontier {
                if !distances.contains_key(&node) {
                    distances.insert(node.clone(), level + 1);
                    next_frontier.insert(node);
                }
            }
        }
        
        current_frontier = next_frontier;
        level += 1;
    }
    
    Ok(distances)
}

/// Process BFS for a single partition
fn process_bfs_partition<N>(
    partition: &Partition<N>,
    current_frontier: &HashSet<N>,
    distances: &HashMap<N, usize>,
    next_level: usize,
) -> HashSet<N>
where
    N: Clone + Hash + Eq,
{
    let mut next_frontier = HashSet::new();
    
    // Check internal edges
    for (u, v, _weight) in &partition.internal_edges {
        if current_frontier.contains(u) && !distances.contains_key(v) {
            next_frontier.insert(v.clone());
        }
    }
    
    // Check cut edges
    for (u, v, _weight) in &partition.cut_edges {
        if current_frontier.contains(u) && !distances.contains_key(v) && partition.nodes.contains(v) {
            next_frontier.insert(v.clone());
        }
    }
    
    next_frontier
}

/// Distributed connected components using label propagation
pub async fn distributed_connected_components<G, N>(
    graph: &G,
    num_workers: usize,
) -> Result<Vec<Vec<N>>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq + Send + Sync,
{
    if num_workers <= 1 {
        // Fall back to single-threaded algorithm
        return crate::algorithms::connectivity::connected_components(graph);
    }
    
    // Partition the graph
    let partitioner = GraphPartitioner::new(PartitionStrategy::EdgeCut, num_workers);
    let partitions = partitioner.partition(graph)?;
    
    let nodes: Vec<N> = graph.nodes().collect();
    
    // Initialize each node as its own component
    let mut component_id: HashMap<N, usize> = nodes.iter()
        .enumerate()
        .map(|(i, node)| (node.clone(), i))
        .collect();
    
    let mut changed = true;
    let max_iterations = 100;
    let mut iteration = 0;
    
    // Iterate until convergence
    while changed && iteration < max_iterations {
        changed = false;
        let old_component_id = component_id.clone();
        
        // Process each partition
        for partition in &partitions {
            let partition_changed = process_components_partition(
                partition,
                &mut component_id,
            );
            changed = changed || partition_changed;
        }
        
        iteration += 1;
    }
    
    // Group nodes by component ID
    let mut components: HashMap<usize, Vec<N>> = HashMap::new();
    for (node, &comp_id) in &component_id {
        components.entry(comp_id).or_insert_with(Vec::new).push(node.clone());
    }
    
    Ok(components.into_values().collect())
}

/// Process connected components for a single partition
fn process_components_partition<N>(
    partition: &Partition<N>,
    component_id: &mut HashMap<N, usize>,
) -> bool
where
    N: Clone + Hash + Eq,
{
    let mut changed = false;
    
    // Process internal edges
    for (u, v, _weight) in &partition.internal_edges {
        let u_comp = component_id.get(u).copied().unwrap_or(0);
        let v_comp = component_id.get(v).copied().unwrap_or(0);
        
        if u_comp != v_comp {
            let min_comp = u_comp.min(v_comp);
            let max_comp = u_comp.max(v_comp);
            
            // Update to smaller component ID
            if component_id.get(u).copied().unwrap_or(0) == max_comp {
                component_id.insert(u.clone(), min_comp);
                changed = true;
            }
            if component_id.get(v).copied().unwrap_or(0) == max_comp {
                component_id.insert(v.clone(), min_comp);
                changed = true;
            }
        }
    }
    
    // Process cut edges (simplified)
    for (u, v, _weight) in &partition.cut_edges {
        if partition.nodes.contains(v) {
            let u_comp = component_id.get(u).copied().unwrap_or(0);
            let v_comp = component_id.get(v).copied().unwrap_or(0);
            
            if u_comp != v_comp {
                let min_comp = u_comp.min(v_comp);
                component_id.insert(v.clone(), min_comp);
                changed = true;
            }
        }
    }
    
    changed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::Graph;

    #[tokio::test]
    async fn test_distributed_pagerank() {
        let mut graph = Graph::new();
        graph.add_edge(1, 2, Some(1.0));
        graph.add_edge(2, 3, Some(1.0));
        graph.add_edge(3, 1, Some(1.0));
        
        let result = distributed_pagerank(&graph, 2, Some(0.85), Some(10), Some(1e-6)).await.unwrap();
        
        assert_eq!(result.len(), 3);
        for &value in result.values() {
            assert!(value > 0.0);
        }
    }
    
    #[tokio::test]
    async fn test_distributed_bfs() {
        let mut graph = Graph::new();
        graph.add_edge(1, 2, Some(1.0));
        graph.add_edge(2, 3, Some(1.0));
        graph.add_edge(1, 4, Some(1.0));
        
        let result = distributed_bfs(&graph, 1, 2).await.unwrap();
        
        assert_eq!(result.get(&1), Some(&0));
        assert_eq!(result.get(&2), Some(&1));
        assert_eq!(result.get(&4), Some(&1));
    }
    
    #[tokio::test]
    async fn test_distributed_connected_components() {
        let mut graph = Graph::new();
        graph.add_edge(1, 2, Some(1.0));
        graph.add_edge(2, 3, Some(1.0));
        graph.add_edge(4, 5, Some(1.0));
        
        let result = distributed_connected_components(&graph, 2).await.unwrap();
        
        assert_eq!(result.len(), 2); // Two components
        let mut sizes: Vec<usize> = result.iter().map(|comp| comp.len()).collect();
        sizes.sort();
        assert_eq!(sizes, vec![2, 3]);
    }
}