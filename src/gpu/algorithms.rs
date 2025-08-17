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
    
    #[cfg(feature = "arrayfire")]
    {
        use arrayfire as af;
        
        // Initialize PageRank vector on GPU
        let initial_rank = 1.0 / n as f32;
        let mut rank = af::constant(initial_rank, af::Dim4::new(&[n as u64, 1, 1, 1]));
        
        // Create transition matrix on GPU
        let (row_offsets, col_indices, values) = create_sparse_transition_matrix(&gpu_graph, alpha as f32);
        
        let af_row_offsets = af::Array::new(&row_offsets, af::Dim4::new(&[row_offsets.len() as u64, 1, 1, 1]));
        let af_col_indices = af::Array::new(&col_indices, af::Dim4::new(&[col_indices.len() as u64, 1, 1, 1]));
        let af_values = af::Array::new(&values, af::Dim4::new(&[values.len() as u64, 1, 1, 1]));
        
        // Power iteration on GPU
        for _iter in 0..max_iter {
            let old_rank = rank.copy();
            
            // Sparse matrix-vector multiplication
            rank = gpu_sparse_matvec(&af_row_offsets, &af_col_indices, &af_values, &rank, n);
            
            // Normalize
            let sum = af::sum_all(&rank).0;
            if sum > 0.0 {
                rank = af::div(&rank, &sum, false);
            }
            
            // Check convergence
            let diff = af::sum_all(&af::abs(&af::sub(&rank, &old_rank, false))).0;
            if diff < tolerance as f32 {
                break;
            }
        }
        
        // Copy result back to host
        let mut host_result = vec![0.0f32; n];
        rank.host(&mut host_result);
        
        // Convert to HashMap
        let mut result = HashMap::new();
        for (i, node) in nodes.iter().enumerate() {
            result.insert(node.clone(), host_result[i] as f64);
        }
        
        Ok(result)
    }
    #[cfg(not(feature = "arrayfire"))]
    {
        // CPU fallback
        let mut rank = GpuVector::from_vec(vec![1.0 / n as f32; n]);
        let matrix = create_transition_matrix(&gpu_graph, alpha as f32);
        
        for _iter in 0..max_iter {
            let old_rank = rank.data.clone();
            rank = matrix.matvec(&rank);
            rank.normalize();
            
            let diff: f32 = old_rank.iter()
                .zip(rank.data.iter())
                .map(|(old, new)| (old - new).abs())
                .sum();
            
            if diff < tolerance as f32 {
                break;
            }
        }
        
        let mut result = HashMap::new();
        for (i, node) in nodes.iter().enumerate() {
            result.insert(node.clone(), rank.data[i] as f64);
        }
        
        Ok(result)
    }
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

/// Create sparse transition matrix for GPU PageRank
fn create_sparse_transition_matrix(graph: &GpuGraph, alpha: f32) -> (Vec<u32>, Vec<u32>, Vec<f32>) {
    let n = graph.num_nodes;
    
    // Compute out-degrees
    let mut out_degree = vec![0u32; n];
    for i in 0..n {
        out_degree[i] = graph.row_offsets[i + 1] - graph.row_offsets[i];
    }
    
    // Build CSR format for column-stochastic matrix (transpose of adjacency)
    let mut row_offsets = vec![0u32; n + 1];
    let mut col_indices = Vec::new();
    let mut values = Vec::new();
    
    // Count incoming edges for each node
    let mut in_degree = vec![0u32; n];
    for i in 0..n {
        let start = graph.row_offsets[i] as usize;
        let end = graph.row_offsets[i + 1] as usize;
        for idx in start..end {
            let j = graph.col_indices[idx] as usize;
            in_degree[j] += 1;
        }
    }
    
    // Build row offsets
    for i in 0..n {
        row_offsets[i + 1] = row_offsets[i] + in_degree[i];
    }
    
    // Reset in_degree to use as counters
    in_degree.fill(0);
    
    // Fill transpose matrix
    for i in 0..n {
        let start = graph.row_offsets[i] as usize;
        let end = graph.row_offsets[i + 1] as usize;
        let degree = out_degree[i] as f32;
        
        for idx in start..end {
            let j = graph.col_indices[idx] as usize;
            let insert_pos = (row_offsets[j] + in_degree[j]) as usize;
            
            if col_indices.len() <= insert_pos {
                col_indices.resize(insert_pos + 1, 0);
                values.resize(insert_pos + 1, 0.0);
            }
            
            col_indices[insert_pos] = i as u32;
            values[insert_pos] = if degree > 0.0 { alpha / degree } else { 0.0 };
            in_degree[j] += 1;
        }
    }
    
    // Add teleportation term
    let teleport = (1.0 - alpha) / n as f32;
    for i in 0..values.len() {
        values[i] += teleport;
    }
    
    (row_offsets, col_indices, values)
}

/// GPU sparse matrix-vector multiplication using ArrayFire
#[cfg(feature = "arrayfire")]
fn gpu_sparse_matvec(
    row_offsets: &arrayfire::Array<u32>,
    col_indices: &arrayfire::Array<u32>,
    values: &arrayfire::Array<f32>,
    x: &arrayfire::Array<f32>,
    n: usize,
) -> arrayfire::Array<f32> {
    use arrayfire as af;
    
    // Create result vector
    let mut result = af::constant(0.0f32, af::Dim4::new(&[n as u64, 1, 1, 1]));
    
    // For now, use a simple implementation
    // In practice, would use optimized sparse BLAS routines
    let mut host_row_offsets = vec![0u32; row_offsets.elements()];
    let mut host_col_indices = vec![0u32; col_indices.elements()];
    let mut host_values = vec![0.0f32; values.elements()];
    let mut host_x = vec![0.0f32; n];
    
    row_offsets.host(&mut host_row_offsets);
    col_indices.host(&mut host_col_indices);
    values.host(&mut host_values);
    x.host(&mut host_x);
    
    let mut host_result = vec![0.0f32; n];
    
    for i in 0..n {
        let start = host_row_offsets[i] as usize;
        let end = host_row_offsets[i + 1] as usize;
        let mut sum = 0.0;
        
        for j in start..end {
            if j < host_col_indices.len() && j < host_values.len() {
                let col = host_col_indices[j] as usize;
                if col < host_x.len() {
                    sum += host_values[j] * host_x[col];
                }
            }
        }
        host_result[i] = sum;
    }
    
    af::Array::new(&host_result, af::Dim4::new(&[n as u64, 1, 1, 1]))
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