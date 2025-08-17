//! Connectivity algorithms for graphs

pub mod components;
pub mod cuts;
pub mod paths;
pub mod bipartite;

pub use components::{
    connected_components,
    strongly_connected_components,
    weakly_connected_components,
    is_connected,
    is_strongly_connected,
    number_connected_components,
};

pub use cuts::{
    minimum_cut,
    minimum_edge_cut,
    node_connectivity,
    edge_connectivity,
    is_k_edge_connected,
};

pub use paths::{
    has_path,
    node_disjoint_paths,
    edge_disjoint_paths,
};

pub use bipartite::{
    is_bipartite,
    bipartite_sets,
    bipartite_color,
};