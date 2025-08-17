"""Algorithm wrappers for NetworkX-RS"""

from typing import Optional, Dict, List, Tuple, Any
import networkx_rs.networkx_rs as _lib


def shortest_path(G, source=None, target=None, weight=None, method='dijkstra'):
    """Compute shortest paths in the graph."""
    if method == 'dijkstra':
        if hasattr(G, '_graph'):
            if G.is_directed():
                return _lib.dijkstra_path_digraph(G._graph, source, target)
            else:
                return _lib.dijkstra_path(G._graph, source, target)
    raise NotImplementedError(f"Method {method} not implemented")


def dijkstra_path(G, source, target, weight=None):
    """Find shortest path using Dijkstra's algorithm."""
    return shortest_path(G, source, target, weight, method='dijkstra')


def dijkstra_path_length(G, source, target, weight=None):
    """Find shortest path length using Dijkstra's algorithm."""
    result = dijkstra_path(G, source, target, weight)
    if result:
        return result[1]  # Return just the length
    return None


def single_source_dijkstra(G, source, weight=None, cutoff=None):
    """Compute shortest paths and lengths from source."""
    # TODO: Implement full single source
    return dijkstra_path(G, source, None, weight)


def all_pairs_dijkstra(G, weight=None, cutoff=None):
    """Find shortest paths between all pairs of nodes."""
    # TODO: Implement all pairs
    pass


def astar_path(G, source, target, heuristic=None, weight=None):
    """Find shortest path using A* algorithm."""
    if heuristic is None:
        heuristic = lambda n: 0
    if hasattr(G, '_graph'):
        return _lib.astar_path(G._graph, source, target, heuristic)
    raise TypeError("Graph must be a NetworkX-RS graph")


def bellman_ford_path(G, source, target, weight=None):
    """Find shortest path using Bellman-Ford algorithm."""
    if hasattr(G, '_graph') and G.is_directed():
        distances, predecessors = _lib.bellman_ford(G._graph, source)
        if target in distances:
            # Reconstruct path
            path = [target]
            current = target
            while current != source and current in predecessors:
                current = predecessors[current]
                path.append(current)
            path.reverse()
            return path, distances[target]
    return None


def floyd_warshall(G, weight=None):
    """Find all-pairs shortest paths using Floyd-Warshall."""
    if hasattr(G, '_graph') and G.is_directed():
        return _lib.floyd_warshall(G._graph)
    raise TypeError("Graph must be a directed NetworkX-RS graph")


def k_shortest_paths(G, source, target, k, weight=None):
    """Find k shortest paths between source and target."""
    if hasattr(G, '_graph'):
        return _lib.k_shortest_paths(G._graph, source, target, k)
    raise TypeError("Graph must be a NetworkX-RS graph")


# Traversal algorithms (stubs for now)
def bfs_edges(G, source):
    """Breadth-first search edges."""
    pass


def bfs_tree(G, source):
    """Breadth-first search tree."""
    pass


def dfs_edges(G, source):
    """Depth-first search edges."""
    pass


def dfs_tree(G, source):
    """Depth-first search tree."""
    pass


# Centrality algorithms (stubs for now)
def betweenness_centrality(G):
    """Compute betweenness centrality."""
    pass


def closeness_centrality(G):
    """Compute closeness centrality."""
    pass


def degree_centrality(G):
    """Compute degree centrality."""
    pass


def eigenvector_centrality(G):
    """Compute eigenvector centrality."""
    pass


def pagerank(G, alpha=0.85):
    """Compute PageRank."""
    pass


# Connectivity algorithms (stubs for now)
def connected_components(G):
    """Find connected components."""
    pass


def strongly_connected_components(G):
    """Find strongly connected components."""
    pass


def is_connected(G):
    """Check if graph is connected."""
    pass


def is_strongly_connected(G):
    """Check if directed graph is strongly connected."""
    pass