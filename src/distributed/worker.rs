//! Distributed worker implementation

use super::partition::Partition;
use crate::errors::Result;
use std::collections::HashMap;
use std::hash::Hash;

/// Worker configuration
#[derive(Debug, Clone)]
pub struct WorkerConfig {
    pub worker_id: usize,
    pub coordinator_addr: String,
    pub listen_port: u16,
    pub max_memory_mb: usize,
}

impl Default for WorkerConfig {
    fn default() -> Self {
        WorkerConfig {
            worker_id: 0,
            coordinator_addr: "localhost:50051".to_string(),
            listen_port: 50052,
            max_memory_mb: 4096, // 4GB default
        }
    }
}

/// Worker status
#[derive(Debug, Clone, PartialEq)]
pub enum WorkerStatus {
    Idle,
    Computing,
    Communicating,
    Error(String),
}

/// Message types for worker communication
#[derive(Debug, Clone)]
pub enum WorkerMessage<N> {
    AssignPartition(Partition<N>),
    ComputePageRank { 
        alpha: f64, 
        current_values: HashMap<N, f64> 
    },
    ComputeBFS { 
        frontier: Vec<N>, 
        level: usize 
    },
    Shutdown,
}

/// Worker response types
#[derive(Debug, Clone)]
pub enum WorkerResponse<N> {
    PartitionAssigned,
    PageRankResult(HashMap<N, f64>),
    BFSResult { 
        next_frontier: Vec<N>, 
        distances: HashMap<N, usize> 
    },
    Error(String),
    Acknowledged,
}

/// Distributed worker
pub struct Worker<N> {
    config: WorkerConfig,
    partition: Option<Partition<N>>,
    status: WorkerStatus,
    message_buffer: Vec<WorkerMessage<N>>,
}

impl<N> Worker<N> 
where
    N: Clone + Hash + Eq,
{
    /// Create a new worker
    pub fn new(config: WorkerConfig) -> Self {
        Worker {
            config,
            partition: None,
            status: WorkerStatus::Idle,
            message_buffer: Vec::new(),
        }
    }
    
    /// Start the worker (simplified implementation)
    pub async fn start(&mut self) -> Result<(), crate::errors::NetworkXError> {
        println!("Worker {} starting on port {}", self.config.worker_id, self.config.listen_port);
        
        #[cfg(feature = "distributed")]
        {
            // In a real implementation, this would start a gRPC server
            // For now, just simulate the worker being ready
            self.status = WorkerStatus::Idle;
            println!("Worker {} ready to accept connections", self.config.worker_id);
            Ok(())
        }
        #[cfg(not(feature = "distributed"))]
        {
            Err(crate::errors::NetworkXError::ComputationError(
                "Distributed support not compiled".to_string()
            ))
        }
    }
    
    /// Assign a partition to this worker
    pub fn assign_partition(&mut self, partition: Partition<N>) {
        self.partition = Some(partition);
        self.status = WorkerStatus::Idle;
        println!("Worker {} assigned partition with {} nodes", 
                 self.config.worker_id, 
                 self.partition.as_ref().map(|p| p.nodes.len()).unwrap_or(0));
    }
    
    /// Process a work message
    pub async fn process_message(&mut self, message: WorkerMessage<N>) -> WorkerResponse<N> {
        match message {
            WorkerMessage::AssignPartition(partition) => {
                self.assign_partition(partition);
                WorkerResponse::PartitionAssigned
            },
            
            WorkerMessage::ComputePageRank { alpha, current_values } => {
                self.status = WorkerStatus::Computing;
                
                match &self.partition {
                    Some(partition) => {
                        let result = self.compute_pagerank_local(partition, &current_values, alpha);
                        self.status = WorkerStatus::Idle;
                        WorkerResponse::PageRankResult(result)
                    },
                    None => {
                        self.status = WorkerStatus::Error("No partition assigned".to_string());
                        WorkerResponse::Error("No partition assigned".to_string())
                    }
                }
            },
            
            WorkerMessage::ComputeBFS { frontier, level } => {
                self.status = WorkerStatus::Computing;
                
                match &self.partition {
                    Some(partition) => {
                        let (next_frontier, distances) = self.compute_bfs_local(partition, &frontier, level);
                        self.status = WorkerStatus::Idle;
                        WorkerResponse::BFSResult { next_frontier, distances }
                    },
                    None => {
                        self.status = WorkerStatus::Error("No partition assigned".to_string());
                        WorkerResponse::Error("No partition assigned".to_string())
                    }
                }
            },
            
            WorkerMessage::Shutdown => {
                self.status = WorkerStatus::Idle;
                WorkerResponse::Acknowledged
            }
        }
    }
    
    /// Compute PageRank locally for this worker's partition
    fn compute_pagerank_local(
        &self,
        partition: &Partition<N>,
        current_values: &HashMap<N, f64>,
        alpha: f64,
    ) -> HashMap<N, f64> {
        let mut result = HashMap::new();
        let total_nodes = current_values.len();
        let teleport = (1.0 - alpha) / total_nodes as f64;
        
        // Process internal edges
        for (u, v, _weight) in &partition.internal_edges {
            let u_rank = current_values.get(u).copied().unwrap_or(0.0);
            let u_degree = partition.internal_edges.iter()
                .filter(|(source, _, _)| source == u)
                .count() as f64;
            
            if u_degree > 0.0 {
                *result.entry(v.clone()).or_insert(teleport) += alpha * u_rank / u_degree;
            }
        }
        
        // Process cut edges
        for (u, v, _weight) in &partition.cut_edges {
            if partition.nodes.contains(v) {
                let u_rank = current_values.get(u).copied().unwrap_or(0.0);
                *result.entry(v.clone()).or_insert(teleport) += alpha * u_rank;
            }
        }
        
        // Ensure all nodes in partition have values
        for node in &partition.nodes {
            result.entry(node.clone()).or_insert(teleport);
        }
        
        result
    }
    
    /// Compute BFS locally for this worker's partition
    fn compute_bfs_local(
        &self,
        partition: &Partition<N>,
        frontier: &[N],
        level: usize,
    ) -> (Vec<N>, HashMap<N, usize>) {
        let mut next_frontier = Vec::new();
        let mut distances = HashMap::new();
        
        let frontier_set: std::collections::HashSet<_> = frontier.iter().collect();
        
        // Check internal edges
        for (u, v, _weight) in &partition.internal_edges {
            if frontier_set.contains(u) && partition.nodes.contains(v) {
                if !distances.contains_key(v) {
                    distances.insert(v.clone(), level);
                    next_frontier.push(v.clone());
                }
            }
        }
        
        // Check cut edges
        for (u, v, _weight) in &partition.cut_edges {
            if frontier_set.contains(u) && partition.nodes.contains(v) {
                if !distances.contains_key(v) {
                    distances.insert(v.clone(), level);
                    next_frontier.push(v.clone());
                }
            }
        }
        
        (next_frontier, distances)
    }
    
    /// Get worker status
    pub fn status(&self) -> &WorkerStatus {
        &self.status
    }
    
    /// Get worker configuration
    pub fn config(&self) -> &WorkerConfig {
        &self.config
    }
    
    /// Get partition info
    pub fn partition_info(&self) -> Option<(usize, usize, usize)> {
        self.partition.as_ref().map(|p| {
            (p.nodes.len(), p.internal_edges.len(), p.cut_edges.len())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::distributed::partition::*;
    use std::collections::HashSet;

    #[tokio::test]
    async fn test_worker_creation() {
        let config = WorkerConfig::default();
        let worker: Worker<i32> = Worker::new(config);
        
        assert_eq!(worker.status(), &WorkerStatus::Idle);
        assert!(worker.partition.is_none());
    }
    
    #[tokio::test]
    async fn test_worker_partition_assignment() {
        let config = WorkerConfig::default();
        let mut worker = Worker::new(config);
        
        let mut partition = Partition::new();
        partition.add_node(1);
        partition.add_node(2);
        partition.add_internal_edge(1, 2, 1.0);
        
        let message = WorkerMessage::AssignPartition(partition);
        let response = worker.process_message(message).await;
        
        match response {
            WorkerResponse::PartitionAssigned => {},
            _ => panic!("Expected PartitionAssigned response"),
        }
        
        assert_eq!(worker.partition_info(), Some((2, 1, 0)));
    }
    
    #[tokio::test]
    async fn test_worker_pagerank_computation() {
        let config = WorkerConfig::default();
        let mut worker = Worker::new(config);
        
        let mut partition = Partition::new();
        partition.add_node(1);
        partition.add_node(2);
        partition.add_internal_edge(1, 2, 1.0);
        
        worker.assign_partition(partition);
        
        let mut current_values = HashMap::new();
        current_values.insert(1, 0.5);
        current_values.insert(2, 0.5);
        
        let message = WorkerMessage::ComputePageRank {
            alpha: 0.85,
            current_values,
        };
        
        let response = worker.process_message(message).await;
        
        match response {
            WorkerResponse::PageRankResult(result) => {
                assert!(result.contains_key(&1));
                assert!(result.contains_key(&2));
            },
            _ => panic!("Expected PageRankResult response"),
        }
    }
}