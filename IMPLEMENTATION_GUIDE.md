# NetworkX-RS Implementation Guide

## 🏗️ Project Architecture

NetworkX-RS uses a layered architecture to provide high-performance graph algorithms while maintaining NetworkX API compatibility:

```
┌─────────────────────────────────┐
│     Python API Layer            │  <- NetworkX-compatible interface
├─────────────────────────────────┤
│     PyO3 Bindings Layer         │  <- Rust-Python bridge
├─────────────────────────────────┤
│     Rust Algorithm Layer        │  <- High-performance implementations
├─────────────────────────────────┤
│     Rust Graph Core            │  <- Optimized data structures
└─────────────────────────────────┘
```

## 📁 Directory Structure

```
networkx-rs/
├── src/                          # Rust source code
│   ├── graph/                   # Graph data structures
│   │   ├── mod.rs              # Module exports
│   │   ├── traits.rs           # Graph trait definitions
│   │   ├── undirected.rs       # Undirected graph
│   │   └── directed.rs         # Directed graph
│   ├── algorithms/              # Algorithm implementations
│   │   ├── paths/              # Shortest path algorithms
│   │   │   ├── dijkstra.rs    # Dijkstra's algorithm
│   │   │   ├── astar.rs       # A* search
│   │   │   └── bellman_ford.rs # Bellman-Ford
│   │   ├── centrality/         # Centrality measures
│   │   ├── connectivity/       # Graph connectivity
│   │   └── traversal/          # BFS, DFS, etc.
│   ├── python/                  # Python binding code
│   ├── utils.rs                # Utility functions
│   ├── errors.rs               # Error types
│   └── lib.rs                  # Library entry point
├── python/                      # Python package
│   └── networkx_rs/            # Python module
│       ├── __init__.py         # Package exports
│       ├── classes.py          # Graph classes
│       └── algorithms.py       # Algorithm wrappers
├── benches/                     # Performance benchmarks
├── tests/                       # Integration tests
└── examples/                    # Usage examples
```

## 🚀 Implementation Phases

### Phase 1: Foundation (Completed ✅)
- [x] Core graph data structures
- [x] Basic graph operations
- [x] PyO3 setup
- [x] BFS/DFS traversal
- [x] Dijkstra's algorithm

### Phase 2: Path Algorithms (In Progress 🔄)
- [ ] A* search with heuristics
- [ ] Bellman-Ford for negative weights
- [ ] Floyd-Warshall all-pairs
- [ ] Johnson's algorithm
- [ ] K-shortest paths

### Phase 3: Centrality Measures
- [ ] Betweenness centrality (parallel)
- [ ] Closeness centrality
- [ ] Degree centrality
- [ ] Eigenvector centrality
- [ ] PageRank
- [ ] HITS algorithm

### Phase 4: Connectivity
- [ ] Connected components (union-find)
- [ ] Strongly connected components (Tarjan)
- [ ] Bridges and articulation points
- [ ] Minimum spanning tree (Kruskal/Prim)
- [ ] Maximum flow (push-relabel)
- [ ] Minimum cut

## 🔧 Development Setup

### Prerequisites
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Python development dependencies
pip install maturin pytest pytest-benchmark networkx hypothesis
```

### Building the Project
```bash
# Build Rust library
cargo build --release

# Build Python extension
maturin develop --release

# Run tests
cargo test
pytest python/tests/
```

### Running Benchmarks
```bash
# Rust benchmarks
cargo bench

# Python comparison benchmarks
python benches/compare_networkx.py
```

## 📊 Performance Optimization Strategies

### 1. Data Structure Optimizations
- **Adjacency List**: Using `Vec<AHashMap>` for O(1) neighbor access
- **Hash Functions**: AHash for faster hashing
- **Memory Layout**: Cache-friendly data structures

### 2. Algorithm Optimizations
- **Priority Queues**: Binary heap for Dijkstra
- **Parallel Processing**: Rayon for embarrassingly parallel operations
- **SIMD**: Vectorized operations where applicable
- **Bit Manipulation**: For connected components

### 3. Python Binding Optimizations
- **Zero-Copy**: Minimize data copying between Python and Rust
- **Lazy Evaluation**: Defer computation until needed
- **Batch Operations**: Process multiple items in single FFI call

## 🧪 Testing Strategy

### Unit Tests
```rust
#[test]
fn test_dijkstra_simple() {
    let graph = create_test_graph();
    let path = dijkstra_path(&graph, 0, 4);
    assert_eq!(path, Some(vec![0, 1, 3, 4]));
}
```

### Property-Based Testing
```python
from hypothesis import given, strategies as st

@given(st.integers(10, 100))
def test_graph_invariants(n_nodes):
    G = nx.Graph()
    # Test that our implementation maintains invariants
```

### Differential Testing
```python
def test_dijkstra_compatibility():
    # Create identical graphs in NetworkX and NetworkX-RS
    nx_graph = nx.Graph(edges)
    rs_graph = nxrs.Graph(edges)
    
    # Compare results
    nx_path = nx.shortest_path(nx_graph, 0, 10)
    rs_path = nxrs.shortest_path(rs_graph, 0, 10)
    assert nx_path == rs_path
```

## 🎯 API Compatibility Guidelines

### Function Signatures
Match NetworkX exactly:
```python
# NetworkX
def shortest_path(G, source=None, target=None, weight=None, method='dijkstra'):
    ...

# NetworkX-RS (must be identical)
def shortest_path(G, source=None, target=None, weight=None, method='dijkstra'):
    ...
```

### Return Types
Maintain consistency:
- Lists for paths: `[0, 1, 2, 3]`
- Dicts for distances: `{0: 0, 1: 1.5, 2: 3.0}`
- Generators where NetworkX uses them

### Error Handling
```python
# Match NetworkX exceptions
class NetworkXError(Exception):
    pass

class NodeNotFound(NetworkXError):
    pass
```

## 🚄 Adding New Algorithms

### 1. Implement in Rust
```rust
// src/algorithms/new_algorithm.rs
pub fn new_algorithm<G: GraphBase>(graph: &G) -> Result<Output> {
    // Implementation
}
```

### 2. Add Python Bindings
```rust
// src/python/algorithms.rs
#[pyfunction]
fn py_new_algorithm(graph: &PyGraph) -> PyResult<Output> {
    Ok(new_algorithm(&graph.inner)?)
}
```

### 3. Expose in Python
```python
# python/networkx_rs/algorithms.py
def new_algorithm(G, **kwargs):
    """NetworkX-compatible wrapper"""
    return _lib.new_algorithm(G._graph, **kwargs)
```

### 4. Add Tests
```python
# python/tests/test_new_algorithm.py
def test_new_algorithm():
    G = create_test_graph()
    result = nxrs.new_algorithm(G)
    expected = nx.new_algorithm(G)
    assert result == expected
```

## 📈 Benchmarking Guidelines

### Micro-benchmarks
```rust
#[bench]
fn bench_dijkstra_1000_nodes(b: &mut Bencher) {
    let graph = create_graph(1000);
    b.iter(|| dijkstra(&graph, 0, 999));
}
```

### Comparison Benchmarks
```python
import networkx as nx
import networkx_rs as nxrs
import time

def benchmark_algorithm(graph_size):
    # Create graphs
    nx_g = nx.random_geometric_graph(graph_size, 0.125)
    rs_g = nxrs.Graph(nx_g.edges())
    
    # NetworkX timing
    start = time.perf_counter()
    nx_result = nx.algorithm(nx_g)
    nx_time = time.perf_counter() - start
    
    # NetworkX-RS timing
    start = time.perf_counter()
    rs_result = nxrs.algorithm(rs_g)
    rs_time = time.perf_counter() - start
    
    speedup = nx_time / rs_time
    print(f"Speedup: {speedup:.2f}x")
```

## 🐛 Debugging Tips

### Rust Debugging
```bash
# Enable debug symbols
RUSTFLAGS="-g" cargo build

# Use LLDB/GDB
rust-lldb target/debug/networkx-rs
```

### Python-Rust Boundary
```python
# Enable verbose PyO3 errors
import os
os.environ['RUST_BACKTRACE'] = '1'
```

### Performance Profiling
```bash
# CPU profiling
cargo install flamegraph
cargo flamegraph --bench dijkstra

# Memory profiling
valgrind --tool=massif target/release/bench
```

## 🤝 Contributing

### Code Style
- Rust: `cargo fmt` and `cargo clippy`
- Python: `black` and `ruff`

### Pull Request Process
1. Create feature branch
2. Implement with tests
3. Benchmark performance
4. Update documentation
5. Submit PR with results

### Performance Requirements
New algorithms must:
- Be at least 5x faster than pure Python
- Maintain 100% API compatibility
- Include comprehensive tests
- Have benchmark comparisons

## 📚 Resources

- [NetworkX Documentation](https://networkx.org/documentation/stable/)
- [PyO3 Book](https://pyo3.rs/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Graph Algorithm Implementations](https://github.com/petgraph/petgraph)

## ❓ FAQ

**Q: Why not use petgraph directly?**
A: Custom implementation allows better control over memory layout and NetworkX-specific optimizations.

**Q: How to handle NetworkX's node labels?**
A: Internal indices with bidirectional mapping to preserve performance.

**Q: What about graph attributes?**
A: Stored separately to avoid impacting algorithm performance.

**Q: Memory safety with Python?**
A: PyO3 handles reference counting; Rust ensures memory safety.