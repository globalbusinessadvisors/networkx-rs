//! Distributed worker implementation

use super::partition::Partition;

/// Worker configuration
#[derive(Debug, Clone)]
pub struct WorkerConfig {
    pub worker_id: usize,
    pub coordinator_addr: String,
    pub listen_port: u16,
}

/// Distributed worker
pub struct Worker<N> {
    config: WorkerConfig,
    partition: Option<Partition<N>>,
}

impl<N> Worker<N> {
    /// Create a new worker
    pub fn new(config: WorkerConfig) -> Self {
        Worker {
            config,
            partition: None,
        }
    }
    
    /// Start the worker
    pub async fn start(&mut self) -> Result<(), String> {
        // Would implement gRPC server here
        todo!("Worker implementation pending")
    }
}