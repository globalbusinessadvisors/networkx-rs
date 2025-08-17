//! GPU acceleration module

#[cfg(feature = "gpu")]
pub mod kernels {
    pub fn gpu_pagerank() {
        unimplemented!("GPU PageRank not yet implemented")
    }
}

pub fn init_gpu() -> Result<(), String> {
    Ok(())
}
