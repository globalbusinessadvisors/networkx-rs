//! GPU acceleration module for graph algorithms

#[cfg(feature = "gpu")]
pub mod device;
#[cfg(feature = "gpu")]
pub mod kernels;
#[cfg(feature = "gpu")]
pub mod memory;
#[cfg(feature = "gpu")]
pub mod algorithms;

#[cfg(feature = "gpu")]
pub use device::{GpuDevice, DeviceInfo, select_device};
#[cfg(feature = "gpu")]
pub use memory::{GpuMatrix, GpuVector, GpuGraph};
#[cfg(feature = "gpu")]
pub use algorithms::{gpu_pagerank, gpu_bfs, gpu_shortest_paths};

/// Check if GPU support is available
pub fn is_gpu_available() -> bool {
    #[cfg(feature = "gpu")]
    {
        device::get_device_count() > 0
    }
    #[cfg(not(feature = "gpu"))]
    {
        false
    }
}

/// Get information about available GPUs
pub fn get_gpu_info() -> Vec<String> {
    #[cfg(feature = "gpu")]
    {
        device::get_all_devices()
            .iter()
            .map(|d| d.to_string())
            .collect()
    }
    #[cfg(not(feature = "gpu"))]
    {
        vec!["GPU support not compiled".to_string()]
    }
}