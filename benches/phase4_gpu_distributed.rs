use criterion::{black_box, criterion_group, criterion_main, Criterion};
use networkx_rs::graph::Graph;
use networkx_rs::algorithms::centrality::pagerank;

#[cfg(feature = "gpu")]
use networkx_rs::gpu::{gpu_pagerank, init_gpu, is_gpu_available};

#[cfg(feature = "distributed")]
use networkx_rs::distributed::{distributed_pagerank, init_distributed, is_distributed_available};

fn create_test_graph(n: usize) -> Graph<usize> {
    let mut graph = Graph::new();
    
    // Create a scale-free graph
    for i in 0..n {
        for j in 0..std::cmp::min(i, 5) {
            if i != j {
                graph.add_edge(i, j, Some(1.0));
            }
        }
    }
    
    graph
}

fn benchmark_cpu_pagerank(c: &mut Criterion) {
    let graph = create_test_graph(1000);
    
    c.bench_function("cpu_pagerank_1k", |b| {
        b.iter(|| {
            pagerank(black_box(&graph), Some(0.85), Some(100), Some(1e-6))
        })
    });
}

#[cfg(feature = "gpu")]
fn benchmark_gpu_pagerank(c: &mut Criterion) {
    if !is_gpu_available() {
        println!("GPU not available, skipping GPU benchmarks");
        return;
    }
    
    if let Err(e) = init_gpu() {
        println!("Failed to initialize GPU: {:?}", e);
        return;
    }
    
    let graph = create_test_graph(1000);
    
    c.bench_function("gpu_pagerank_1k", |b| {
        b.iter(|| {
            gpu_pagerank(black_box(&graph), 0.85, 100, 1e-6)
        })
    });
}

#[cfg(feature = "distributed")]
fn benchmark_distributed_pagerank(c: &mut Criterion) {
    if !is_distributed_available() {
        println!("Distributed support not available, skipping distributed benchmarks");
        return;
    }
    
    let rt = tokio::runtime::Runtime::new().unwrap();
    let graph = create_test_graph(1000);
    
    c.bench_function("distributed_pagerank_1k", |b| {
        b.iter(|| {
            rt.block_on(distributed_pagerank(
                black_box(&graph), 
                4, 
                Some(0.85), 
                Some(100), 
                Some(1e-6)
            ))
        })
    });
}

fn benchmark_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("pagerank_comparison");
    
    let graph = create_test_graph(500);
    
    group.bench_function("cpu", |b| {
        b.iter(|| {
            pagerank(black_box(&graph), Some(0.85), Some(100), Some(1e-6))
        })
    });
    
    #[cfg(feature = "gpu")]
    {
        if is_gpu_available() && init_gpu().is_ok() {
            group.bench_function("gpu", |b| {
                b.iter(|| {
                    gpu_pagerank(black_box(&graph), 0.85, 100, 1e-6)
                })
            });
        }
    }
    
    #[cfg(feature = "distributed")]
    {
        if is_distributed_available() {
            let rt = tokio::runtime::Runtime::new().unwrap();
            group.bench_function("distributed", |b| {
                b.iter(|| {
                    rt.block_on(distributed_pagerank(
                        black_box(&graph), 
                        2, 
                        Some(0.85), 
                        Some(100), 
                        Some(1e-6)
                    ))
                })
            });
        }
    }
    
    group.finish();
}

fn benchmark_scalability(c: &mut Criterion) {
    let mut group = c.benchmark_group("scalability");
    
    for size in [100, 500, 1000].iter() {
        let graph = create_test_graph(*size);
        
        group.bench_with_input(
            format!("cpu_{}", size), 
            size, 
            |b, _| {
                b.iter(|| {
                    pagerank(black_box(&graph), Some(0.85), Some(50), Some(1e-6))
                })
            }
        );
        
        #[cfg(feature = "gpu")]
        {
            if is_gpu_available() && init_gpu().is_ok() {
                group.bench_with_input(
                    format!("gpu_{}", size), 
                    size, 
                    |b, _| {
                        b.iter(|| {
                            gpu_pagerank(black_box(&graph), 0.85, 50, 1e-6)
                        })
                    }
                );
            }
        }
    }
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_cpu_pagerank,
    #[cfg(feature = "gpu")]
    benchmark_gpu_pagerank,
    #[cfg(feature = "distributed")]
    benchmark_distributed_pagerank,
    benchmark_comparison,
    benchmark_scalability
);

criterion_main!(benches);