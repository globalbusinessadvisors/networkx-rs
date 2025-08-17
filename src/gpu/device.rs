//! GPU device management

use std::fmt;

/// Represents a GPU device
#[derive(Debug, Clone)]
pub struct GpuDevice {
    pub id: usize,
    pub name: String,
    pub memory_gb: f32,
    pub compute_capability: (u32, u32),
}

impl fmt::Display for GpuDevice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "GPU {}: {} ({:.1} GB, Compute {}.{})",
            self.id, self.name, self.memory_gb, 
            self.compute_capability.0, self.compute_capability.1
        )
    }
}

/// Information about a GPU device
pub struct DeviceInfo {
    pub total_memory: usize,
    pub free_memory: usize,
    pub multiprocessor_count: u32,
    pub max_threads_per_block: u32,
    pub warp_size: u32,
}

/// Get the number of available GPU devices
pub fn get_device_count() -> usize {
    #[cfg(feature = "arrayfire")]
    {
        use arrayfire as af;
        af::device_count() as usize
    }
    #[cfg(not(feature = "arrayfire"))]
    {
        0
    }
}

/// Get information about all available devices
pub fn get_all_devices() -> Vec<GpuDevice> {
    #[cfg(feature = "arrayfire")]
    {
        use arrayfire as af;
        let count = af::device_count();
        let mut devices = Vec::new();
        
        for i in 0..count {
            af::set_device(i);
            let info = af::device_info();
            
            devices.push(GpuDevice {
                id: i as usize,
                name: info.0,
                memory_gb: (info.3 as f32) / (1024.0 * 1024.0 * 1024.0),
                compute_capability: (info.1 as u32, info.2 as u32),
            });
        }
        
        devices
    }
    #[cfg(not(feature = "arrayfire"))]
    {
        Vec::new()
    }
}

/// Select the best available GPU device
pub fn select_device() -> Option<GpuDevice> {
    let devices = get_all_devices();
    
    // Select device with most memory
    devices.into_iter()
        .max_by(|a, b| a.memory_gb.partial_cmp(&b.memory_gb).unwrap())
}

/// Set the active GPU device
pub fn set_device(device_id: usize) -> Result<(), String> {
    #[cfg(feature = "arrayfire")]
    {
        use arrayfire as af;
        if device_id >= af::device_count() as usize {
            return Err(format!("Invalid device ID: {}", device_id));
        }
        af::set_device(device_id as i32);
        Ok(())
    }
    #[cfg(not(feature = "arrayfire"))]
    {
        Err("GPU support not compiled".to_string())
    }
}

/// Get memory information for the current device
pub fn get_memory_info() -> Result<(usize, usize), String> {
    #[cfg(feature = "arrayfire")]
    {
        use arrayfire as af;
        let (free, total) = af::device_mem_info();
        Ok((free, total))
    }
    #[cfg(not(feature = "arrayfire"))]
    {
        Err("GPU support not compiled".to_string())
    }
}