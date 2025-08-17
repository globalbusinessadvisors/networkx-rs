//! VF2 algorithm for graph isomorphism

use super::Isomorphism;
use crate::graph::traits::GraphBase;
use crate::errors::Result;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

/// VF2 algorithm state
pub struct VF2State<N1, N2> {
    /// Mapping from G1 to G2
    core_1: HashMap<N1, N2>,
    /// Mapping from G2 to G1
    core_2: HashMap<N2, N1>,
    /// Nodes in G1 that are neighbors of mapped nodes
    in_1: HashSet<N1>,
    /// Nodes in G2 that are neighbors of mapped nodes
    in_2: HashSet<N2>,
    /// Nodes in G1 that have neighbors in mapped nodes
    out_1: HashSet<N1>,
    /// Nodes in G2 that have neighbors in mapped nodes
    out_2: HashSet<N2>,
}

impl<N1, N2> VF2State<N1, N2>
where
    N1: Clone + Hash + Eq,
    N2: Clone + Hash + Eq,
{
    /// Create a new VF2 state
    pub fn new() -> Self {
        VF2State {
            core_1: HashMap::new(),
            core_2: HashMap::new(),
            in_1: HashSet::new(),
            in_2: HashSet::new(),
            out_1: HashSet::new(),
            out_2: HashSet::new(),
        }
    }
    
    /// Check if a node from G1 is mapped
    pub fn is_mapped_1(&self, node: &N1) -> bool {
        self.core_1.contains_key(node)
    }
    
    /// Check if a node from G2 is mapped
    pub fn is_mapped_2(&self, node: &N2) -> bool {
        self.core_2.contains_key(node)
    }
    
    /// Add a mapping
    pub fn add_mapping(&mut self, n1: N1, n2: N2) {
        self.core_1.insert(n1.clone(), n2.clone());
        self.core_2.insert(n2, n1);
    }
    
    /// Remove a mapping
    pub fn remove_mapping(&mut self, n1: &N1, n2: &N2) {
        self.core_1.remove(n1);
        self.core_2.remove(n2);
    }
}

/// Check if two graphs are isomorphic using VF2
pub fn is_isomorphic<G1, G2, N1, N2>(g1: &G1, g2: &G2) -> bool
where
    G1: GraphBase<NodeId = N1>,
    G2: GraphBase<NodeId = N2>,
    N1: Clone + Hash + Eq,
    N2: Clone + Hash + Eq,
{
    // Quick checks
    if g1.node_count() != g2.node_count() || g1.edge_count() != g2.edge_count() {
        return false;
    }
    
    find_isomorphism(g1, g2).is_some()
}

/// Find an isomorphism between two graphs
pub fn find_isomorphism<G1, G2, N1, N2>(
    g1: &G1,
    g2: &G2,
) -> Option<HashMap<N1, N2>>
where
    G1: GraphBase<NodeId = N1>,
    G2: GraphBase<NodeId = N2>,
    N1: Clone + Hash + Eq,
    N2: Clone + Hash + Eq,
{
    let nodes1: Vec<N1> = g1.nodes().collect();
    let nodes2: Vec<N2> = g2.nodes().collect();
    
    if nodes1.len() != nodes2.len() {
        return None;
    }
    
    let mut state = VF2State::new();
    let mut mapping = HashMap::new();
    
    if vf2_recursive(g1, g2, &nodes1, &nodes2, &mut state, &mut mapping) {
        Some(mapping)
    } else {
        None
    }
}

/// Recursive VF2 algorithm
fn vf2_recursive<G1, G2, N1, N2>(
    g1: &G1,
    g2: &G2,
    nodes1: &[N1],
    nodes2: &[N2],
    state: &mut VF2State<N1, N2>,
    mapping: &mut HashMap<N1, N2>,
) -> bool
where
    G1: GraphBase<NodeId = N1>,
    G2: GraphBase<NodeId = N2>,
    N1: Clone + Hash + Eq,
    N2: Clone + Hash + Eq,
{
    // Check if we have a complete mapping
    if mapping.len() == nodes1.len() {
        return is_valid_isomorphism(g1, g2, mapping);
    }
    
    // Get candidate pairs
    let candidates = get_candidate_pairs(g1, g2, nodes1, nodes2, state);
    
    // Try each candidate pair
    for (n1, n2) in candidates {
        if is_feasible(g1, g2, &n1, &n2, state) {
            // Add the mapping
            state.add_mapping(n1.clone(), n2.clone());
            mapping.insert(n1.clone(), n2.clone());
            
            // Update in/out sets
            update_sets(g1, g2, &n1, &n2, state);
            
            // Recursive call
            if vf2_recursive(g1, g2, nodes1, nodes2, state, mapping) {
                return true;
            }
            
            // Backtrack
            state.remove_mapping(&n1, &n2);
            mapping.remove(&n1);
        }
    }
    
    false
}

/// Get candidate pairs for mapping
fn get_candidate_pairs<G1, G2, N1, N2>(
    _g1: &G1,
    _g2: &G2,
    nodes1: &[N1],
    nodes2: &[N2],
    state: &VF2State<N1, N2>,
) -> Vec<(N1, N2)>
where
    G1: GraphBase<NodeId = N1>,
    G2: GraphBase<NodeId = N2>,
    N1: Clone + Hash + Eq,
    N2: Clone + Hash + Eq,
{
    let mut pairs = Vec::new();
    
    // Find unmapped nodes
    for n1 in nodes1 {
        if !state.is_mapped_1(n1) {
            for n2 in nodes2 {
                if !state.is_mapped_2(n2) {
                    pairs.push((n1.clone(), n2.clone()));
                }
            }
            break; // Only need one node from G1
        }
    }
    
    pairs
}

/// Check if a pair is feasible
fn is_feasible<G1, G2, N1, N2>(
    g1: &G1,
    g2: &G2,
    n1: &N1,
    n2: &N2,
    state: &VF2State<N1, N2>,
) -> bool
where
    G1: GraphBase<NodeId = N1>,
    G2: GraphBase<NodeId = N2>,
    N1: Clone + Hash + Eq,
    N2: Clone + Hash + Eq,
{
    // Check degree compatibility
    if g1.degree(n1) != g2.degree(n2) {
        return false;
    }
    
    // Check consistency with existing mapping
    for neighbor1 in g1.neighbors(n1) {
        if let Some(mapped) = state.core_1.get(&neighbor1) {
            if !g2.has_edge(n2, mapped) {
                return false;
            }
        }
    }
    
    true
}

/// Update in/out sets after adding a mapping
fn update_sets<G1, G2, N1, N2>(
    g1: &G1,
    g2: &G2,
    n1: &N1,
    n2: &N2,
    state: &mut VF2State<N1, N2>,
) where
    G1: GraphBase<NodeId = N1>,
    G2: GraphBase<NodeId = N2>,
    N1: Clone + Hash + Eq,
    N2: Clone + Hash + Eq,
{
    // Update in/out sets based on neighbors
    for neighbor in g1.neighbors(n1) {
        if !state.is_mapped_1(&neighbor) {
            state.out_1.insert(neighbor.clone());
        }
    }
    
    for neighbor in g2.neighbors(n2) {
        if !state.is_mapped_2(&neighbor) {
            state.out_2.insert(neighbor.clone());
        }
    }
}

/// Check if a mapping is a valid isomorphism
fn is_valid_isomorphism<G1, G2, N1, N2>(
    g1: &G1,
    g2: &G2,
    mapping: &HashMap<N1, N2>,
) -> bool
where
    G1: GraphBase<NodeId = N1>,
    G2: GraphBase<NodeId = N2>,
    N1: Clone + Hash + Eq,
    N2: Clone + Hash + Eq,
{
    // Check that all edges are preserved
    for (u, v, _) in g1.edges() {
        if let (Some(u2), Some(v2)) = (mapping.get(&u), mapping.get(&v)) {
            if !g2.has_edge(u2, v2) {
                return false;
            }
        } else {
            return false;
        }
    }
    
    true
}

/// Find all isomorphisms between two graphs
pub fn find_all_isomorphisms<G1, G2, N1, N2>(
    g1: &G1,
    g2: &G2,
) -> Vec<HashMap<N1, N2>>
where
    G1: GraphBase<NodeId = N1>,
    G2: GraphBase<NodeId = N2>,
    N1: Clone + Hash + Eq,
    N2: Clone + Hash + Eq,
{
    let nodes1: Vec<N1> = g1.nodes().collect();
    let nodes2: Vec<N2> = g2.nodes().collect();
    
    if nodes1.len() != nodes2.len() {
        return Vec::new();
    }
    
    let mut all_mappings = Vec::new();
    let mut state = VF2State::new();
    let mut mapping = HashMap::new();
    
    find_all_recursive(g1, g2, &nodes1, &nodes2, &mut state, &mut mapping, &mut all_mappings);
    
    all_mappings
}

/// Recursive function to find all isomorphisms
fn find_all_recursive<G1, G2, N1, N2>(
    g1: &G1,
    g2: &G2,
    nodes1: &[N1],
    nodes2: &[N2],
    state: &mut VF2State<N1, N2>,
    mapping: &mut HashMap<N1, N2>,
    all_mappings: &mut Vec<HashMap<N1, N2>>,
) where
    G1: GraphBase<NodeId = N1>,
    G2: GraphBase<NodeId = N2>,
    N1: Clone + Hash + Eq,
    N2: Clone + Hash + Eq,
{
    if mapping.len() == nodes1.len() {
        if is_valid_isomorphism(g1, g2, mapping) {
            all_mappings.push(mapping.clone());
        }
        return;
    }
    
    let candidates = get_candidate_pairs(g1, g2, nodes1, nodes2, state);
    
    for (n1, n2) in candidates {
        if is_feasible(g1, g2, &n1, &n2, state) {
            state.add_mapping(n1.clone(), n2.clone());
            mapping.insert(n1.clone(), n2.clone());
            update_sets(g1, g2, &n1, &n2, state);
            
            find_all_recursive(g1, g2, nodes1, nodes2, state, mapping, all_mappings);
            
            state.remove_mapping(&n1, &n2);
            mapping.remove(&n1);
        }
    }
}