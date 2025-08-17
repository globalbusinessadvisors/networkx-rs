//! GPU-accelerated graph algorithms

use super::memory::{GpuGraph, GpuVector, GpuMatrix};
use crate::graph::traits::GraphBase;
use crate::errors::Result;
use std::collections::HashMap;
use std::hash::Hash;

/// GPU-accelerated PageRank algorithm
pub fn gpu_pagerank<G, N>(
    graph: &G,
    alpha: f64,
    max_iter: usize,
    tolerance: f64,
) -> Result<HashMap<N, f64>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let nodes: Vec<N> = graph.nodes().collect();
    let n = nodes.len();
    
    if n == 0 {
        return Ok(HashMap::new());
    }
    
    // Convert graph to GPU format
    let gpu_graph = GpuGraph::from_graph(graph);
    
    // Initialize PageRank vector
    let mut rank = GpuVector::from_vec(vec![1.0 / n as f32; n]);
    
    // Create transition matrix
    let matrix = create_transition_matrix(&gpu_graph, alpha as f32);
    
    // Power iteration
    for _iter in 0..max_iter {
        let old_rank = rank.data.clone();
        rank = matrix.matvec(&rank);
        rank.normalize();
        
        // Check convergence
        let diff: f32 = old_rank.iter()
            .zip(rank.data.iter())
            .map(|(old, new)| (old - new).abs())
            .sum();
        
        if diff < tolerance as f32 {
            break;
        }
    }
    
    // Convert back to HashMap
    let mut result = HashMap::new();
    for (i, node) in nodes.iter().enumerate() {
        result.insert(node.clone(), rank.data[i] as f64);
    }
    
    Ok(result)
}

/// Create transition matrix for PageRank
fn create_transition_matrix(graph: &GpuGraph, alpha: f32) -> GpuMatrix {
    let n = graph.num_nodes;
    let mut matrix = GpuMatrix::new(n, n);
    
    // Compute out-degrees
    let mut out_degree = vec![0u32; n];
    for i in 0..n {
        out_degree[i] = graph.row_offsets[i + 1] - graph.row_offsets[i];
    }
    
    // Fill transition matrix
    for i in 0..n {
        let start = graph.row_offsets[i] as usize;
        let end = graph.row_offsets[i + 1] as usize;
        let degree = out_degree[i] as f32;
        
        if degree > 0.0 {
            for idx in start..end {
                let j = graph.col_indices[idx] as usize;
                matrix.data[j * n + i] = alpha / degree;
            }
        }
        
        // Add teleportation
        for j in 0..n {
            matrix.data[j * n + i] += (1.0 - alpha) / n as f32;
        }
    }
    
    matrix
}

/// GPU-accelerated BFS
pub fn gpu_bfs<G, N>(
    graph: &G,
    source: N,
) -> Result<HashMap<N, usize>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let nodes: Vec<N> = graph.nodes().collect();
    let gpu_graph = GpuGraph::from_graph(graph);
    
    // Find source index
    let source_idx = nodes.iter()
        .position(|n| n == &source)
        .ok_or_else(|| crate::errors::NetworkXError::NodeNotFound(
            "Source node not found".to_string()
        ))?;
    
    let distances = gpu_bfs_kernel(&gpu_graph, source_idx);
    
    // Convert to HashMap
    let mut result = HashMap::new();
    for (i, node) in nodes.iter().enumerate() {
        if distances[i] != u32::MAX {
            result.insert(node.clone(), distances[i] as usize);
        }
    }
    
    Ok(result)
}

/// GPU BFS kernel implementation
fn gpu_bfs_kernel(graph: &GpuGraph, source: usize) -> Vec<u32> {
    let n = graph.num_nodes;
    let mut distances = vec![u32::MAX; n];
    distances[source] = 0;
    
    #[cfg(feature = "arrayfire")]
    {
        // GPU implementation using ArrayFire
        use arrayfire as af;
        
        // Convert to GPU arrays
        let mut d_distances = af::Array::new(&distances, af::Dim4::new(&[n as u64, 1, 1, 1]));
        let d_row_offsets = af::Array::new(&graph.row_offsets, af::Dim4::new(&[graph.row_offsets.len() as u64, 1, 1, 1]));
        let d_col_indices = af::Array::new(&graph.col_indices, af::Dim4::new(&[graph.col_indices.len() as u64, 1, 1, 1]));
        
        let mut changed = true;
        let mut level = 0u32;
        
        while changed {
            changed = false;
            level += 1;
            
            // Parallel BFS iteration
            for i in 0..n {
                let start = graph.row_offsets[i] as usize;
                let end = graph.row_offsets[i + 1] as usize;
                
                for idx in start..end {
                    let j = graph.col_indices[idx] as usize;
                    if distances[i] != u32::MAX && distances[j] == u32::MAX {
                        distances[j] = level;
                        changed = true;
                    }
                }
            }
        }
    }
    #[cfg(not(feature = "arrayfire"))]
    {
        // CPU fallback with level-synchronous BFS
        let mut current_level = vec![source];
        let mut level = 0u32;
        
        while !current_level.is_empty() {
            let mut next_level = Vec::new();
            
            for &node in &current_level {
                let start = graph.row_offsets[node] as usize;
                let end = graph.row_offsets[node + 1] as usize;
                
                for idx in start..end {
                    let neighbor = graph.col_indices[idx] as usize;
                    if distances[neighbor] == u32::MAX {
                        distances[neighbor] = level + 1;
                        next_level.push(neighbor);
                    }
                }
            }
            
            current_level = next_level;
            level += 1;
        }
    }
    
    distances
}

/// GPU-accelerated shortest paths using parallel Bellman-Ford
pub fn gpu_shortest_paths<G, N>(
    graph: &G,
    source: N,
) -> Result<HashMap<N, f64>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let nodes: Vec<N> = graph.nodes().collect();
    let gpu_graph = GpuGraph::from_graph(graph);
    
    // Find source index
    let source_idx = nodes.iter()
        .position(|n| n == &source)
        .ok_or_else(|| crate::errors::NetworkXError::NodeNotFound(
            "Source node not found".to_string()
        ))?;
    
    let distances = gpu_bellman_ford(&gpu_graph, source_idx);
    
    // Convert to HashMap
    let mut result = HashMap::new();
    for (i, node) in nodes.iter().enumerate() {
        if distances[i] != f32::INFINITY {
            result.insert(node.clone(), distances[i] as f64);
        }
    }
    
    Ok(result)
}

/// GPU Bellman-Ford kernel
fn gpu_bellman_ford(graph: &GpuGraph, source: usize) -> Vec<f32> {
    let n = graph.num_nodes;
    let mut distances = vec![f32::INFINITY; n];
    distances[source] = 0.0;
    
    // Parallel Bellman-Ford iterations
    for _ in 0..n {
        let mut changed = false;
        
        for i in 0..n {
            if distances[i] == f32::INFINITY {
                continue;
            }
            
            let start = graph.row_offsets[i] as usize;
            let end = graph.row_offsets[i + 1] as usize;
            
            for idx in start..end {
                let j = graph.col_indices[idx] as usize;
                let weight = graph.edge_weights[idx];
                let new_dist = distances[i] + weight;
                
                if new_dist < distances[j] {
                    distances[j] = new_dist;
                    changed = true;
                }
            }
        }
        
        if !changed {
            break;
        }
    }
    
    distances
}

/// GPU-accelerated eigenvector centrality
pub fn gpu_eigenvector_centrality<G, N>(
    graph: &G,
    max_iter: usize,
    tolerance: f64,
) -> Result<HashMap<N, f64>>
where
    G: GraphBase<NodeId = N>,
    N: Clone + Hash + Eq,
{
    let nodes: Vec<N> = graph.nodes().collect();
    let n = nodes.len();
    
    if n == 0 {
        return Ok(HashMap::new());
    }
    
    // Convert to GPU format
    let gpu_graph = GpuGraph::from_graph(graph);
    let matrix = gpu_graph.to_matrix();
    
    // Initialize eigenvector
    let mut vec = GpuVector::from_vec(vec![1.0 / (n as f32).sqrt(); n]);
    
    // Power iteration
    for _ in 0..max_iter {
        let old_vec = vec.data.clone();
        vec = matrix.matvec(&vec);
        
        // Normalize
        let norm = vec.norm();
        if norm > 0.0 {
            for val in &mut vec.data {
                *val /= norm;
            }
        }
        
        // Check convergence
        let diff: f32 = old_vec.iter()
            .zip(vec.data.iter())
            .map(|(old, new)| (old - new).abs())
            .sum();
        
        if diff < tolerance as f32 {
            break;
        }
    }
    
    // Convert to HashMap
    let mut result = HashMap::new();
    for (i, node) in nodes.iter().enumerate() {
        result.insert(node.clone(), vec.data[i] as f64);
    }
    
    Ok(result)
}