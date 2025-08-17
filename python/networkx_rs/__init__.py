"""NetworkX-RS: Rust-accelerated NetworkX

Drop-in replacement for NetworkX with Rust performance.
"""

from networkx_rs.networkx_rs import (
    __version__,
    __rust_version__,
)

from networkx_rs.classes import Graph, DiGraph
from networkx_rs.algorithms import *

__all__ = [
    "__version__",
    "__rust_version__",
    "Graph",
    "DiGraph",
    # Paths
    "shortest_path",
    "dijkstra_path",
    "dijkstra_path_length",
    "single_source_dijkstra",
    "all_pairs_dijkstra",
    "astar_path",
    "bellman_ford_path",
    # Traversal
    "bfs_edges",
    "bfs_tree",
    "dfs_edges",
    "dfs_tree",
    # Centrality
    "betweenness_centrality",
    "closeness_centrality",
    "degree_centrality",
    "eigenvector_centrality",
    "pagerank",
    # Connectivity
    "connected_components",
    "strongly_connected_components",
    "is_connected",
    "is_strongly_connected",
]