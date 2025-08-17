//! Distributed computing module for large-scale graph processing

#[cfg(feature = "distributed")]
pub mod partition;
#[cfg(feature = "distributed")]
pub mod worker;
#[cfg(feature = "distributed")]
pub mod coordinator;
#[cfg(feature = "distributed")]
pub mod algorithms;

#[cfg(feature = "distributed")]
pub use partition::{GraphPartitioner, PartitionStrategy, partition_graph};
#[cfg(feature = "distributed")]
pub use worker::{Worker, WorkerConfig};
#[cfg(feature = "distributed")]
pub use coordinator::{Coordinator, Job, JobResult};
#[cfg(feature = "distributed")]
pub use algorithms::{distributed_pagerank, distributed_bfs, distributed_connected_components};

/// Configuration for distributed graph processing
#[derive(Debug, Clone)]
pub struct DistributedConfig {
    /// Number of worker nodes
    pub num_workers: usize,
    /// Maximum message size in bytes
    pub max_message_size: usize,
    /// Timeout for operations in seconds
    pub timeout_secs: u64,
    /// Enable compression for messages
    pub enable_compression: bool,
}