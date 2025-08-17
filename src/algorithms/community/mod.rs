//! Community detection algorithms

pub mod louvain;
pub mod label_propagation;
pub mod modularity;

pub use louvain::louvain_communities;
pub use label_propagation::label_propagation_communities;
pub use modularity::{modularity, modularity_matrix};