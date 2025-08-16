//! Python bindings using PyO3

use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;
use std::collections::HashMap;
use crate::graph::{Graph as RustGraph, DiGraph as RustDiGraph};
use crate::graph::traits::GraphBase;
use crate::algorithms::paths;

/// Python wrapper for undirected graph
#[pyclass]
pub struct PyGraph {
    inner: RustGraph<i32>,
}

#[pymethods]
impl PyGraph {
    #[new]
    pub fn new() -> Self {
        PyGraph {
            inner: RustGraph::new(),
        }
    }
    
    pub fn add_node(&mut self, node: i32) {
        self.inner.add_node(node);
    }
    
    #[pyo3(signature = (source, target, weight=None))]
    pub fn add_edge(&mut self, source: i32, target: i32, weight: Option<f64>) {
        self.inner.add_edge(source, target, weight);
    }
    
    pub fn has_node(&self, node: i32) -> bool {
        self.inner.has_node(&node)
    }
    
    pub fn has_edge(&self, source: i32, target: i32) -> bool {
        self.inner.has_edge(&source, &target)
    }
    
    pub fn node_count(&self) -> usize {
        self.inner.node_count()
    }
    
    pub fn edge_count(&self) -> usize {
        self.inner.edge_count()
    }
    
    pub fn nodes(&self) -> Vec<i32> {
        self.inner.nodes().collect()
    }
    
    pub fn edges(&self) -> Vec<(i32, i32, f64)> {
        self.inner.edges().collect()
    }
    
    pub fn neighbors(&self, node: i32) -> Vec<i32> {
        self.inner.neighbors(&node).collect()
    }
    
    pub fn degree(&self, node: i32) -> usize {
        self.inner.degree(&node)
    }
}

/// Python wrapper for directed graph
#[pyclass]
pub struct PyDiGraph {
    inner: RustDiGraph<i32>,
}

#[pymethods]
impl PyDiGraph {
    #[new]
    pub fn new() -> Self {
        PyDiGraph {
            inner: RustDiGraph::new(),
        }
    }
    
    pub fn add_node(&mut self, node: i32) {
        self.inner.add_node(node);
    }
    
    #[pyo3(signature = (source, target, weight=None))]
    pub fn add_edge(&mut self, source: i32, target: i32, weight: Option<f64>) {
        self.inner.add_edge(source, target, weight);
    }
    
    pub fn has_node(&self, node: i32) -> bool {
        self.inner.has_node(&node)
    }
    
    pub fn has_edge(&self, source: i32, target: i32) -> bool {
        self.inner.has_edge(&source, &target)
    }
    
    pub fn node_count(&self) -> usize {
        self.inner.node_count()
    }
    
    pub fn edge_count(&self) -> usize {
        self.inner.edge_count()
    }
    
    pub fn nodes(&self) -> Vec<i32> {
        self.inner.nodes().collect()
    }
    
    pub fn edges(&self) -> Vec<(i32, i32, f64)> {
        self.inner.edges().collect()
    }
    
    pub fn neighbors(&self, node: i32) -> Vec<i32> {
        self.inner.neighbors(&node).collect()
    }
    
    pub fn in_degree(&self, node: i32) -> usize {
        // Count incoming edges
        self.inner.edges()
            .filter(|(_, target, _)| *target == node)
            .count()
    }
    
    pub fn out_degree(&self, node: i32) -> usize {
        self.inner.degree(&node)
    }
}

/// Register Python functions for path algorithms
#[pyfunction]
fn dijkstra_path(graph: &PyGraph, source: i32, target: i32) -> PyResult<Option<(Vec<i32>, f64)>> {
    paths::dijkstra_path(&graph.inner, source, target, None::<fn(&i32, &i32) -> f64>)
        .map_err(|e| PyValueError::new_err(e.to_string()))
}

#[pyfunction]
fn dijkstra_path_digraph(graph: &PyDiGraph, source: i32, target: i32) -> PyResult<Option<(Vec<i32>, f64)>> {
    paths::dijkstra_path(&graph.inner, source, target, None::<fn(&i32, &i32) -> f64>)
        .map_err(|e| PyValueError::new_err(e.to_string()))
}

#[pyfunction]
fn astar_path(
    graph: &PyGraph, 
    source: i32, 
    target: i32,
    heuristic: PyObject,
) -> PyResult<Option<(Vec<i32>, f64)>> {
    Python::with_gil(|py| {
        let h = |node: &i32| -> f64 {
            heuristic.call1(py, (*node,))
                .and_then(|r| r.extract::<f64>(py))
                .unwrap_or(0.0)
        };
        
        paths::astar_path(&graph.inner, source, target, h, None::<fn(&i32, &i32) -> f64>)
            .map_err(|e| PyValueError::new_err(e.to_string()))
    })
}

#[pyfunction]
fn bellman_ford(graph: &PyDiGraph, source: i32) -> PyResult<(HashMap<i32, f64>, HashMap<i32, i32>)> {
    paths::bellman_ford(&graph.inner, source, None::<fn(&i32, &i32) -> f64>)
        .map_err(|e| PyValueError::new_err(e.to_string()))
}

#[pyfunction]
fn floyd_warshall(graph: &PyDiGraph) -> PyResult<HashMap<(i32, i32), f64>> {
    let (distances, _) = paths::floyd_warshall(&graph.inner, None::<fn(&i32, &i32) -> f64>)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    Ok(distances)
}

#[pyfunction]
fn k_shortest_paths(
    graph: &PyGraph,
    source: i32,
    target: i32,
    k: usize,
) -> PyResult<Vec<(Vec<i32>, f64)>> {
    paths::k_shortest_paths(&graph.inner, source, target, k, None::<fn(&i32, &i32) -> f64>)
        .map_err(|e| PyValueError::new_err(e.to_string()))
}

/// Register all Python bindings
pub fn register_module(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyGraph>()?;
    m.add_class::<PyDiGraph>()?;
    
    m.add_function(wrap_pyfunction!(dijkstra_path, m)?)?;
    m.add_function(wrap_pyfunction!(dijkstra_path_digraph, m)?)?;
    m.add_function(wrap_pyfunction!(astar_path, m)?)?;
    m.add_function(wrap_pyfunction!(bellman_ford, m)?)?;
    m.add_function(wrap_pyfunction!(floyd_warshall, m)?)?;
    m.add_function(wrap_pyfunction!(k_shortest_paths, m)?)?;
    
    Ok(())
}