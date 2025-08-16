//! Benchmark for Dijkstra's algorithm

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use networkx_rs_core::graph::{Graph, DiGraph};
use networkx_rs_core::graph::traits::GraphMut;
use networkx_rs_core::algorithms::paths::dijkstra_path;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

fn create_random_graph(nodes: usize, edge_probability: f64, seed: u64) -> Graph {
    let mut graph = Graph::with_capacity(nodes);
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    
    // Add nodes
    for _ in 0..nodes {
        graph.add_node();
    }
    
    // Add edges with random weights
    for i in 0..nodes {
        for j in i+1..nodes {
            if rng.gen::<f64>() < edge_probability {
                let weight = rng.gen_range(1.0..10.0);
                graph.add_edge(i, j, weight);
            }
        }
    }
    
    graph
}

fn bench_dijkstra(c: &mut Criterion) {
    let mut group = c.benchmark_group("dijkstra");
    
    for size in [100, 500, 1000, 5000].iter() {
        let graph = create_random_graph(*size, 0.1, 42);
        
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &graph,
            |b, graph| {
                b.iter(|| {
                    dijkstra_path(graph, black_box(0), black_box(size - 1))
                });
            },
        );
    }
    
    group.finish();
}

fn bench_dijkstra_dense(c: &mut Criterion) {
    let mut group = c.benchmark_group("dijkstra_dense");
    
    for size in [50, 100, 200, 500].iter() {
        let graph = create_random_graph(*size, 0.5, 42);
        
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &graph,
            |b, graph| {
                b.iter(|| {
                    dijkstra_path(graph, black_box(0), black_box(size - 1))
                });
            },
        );
    }
    
    group.finish();
}

criterion_group!(benches, bench_dijkstra, bench_dijkstra_dense);
criterion_main!(benches);