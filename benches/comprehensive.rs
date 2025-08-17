//! Comprehensive benchmarking suite for NetworkX-RS

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use networkx_rs_core::graph::{Graph, DiGraph};
use networkx_rs_core::algorithms;
use std::time::Duration;

/// Generate a random graph for benchmarking
fn generate_random_graph(n: usize, p: f64) -> Graph<usize> {
    algorithms::generators::erdos_renyi(n, p, false, Some(42))
        .expect("Failed to generate graph")
}

/// Generate a scale-free graph
fn generate_scale_free_graph(n: usize, m: usize) -> Graph<usize> {
    algorithms::generators::barabasi_albert(n, m, Some(42))
        .expect("Failed to generate graph")
}

/// Benchmark shortest path algorithms
fn bench_shortest_paths(c: &mut Criterion) {
    let mut group = c.benchmark_group("shortest_paths");
    group.measurement_time(Duration::from_secs(10));
    
    for size in [100, 500, 1000].iter() {
        let graph = generate_random_graph(*size, 0.1);
        
        group.bench_with_input(
            BenchmarkId::new("dijkstra", size),
            &graph,
            |b, g| {
                b.iter(|| {
                    algorithms::paths::dijkstra_path(
                        g,
                        black_box(0),
                        black_box(size - 1),
                        None::<fn(&usize, &usize) -> f64>,
                    )
                });
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("bellman_ford", size),
            &graph,
            |b, g| {
                b.iter(|| {
                    algorithms::paths::bellman_ford_path(
                        g,
                        black_box(0),
                        black_box(size - 1),
                        None::<fn(&usize, &usize) -> f64>,
                    )
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark centrality algorithms
fn bench_centrality(c: &mut Criterion) {
    let mut group = c.benchmark_group("centrality");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(10); // Reduce sample size for expensive operations
    
    for size in [50, 100, 200].iter() {
        let graph = generate_scale_free_graph(*size, 3);
        
        group.bench_with_input(
            BenchmarkId::new("betweenness", size),
            &graph,
            |b, g| {
                b.iter(|| {
                    algorithms::centrality::betweenness_centrality(
                        g,
                        black_box(false),
                        black_box(false),
                    )
                });
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("closeness", size),
            &graph,
            |b, g| {
                b.iter(|| {
                    algorithms::centrality::closeness_centrality(
                        g,
                        black_box(false),
                    )
                });
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("pagerank", size),
            &graph,
            |b, g| {
                b.iter(|| {
                    algorithms::centrality::pagerank(
                        g,
                        black_box(0.85),
                        None,
                        black_box(100),
                        black_box(1e-6),
                    )
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark community detection algorithms
fn bench_community(c: &mut Criterion) {
    let mut group = c.benchmark_group("community");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(10);
    
    for size in [100, 200, 500].iter() {
        let graph = generate_scale_free_graph(*size, 5);
        
        group.bench_with_input(
            BenchmarkId::new("louvain", size),
            &graph,
            |b, g| {
                b.iter(|| {
                    algorithms::community::louvain_communities(
                        g,
                        black_box(1.0),
                        black_box(0.0001),
                        Some(100),
                    )
                });
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("label_propagation", size),
            &graph,
            |b, g| {
                b.iter(|| {
                    algorithms::community::label_propagation_communities(
                        g,
                        Some(100),
                    )
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark MST algorithms
fn bench_mst(c: &mut Criterion) {
    let mut group = c.benchmark_group("mst");
    group.measurement_time(Duration::from_secs(5));
    
    for size in [100, 500, 1000].iter() {
        let graph = generate_random_graph(*size, 0.2);
        
        group.bench_with_input(
            BenchmarkId::new("kruskal", size),
            &graph,
            |b, g| {
                b.iter(|| algorithms::mst::kruskal_mst(g));
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("prim", size),
            &graph,
            |b, g| {
                b.iter(|| algorithms::mst::prim_mst(g));
            },
        );
    }
    
    group.finish();
}

/// Benchmark coloring algorithms
fn bench_coloring(c: &mut Criterion) {
    let mut group = c.benchmark_group("coloring");
    group.measurement_time(Duration::from_secs(5));
    
    for size in [50, 100, 200].iter() {
        let graph = generate_random_graph(*size, 0.1);
        
        group.bench_with_input(
            BenchmarkId::new("greedy", size),
            &graph,
            |b, g| {
                b.iter(|| {
                    algorithms::coloring::greedy_color(
                        g,
                        algorithms::coloring::greedy::ColoringStrategy::LargestFirst,
                    )
                });
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("dsatur", size),
            &graph,
            |b, g| {
                b.iter(|| algorithms::coloring::dsatur_coloring(g));
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("welsh_powell", size),
            &graph,
            |b, g| {
                b.iter(|| algorithms::coloring::welsh_powell_coloring(g));
            },
        );
    }
    
    group.finish();
}

/// Benchmark clique algorithms
fn bench_clique(c: &mut Criterion) {
    let mut group = c.benchmark_group("clique");
    group.measurement_time(Duration::from_secs(5));
    group.sample_size(10);
    
    for size in [20, 30, 40].iter() {
        let graph = generate_random_graph(*size, 0.3);
        
        group.bench_with_input(
            BenchmarkId::new("max_clique", size),
            &graph,
            |b, g| {
                b.iter(|| algorithms::clique::max_clique(g));
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("find_cliques", size),
            &graph,
            |b, g| {
                b.iter(|| algorithms::clique::find_cliques(g));
            },
        );
    }
    
    group.finish();
}

/// Benchmark graph generation
fn bench_generators(c: &mut Criterion) {
    let mut group = c.benchmark_group("generators");
    
    group.bench_function("erdos_renyi_1000", |b| {
        b.iter(|| {
            algorithms::generators::erdos_renyi(
                black_box(1000),
                black_box(0.01),
                black_box(false),
                black_box(Some(42)),
            )
        });
    });
    
    group.bench_function("barabasi_albert_1000", |b| {
        b.iter(|| {
            algorithms::generators::barabasi_albert(
                black_box(1000),
                black_box(5),
                black_box(Some(42)),
            )
        });
    });
    
    group.bench_function("watts_strogatz_1000", |b| {
        b.iter(|| {
            algorithms::generators::watts_strogatz(
                black_box(1000),
                black_box(10),
                black_box(0.1),
                black_box(Some(42)),
            )
        });
    });
    
    group.finish();
}

/// Benchmark connectivity algorithms
fn bench_connectivity(c: &mut Criterion) {
    let mut group = c.benchmark_group("connectivity");
    
    for size in [100, 500, 1000].iter() {
        let graph = generate_random_graph(*size, 0.05);
        
        group.bench_with_input(
            BenchmarkId::new("connected_components", size),
            &graph,
            |b, g| {
                b.iter(|| algorithms::connectivity::connected_components(g));
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("is_bipartite", size),
            &graph,
            |b, g| {
                b.iter(|| algorithms::connectivity::is_bipartite(g));
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_shortest_paths,
    bench_centrality,
    bench_community,
    bench_mst,
    bench_coloring,
    bench_clique,
    bench_generators,
    bench_connectivity
);

criterion_main!(benches);