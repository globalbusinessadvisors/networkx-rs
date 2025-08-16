//! NetworkX-RS: High-performance graph algorithms in Rust
//!
//! This library provides Rust implementations of NetworkX's core algorithms
//! with Python bindings for seamless integration.

#![cfg_attr(feature = "simd", feature(portable_simd))]

pub mod graph;
pub mod algorithms;
pub mod utils;
pub mod errors;

#[cfg(feature = "pyo3")]
pub mod python;

// Re-exports for convenience
pub use graph::{Graph, DiGraph};
pub use errors::{NetworkXError, Result};

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

/// Python module initialization
#[cfg(feature = "pyo3")]
#[pymodule]
fn networkx_rs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    Python::with_gil(|py| {
        // Register Python bindings
        python::register_module(py, m)?;
        
        // Version information
        m.add("__version__", env!("CARGO_PKG_VERSION"))?;
        m.add("__rust_version__", env!("CARGO_PKG_VERSION"))?;
        
        Ok(())
    })
}