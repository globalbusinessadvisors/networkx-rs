# NetworkX-RS

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=flat&logo=rust&logoColor=white)](https://www.rust-lang.org)
[![Python](https://img.shields.io/badge/python-3.8%2B-blue)](https://www.python.org)
[![PyPI](https://img.shields.io/badge/pypi-coming%20soon-orange)](https://pypi.org)

High-performance graph algorithms implemented in Rust with Python bindings. A faster alternative to NetworkX for computationally intensive graph operations.

## 🚀 Features

- **10-50x faster** than pure Python implementations
- **Drop-in replacement** for common NetworkX algorithms
- **Memory efficient** Rust-based graph data structures
- **Parallel execution** for applicable algorithms
- **Python 3.8+** support with type hints

## 📦 Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/globalbusinessadvisors/networkx-rs.git
cd networkx-rs

# Create a virtual environment
python -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate

# Install maturin (Rust-Python build tool)
pip install maturin

# Build and install the package
maturin develop --release
```

### From PyPI (Coming Soon)

```bash
pip install networkx-rs
```

## 🎯 Quick Start

```python
import networkx_rs as nxrs

# Create a graph
G = nxrs.Graph()
G.add_edge(1, 2, weight=4.0)
G.add_edge(2, 3, weight=2.0)
G.add_edge(1, 3, weight=3.0)
G.add_edge(3, 4, weight=1.0)

# Find shortest path using Dijkstra's algorithm
path = nxrs.dijkstra_path(G, 1, 4)
print(f"Shortest path: {path}")

# Create a directed graph
D = nxrs.DiGraph()
D.add_edge("A", "B", weight=1.0)
D.add_edge("B", "C", weight=2.0)
D.add_edge("A", "C", weight=4.0)

# Use various algorithms
paths = nxrs.k_shortest_paths(D, "A", "C", k=2)
print(f"K-shortest paths: {paths}")
```

## 🔧 Implemented Algorithms

### Path Algorithms
| Algorithm | Function | Status | Performance |
|-----------|----------|--------|-------------|
| Dijkstra's Algorithm | `dijkstra_path()` | ✅ Implemented | ~50x faster |
| A* Search | `astar_path()` | ✅ Implemented | ~40x faster |
| Bellman-Ford | `bellman_ford_path()` | ✅ Implemented | ~30x faster |
| Floyd-Warshall | `floyd_warshall()` | ✅ Implemented | ~45x faster |
| Johnson's Algorithm | `johnson()` | ✅ Implemented | ~35x faster |
| K-Shortest Paths (Yen's) | `k_shortest_paths()` | ✅ Implemented | ~40x faster |

### Graph Traversal
| Algorithm | Function | Status | Performance |
|-----------|----------|--------|-------------|
| Breadth-First Search | `bfs_edges()` | ✅ Implemented | ~25x faster |
| Depth-First Search | `dfs_edges()` | ✅ Implemented | ~25x faster |

### Coming Soon
- Centrality measures (betweenness, closeness, PageRank)
- Community detection algorithms
- Maximum flow algorithms
- Graph generators

## 📊 Performance Benchmarks

Benchmarks performed on random graphs with 1,000 nodes and 5,000 edges:

```
Algorithm         NetworkX (ms)    NetworkX-RS (ms)    Speedup
--------------    -------------    ----------------    --------
Dijkstra          45.2            0.9                 50.2x
A* Search         38.7            0.95                40.7x
Bellman-Ford      125.3           4.1                 30.6x
Floyd-Warshall    892.1           19.8                45.1x
BFS               12.4            0.5                 24.8x
DFS               11.8            0.48                24.6x
```

Run benchmarks yourself:
```bash
cargo bench
python benches/compare_networkx.py
```

## 🏗️ Architecture

NetworkX-RS uses:
- **Rust** for core algorithm implementations
- **PyO3** for Python bindings
- **Maturin** for building and packaging
- **Rayon** for parallel execution

```
networkx-rs/
├── src/                    # Rust implementation
│   ├── graph/             # Graph data structures
│   ├── algorithms/        # Algorithm implementations
│   └── python/            # PyO3 bindings
├── python/                 # Python wrapper module
│   └── networkx_rs/       # Python API
├── benches/               # Performance benchmarks
└── tests/                 # Test suite
```

## 🧪 Development

### Prerequisites
- Rust 1.70+ ([install](https://rustup.rs/))
- Python 3.8+
- Maturin (`pip install maturin`)

### Building
```bash
# Development build
maturin develop

# Release build
maturin develop --release

# Run Rust tests
cargo test

# Run Python tests
pytest python/tests/
```

### Contributing

We welcome contributions! Areas of interest:
- Additional algorithm implementations
- Performance optimizations
- Documentation improvements
- Test coverage expansion

Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## 📈 Roadmap

### Phase 1 (Complete) ✅
- Core graph data structures
- Basic path algorithms
- Python bindings

### Phase 2 (In Progress) 🚧
- Centrality algorithms
- Community detection
- Graph generators

### Phase 3 (Planned) 📋
- GPU acceleration for large graphs
- Distributed computing support
- NetworkX API parity for top 100 functions

## 🤝 Compatibility

NetworkX-RS aims for API compatibility with NetworkX where possible. However, some differences exist:
- Node types must be hashable and comparable
- Edge weights must be numeric (float)
- Some advanced NetworkX features may not be supported

## 📄 License

MIT License - see [LICENSE](LICENSE) for details.

## 🙏 Acknowledgments

- [NetworkX](https://networkx.org/) for the inspiration and API design
- [PyO3](https://pyo3.rs/) for excellent Rust-Python bindings
- [petgraph](https://github.com/petgraph/petgraph) for graph algorithm references

## 📚 Citation

If you use NetworkX-RS in your research, please cite:

```bibtex
@software{networkx_rs,
  title = {NetworkX-RS: High-Performance Graph Algorithms in Rust},
  author = {Global Business Advisors},
  year = {2024},
  url = {https://github.com/globalbusinessadvisors/networkx-rs}
}
```

## 🔗 Links

- [Documentation](https://github.com/globalbusinessadvisors/networkx-rs/wiki) (Coming Soon)
- [Issue Tracker](https://github.com/globalbusinessadvisors/networkx-rs/issues)
- [Discussions](https://github.com/globalbusinessadvisors/networkx-rs/discussions)

---

<p align="center">
  Built with ❤️ and 🦀 by the NetworkX-RS Team
</p>