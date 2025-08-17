//! Distributed computing module

pub mod partition;
pub mod worker;
pub mod coordinator;
pub mod algorithms;

pub use partition::{GraphPartitioner, PartitionStrategy, Partition, partition_graph};
pub use worker::{Worker, WorkerConfig};
pub use coordinator::{Coordinator, Job, JobResult};
pub use algorithms::{distributed_pagerank, distributed_bfs, distributed_connected_components};

use crate::errors::Result;

/// Initialize distributed computing subsystem
pub fn init_distributed() -> Result<(), crate::errors::NetworkXError> {
    #[cfg(feature = "distributed")]
    {
        // Initialize gRPC runtime
        println!("Distributed computing subsystem initialized");
        Ok(())
    }
    #[cfg(not(feature = "distributed"))]
    {
        Err(crate::errors::NetworkXError::ComputationError(
            "Distributed support not compiled. Enable 'distributed' feature".to_string()
        ))
    }
}

/// Check if distributed computing is available
pub fn is_distributed_available() -> bool {
    #[cfg(feature = "distributed")]
    {
        true
    }
    #[cfg(not(feature = "distributed"))]
    {
        false
    }
}

/// Get distributed computing information
pub fn get_distributed_info() -> String {
    #[cfg(feature = "distributed")]
    {
        "Distributed computing support available with gRPC communication".to_string()
    }
    #[cfg(not(feature = "distributed"))]
    {
        "Distributed support not compiled. Enable 'distributed' feature".to_string()
    }
}
