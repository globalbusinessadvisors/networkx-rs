//! GPU acceleration module

pub mod device;
pub mod memory;
pub mod algorithms;
pub mod kernels;

pub use device::{GpuDevice, get_device_count, get_all_devices, select_device, set_device};
pub use memory::{GpuGraph, GpuMatrix, GpuVector};
pub use algorithms::{gpu_pagerank, gpu_bfs, gpu_shortest_paths, gpu_eigenvector_centrality};

use crate::errors::Result;

/// Initialize GPU subsystem
pub fn init_gpu() -> Result<(), crate::errors::NetworkXError> {
    #[cfg(feature = "arrayfire")]
    {
        use arrayfire as af;
        
        // Initialize ArrayFire
        let device_count = af::device_count();
        if device_count == 0 {
            return Err(crate::errors::NetworkXError::ComputationError(
                "No GPU devices available".to_string()
            ));
        }
        
        // Set to first device by default
        af::set_device(0);
        println!("GPU initialized with {} device(s)", device_count);
    }
    #[cfg(not(feature = "arrayfire"))]
    {
        return Err(crate::errors::NetworkXError::ComputationError(
            "GPU support not compiled. Enable 'gpu' feature".to_string()
        ));
    }
    
    Ok(())
}

/// Check if GPU support is available
pub fn is_gpu_available() -> bool {
    #[cfg(feature = "arrayfire")]
    {
        arrayfire::device_count() > 0
    }
    #[cfg(not(feature = "arrayfire"))]
    {
        false
    }
}

/// Get GPU device information
pub fn get_gpu_info() -> String {
    #[cfg(feature = "arrayfire")]
    {
        if !is_gpu_available() {
            return "No GPU devices available".to_string();
        }
        
        let devices = get_all_devices();
        let mut info = format!("GPU Devices ({}):\n", devices.len());
        
        for device in devices {
            info.push_str(&format!("  {}\n", device));
        }
        
        info
    }
    #[cfg(not(feature = "arrayfire"))]
    {
        "GPU support not compiled. Enable 'gpu' feature".to_string()
    }
}
