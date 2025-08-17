"""Graph classes with NetworkX-compatible API"""

from typing import Optional, Any, Dict, List, Tuple, Iterator
import networkx_rs.networkx_rs as _lib


class Graph:
    """Undirected graph compatible with NetworkX API"""
    
    def __init__(self, incoming_graph_data=None, **attr):
        self._graph = _lib.PyGraph()
        self.graph = attr.copy()
        
        if incoming_graph_data is not None:
            self._init_from_data(incoming_graph_data)
    
    def _init_from_data(self, data):
        """Initialize graph from various data formats"""
        if hasattr(data, 'edges'):
            # NetworkX graph
            for u, v, d in data.edges(data=True):
                self.add_edge(u, v, **d)
        elif isinstance(data, dict):
            # Dict of neighbors
            for node, neighbors in data.items():
                for neighbor in neighbors:
                    self.add_edge(node, neighbor)
        elif hasattr(data, '__iter__'):
            # Edge list
            for edge in data:
                if len(edge) == 2:
                    self.add_edge(edge[0], edge[1])
                elif len(edge) == 3:
                    self.add_edge(edge[0], edge[1], weight=edge[2])
    
    def add_node(self, node_for_adding, **attr):
        """Add a single node with optional attributes"""
        return self._graph.add_node(node_for_adding)
    
    def add_nodes_from(self, nodes_for_adding, **attr):
        """Add multiple nodes"""
        for node in nodes_for_adding:
            if isinstance(node, tuple) and len(node) == 2:
                node_id, node_attr = node
                self.add_node(node_id, **node_attr)
            else:
                self.add_node(node, **attr)
    
    def add_edge(self, u, v, **attr):
        """Add an edge between u and v"""
        weight = attr.pop('weight', None)
        return self._graph.add_edge(u, v, weight)
    
    def add_edges_from(self, edges_for_adding, **attr):
        """Add multiple edges"""
        for edge in edges_for_adding:
            if len(edge) == 2:
                self.add_edge(edge[0], edge[1], **attr)
            elif len(edge) == 3:
                edge_attr = attr.copy()
                edge_attr.update(edge[2] if isinstance(edge[2], dict) else {'weight': edge[2]})
                self.add_edge(edge[0], edge[1], **edge_attr)
    
    def remove_node(self, node):
        """Remove node from the graph"""
        return self._graph.remove_node(node)
    
    def remove_edge(self, u, v):
        """Remove edge between u and v"""
        return self._graph.remove_edge(u, v)
    
    def has_node(self, node) -> bool:
        """Check if node exists in graph"""
        return self._graph.has_node(node)
    
    def has_edge(self, u, v) -> bool:
        """Check if edge exists between u and v"""
        return self._graph.has_edge(u, v)
    
    def number_of_nodes(self) -> int:
        """Return the number of nodes"""
        return self._graph.node_count()
    
    def number_of_edges(self) -> int:
        """Return the number of edges"""
        return self._graph.edge_count()
    
    def nodes(self, data=False):
        """Return a view of nodes"""
        # For now, just return the nodes without data
        return self._graph.nodes()
    
    def edges(self, data=False, default=None):
        """Return a view of edges"""
        # For now, just return the edges without data
        return self._graph.edges()
    
    def neighbors(self, node):
        """Return neighbors of node"""
        return self._graph.neighbors(node)
    
    def degree(self, node=None):
        """Return degree of node(s)"""
        if node is None:
            return self._graph.degree_all()
        return self._graph.degree(node)
    
    def size(self, weight=None) -> float:
        """Return the size of the graph"""
        if weight is None:
            return self.number_of_edges()
        return self._graph.weighted_size(weight)
    
    def is_directed(self) -> bool:
        """Return True if graph is directed"""
        return False
    
    def is_multigraph(self) -> bool:
        """Return True if graph is a multigraph"""
        return False
    
    def clear(self):
        """Remove all nodes and edges"""
        self._graph.clear()
        self.graph.clear()
    
    def copy(self):
        """Return a copy of the graph"""
        return self._graph.copy_wrapped()
    
    def to_directed(self):
        """Return a directed representation"""
        from networkx_rs.classes import DiGraph
        G = DiGraph()
        G.add_edges_from(self.edges(data=True))
        return G
    
    def __len__(self) -> int:
        """Return the number of nodes"""
        return self.number_of_nodes()
    
    def __contains__(self, node) -> bool:
        """Check if node is in graph"""
        return self.has_node(node)
    
    def __iter__(self):
        """Iterate over nodes"""
        return iter(self.nodes())
    
    def __repr__(self) -> str:
        return f"Graph with {self.number_of_nodes()} nodes and {self.number_of_edges()} edges"


class DiGraph(Graph):
    """Directed graph compatible with NetworkX API"""
    
    def __init__(self, incoming_graph_data=None, **attr):
        self._graph = _lib.PyDiGraph()
        self.graph = attr.copy()
        
        if incoming_graph_data is not None:
            self._init_from_data(incoming_graph_data)
    
    def is_directed(self) -> bool:
        """Return True if graph is directed"""
        return True
    
    def to_undirected(self):
        """Return an undirected representation"""
        G = Graph()
        for u, v, data in self.edges(data=True):
            G.add_edge(u, v, **data)
        return G
    
    def in_degree(self, node=None):
        """Return in-degree of node(s)"""
        if node is None:
            return self._graph.in_degree_all()
        return self._graph.in_degree(node)
    
    def out_degree(self, node=None):
        """Return out-degree of node(s)"""
        if node is None:
            return self._graph.out_degree_all()
        return self._graph.out_degree(node)
    
    def predecessors(self, node):
        """Return predecessors of node"""
        return self._graph.predecessors(node)
    
    def successors(self, node):
        """Return successors of node"""
        return self._graph.successors(node)
    
    def reverse(self, copy=True):
        """Return the reverse of the graph"""
        if copy:
            return self._graph.reverse_copy()
        self._graph.reverse_inplace()
        return self
    
    def __repr__(self) -> str:
        return f"DiGraph with {self.number_of_nodes()} nodes and {self.number_of_edges()} edges"