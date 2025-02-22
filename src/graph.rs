use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    marker::PhantomData,
    ops::Add,
    rc::Rc,
    sync::{Arc, RwLock},
    usize,
};

use crate::{
    edge::Edge,
    path::{Path, SharedPath},
};

pub struct Graph<K, V, H>
where
    K: Clone + Hash + Eq + Ord,
    V: IntoConnections<K, H> + Eq + Hash + Clone + Ord,
    H: Eq + Ord + Add<Output = H> + Hash + Default + Clone,
    for<'a> H: Add<&'a H, Output = H> + PartialOrd<&'a H>,
{
    // Use a HashMap to associate each key K with its node (wrapped in Arc<RwLock<>>)
    pub map: HashMap<K, Arc<RwLock<V>>>,
    pub h: PhantomData<H>,
}

impl<K, V, H> Graph<K, V, H>
where
    K: Clone + Hash + Eq + Ord,
    V: IntoConnections<K, H> + Eq + Hash + Clone + Ord,
    H: Eq + Ord + Add<Output = H> + Hash + Default + Clone,
    for<'a> H: Add<&'a H, Output = H> + PartialOrd<&'a H>,
{
    pub fn new(map: HashMap<K, Arc<RwLock<V>>>) -> Self {
        Graph {
            map,
            h: PhantomData,
        }
    }

    pub fn map(&self, from: &K) -> HashMap<K, Vec<SharedPath<K, H>>> {
        let mut open_paths = Vec::new();
        let mut allpaths = HashMap::<K, Vec<SharedPath<K, H>>>::new();
        if let Some(node) = self.map.get(from) {
            if let Ok(guard) = node.read() {
                let mut i = 0;
                let connections = guard.into_connections();
                for c in &connections {
                    let edge = Rc::new(c.into_edge(i.clone()));
                    let h = edge.h.clone();
                    let path = Path::<K, H> {
                        total: h,
                        nodes: HashSet::from([edge.a.clone()]),
                        path: vec![edge],
                    };
                    open_paths.push(SharedPath::new(path));
                    i += 1;
                }
            }
        }

        while let Some(old_path) = open_paths.pop() {
            if let Some(last_edge) = old_path.inner.path.last() {
                let last_node = last_edge.b.clone();
                let outgoing_edges = self.generate_edges(&last_node);
                if outgoing_edges.is_empty() {
                    // If there are no outgoing edges, consider this path complete.
                    allpaths
                        .entry(last_node.clone())
                        .or_default()
                        .push(old_path.clone());
                } else {
                    // Extend the path using propagate_path.
                    let new_paths = self.propagate_path(old_path);
                    for current_path in new_paths {
                        // Get the new last node.
                        if let Some(new_edge) = current_path.inner.path.last() {
                            // Check for cycles: if the new node is already in the path, skip it.
                            let arr = allpaths.entry(new_edge.b.clone()).or_insert(Vec::new());
                            arr.push(current_path.clone());

                            if !current_path.inner.nodes.contains(&new_edge.b) {
                                open_paths.push(current_path);
                            } else {
                            }
                        }
                    }
                }
            } else {
                // If there's no last edge (path is empty), you might decide to handle it specially.
            }
        }
        allpaths
    }

    /// Propagates a given SharedPath (whose edges reference nodes by key K) by extending it with all
    /// possible outgoing edges from the last node in the path.
    /// Possible outgoing edges from the last node in the path.
    pub fn propagate_path(&self, from: SharedPath<K, H>) -> Vec<SharedPath<K, H>> {
        let mut new_paths = Vec::new();
        // Get the last edge in the current path.
        let last = from.inner.path.last().unwrap().clone();
        // For each new edge from the current node, extend the path.
        for edge in self.generate_edges(&last.b) {
            let mut new_edges = from.inner.path.clone();
            let mut new_nodes = from.inner.nodes.clone();
            // Insert the destination node (edge.b) as the newly reached node.
            new_nodes.insert(edge.b.clone());

            let new_total = from.inner.total.clone() + edge.h.clone();
            new_edges.push(edge.clone());
            let new_path = Path {
                total: new_total,
                path: new_edges,
                nodes: new_nodes,
            };
            new_paths.push(SharedPath {
                inner: Rc::new(new_path),
            });
        }
        new_paths
    }

    /// Retrieve edges from the graph for a given key.
    pub fn generate_edges(&self, key: &K) -> Vec<Rc<Edge<K, H>>> {
        let mut arr = Vec::new();
        if let Some(node_arc) = self.map.get(key) {
            let node = node_arc.read().unwrap();
            let connections = node.into_connections();
            let mut i = 0;
            for conn in connections {
                let edge = conn.into_edge(i.clone());
                arr.push(Rc::new(edge));
                i += 1;
            }
        }
        arr
    }
}

/// Trait for converting a node into its outgoing connections.
/// Note: The node type V now produces connections whose edge endpoints are of type K.
pub trait IntoConnections<K, H>
where
    K: Eq + Ord + Hash + Clone,
    H: Ord + Hash + Default + Clone,
{
    type Item: Connection<K, H>;
    fn into_connections(&self) -> Vec<Self::Item>;
}

/// Trait defining a connection that can be converted into an edge.
/// The edge will refer to nodes by their key of type K.
pub trait Connection<K: Eq + Ord + Hash + Clone, H: Ord + Clone + Hash> {
    fn into_edge(&self, id: usize) -> Edge<K, H>;
}

/// Trait for types that have bounded minimum and maximum values.
pub trait Bounded {
    fn min_value() -> Self;
    fn max_value() -> Self;
}

impl Bounded for i32 {
    fn min_value() -> Self {
        i32::MIN
    }
    fn max_value() -> Self {
        i32::MAX
    }
}
