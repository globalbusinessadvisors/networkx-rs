//! Graph generators for creating various types of graphs

pub mod random;
pub mod small_world;
pub mod scale_free;
pub mod classic;

pub use random::{erdos_renyi, fast_gnp_random_graph, gnm_random_graph};
pub use small_world::{watts_strogatz, newman_watts_strogatz, connected_watts_strogatz};
pub use scale_free::{barabasi_albert, extended_barabasi_albert, powerlaw_cluster};
pub use classic::{complete_graph, cycle_graph, path_graph, star_graph, wheel_graph};