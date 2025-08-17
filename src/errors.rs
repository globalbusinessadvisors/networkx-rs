//! Error types for NetworkX-RS

use std::fmt;

#[derive(Debug, Clone)]
pub enum NetworkXError {
    NodeNotFound(String),
    EdgeNotFound(String),
    NegativeWeight,
    NegativeCycle(String),
    GraphCycle,
    InvalidInput(String),
    AlgorithmError(String),
    ComputationError(String),
}

impl fmt::Display for NetworkXError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NetworkXError::NodeNotFound(msg) => write!(f, "Node not found: {}", msg),
            NetworkXError::EdgeNotFound(msg) => write!(f, "Edge not found: {}", msg),
            NetworkXError::NegativeWeight => write!(f, "Negative edge weight not allowed"),
            NetworkXError::NegativeCycle(msg) => write!(f, "Negative cycle detected: {}", msg),
            NetworkXError::GraphCycle => write!(f, "Graph contains a cycle"),
            NetworkXError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            NetworkXError::AlgorithmError(msg) => write!(f, "Algorithm error: {}", msg),
            NetworkXError::ComputationError(msg) => write!(f, "Computation error: {}", msg),
        }
    }
}

impl std::error::Error for NetworkXError {}

pub type Result<T> = std::result::Result<T, NetworkXError>;