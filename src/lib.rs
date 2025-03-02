pub mod pathfinder {
    use crate::edge::{self, Edge};
    use crate::graph::{Graph, IntoConnections};
    use std::collections::{HashMap, HashSet};
    use std::fmt::{Debug, Display};
    use std::hash::Hash;
    use std::ops::{Add, Sub};
    use std::rc::Rc;

    pub trait Pathfind<K, V, H>
    where
        K: Clone + Hash + Eq + Ord + Display + Debug,
        V: IntoConnections<K, H> + Eq + Hash + Ord,
        H: Eq + Ord + Add<Output = H> + Hash + Default + Clone + Display + Debug,
    {
        fn bellman_ford(
            &mut self,
            from: &K,
        ) -> Result<HashMap<K, (H, Vec<Rc<Edge<K, H>>>)>, Vec<Rc<Edge<K, H>>>>;
    }

    impl<K, V, H> Pathfind<K, V, H> for Graph<K, V, H>
    where
        K: Clone + Hash + Eq + Ord + Display + Debug,
        V: IntoConnections<K, H> + Eq + Hash + Ord,
        H: Eq + Ord + Add<Output = H> + Hash + Default + Clone + Display + Debug,
    {
        fn bellman_ford(
            &mut self,
            from: &K,
        ) -> Result<HashMap<K, (H, Vec<Rc<Edge<K, H>>>)>, Vec<Rc<Edge<K, H>>>> {
            let map_keys = self.map.keys().cloned().collect::<Vec<K>>();
            let n = map_keys.len();

            // Initialize paths: for each vertex, store (cost, predecessor_edge).
            // For the source, cost is H::default() (assumed zero) and no predecessor.
            // We assume that generate_edges(from) gives the immediate neighbors.
            let mut paths: HashMap<K, (H, Rc<Edge<K, H>>)> = HashMap::new();
            // For each edge from the source, initialize its path.
            for edge in self.generate_edges(from) {
                paths.insert(edge.b.clone(), (edge.h.clone(), edge.clone()));
            }

            // Relax edges for (n - 1) iterations.
            for _ in 0..(n - 1) {
                // To check if any update happens in this iteration.
                let mut updated = false;
                let current_paths = paths.clone();
                for (_vertex, (cost, pred_edge)) in current_paths.iter() {
                    let new_edges = self.generate_edges(&pred_edge.b);
                    for new_edge in new_edges {
                        let new_cost = cost.clone() + new_edge.h.clone();
                        if let Some((existing_cost, _)) = paths.get(&new_edge.b) {
                            if new_cost < *existing_cost {
                                paths.insert(new_edge.b.clone(), (new_cost, new_edge.clone()));
                                updated = true;
                            }
                        } else {
                            paths.insert(new_edge.b.clone(), (new_cost, new_edge.clone()));
                            updated = true;
                        }
                    }
                }
                if !updated {
                    break;
                }
            }

            // Extra iteration: check for negative cycle.
            // If we can still relax any edge, then there is a negative cycle.
            for (_vertex, (cost, pred_edge)) in paths.clone().iter() {
                let new_edges = self.generate_edges(&pred_edge.b);
                for new_edge in new_edges {
                    if cost.clone() + new_edge.h.clone() < paths.get(&new_edge.b).unwrap().0 {
                        // Negative cycle detected.
                        // Reconstruct cycle by following predecessor pointers.
                        let mut cycle_vertex = new_edge.b.clone();
                        // To ensure we are inside the cycle, follow predecessor pointers n times.
                        for _ in 0..n {
                            let &(ref _c, ref edge) = paths.get(&cycle_vertex).unwrap();
                            cycle_vertex = edge.a.clone();
                        }
                        let start = cycle_vertex.clone();
                        let mut negative_cycle = Vec::new();
                        loop {
                            let edge = paths.get(&cycle_vertex).unwrap().1.clone();
                            negative_cycle.push(edge.clone());
                            cycle_vertex = edge.a.clone();
                            if cycle_vertex == start {
                                break;
                            }
                        }
                        return Err(negative_cycle);
                    }
                }
            }

            // Construct final result paths map with the complete route.
            let mut rpaths = HashMap::<K, (H, Vec<Rc<Edge<K, H>>>)>::new();
            for (vertex, (cost, edge)) in paths.iter() {
                let mut path = Vec::new();
                let mut current_edge = edge.clone();
                // Reconstruct path by following predecessor pointers.
                // Stop when reaching the starting vertex.
                while let Some(prev) = paths.get(&current_edge.a) {
                    // If the predecessor's vertex is the source, finish.
                    if current_edge.a == *from {
                        path.push(current_edge.clone());
                        break;
                    }
                    path.push(current_edge.clone());
                    current_edge = prev.1.clone();
                }
                path.reverse();
                rpaths.insert(vertex.clone(), (cost.clone(), path));
            }

            Ok(rpaths)
        }
    }
}

pub mod edge {
    use std::hash::{Hash, Hasher};

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Edge<K, H>
    where
        K: Hash + Ord,
        H: Eq + Ord + Hash + Clone,
    {
        pub id: usize,
        pub a: K,
        pub b: K,
        pub h: H,
    }

    impl<K, H> Edge<K, H>
    where
        K: Hash + Ord,
        H: Eq + Ord + Hash + Clone,
    {
        pub fn new(id: usize, a: K, b: K, h: H) -> Self {
            Self { id, a, b, h }
        }
    }

    impl<K, H> Hash for Edge<K, H>
    where
        K: Hash + Clone + Ord,
        H: Eq + Ord + Hash + Clone,
    {
        fn hash<T: Hasher>(&self, state: &mut T) {
            // The order of these calls is important.
            self.a.hash(state);
            self.b.hash(state);
        }
    }
}

pub mod graph {
    use std::{
        collections::{HashMap, HashSet},
        fmt::Display,
        hash::Hash,
        marker::PhantomData,
        ops::Add,
        rc::Rc,
        sync::{Arc, RwLock},
        usize,
    };

    use crate::edge::Edge;
    #[derive(Debug)]
    pub struct Graph<K, V, H>
    where
        K: Clone + Hash + Eq + Ord,
        V: IntoConnections<K, H> + Eq,
        H: Eq + Ord + Add<Output = H> + Hash + Default + Clone,
    {
        // Use a HashMap to associate each key K with its node (wrapped in Arc<RwLock<>>)
        pub map: HashMap<K, Arc<RwLock<V>>>,
        pub h: PhantomData<H>,
    }

    impl<K, V, H> Graph<K, V, H>
    where
        K: Clone + Hash + Eq + Ord + Display,
        V: IntoConnections<K, H> + Eq,
        H: Eq + Ord + Add<Output = H> + Hash + Default + Clone,
    {
        pub fn add_node(&mut self, key: K, node: Arc<RwLock<V>>) {
            self.map.insert(key, node);
        }
        pub fn new(map: HashMap<K, Arc<RwLock<V>>>) -> Self {
            Graph {
                map,
                h: PhantomData,
            }
        }

        pub fn propagate_path(
            &self,
            path: &(H, Vec<Rc<Edge<K, H>>>),
        ) -> Vec<(H, Vec<Rc<Edge<K, H>>>, K)> {
            let mut new_paths = Vec::new();
            println!("propagating path");
            if let Some(last_edge) = path.1.last() {
                println!("Last edge: {}", last_edge.b);
                let edges = self.generate_edges(&last_edge.b);
                for edge in edges {
                    let mut new_path = path.1.clone();
                    new_path.push(edge.clone());
                    let cost = path.0.clone() + edge.h.clone();
                    new_paths.push((cost, new_path, edge.b.clone()));
                }
            }
            new_paths
        }

        /// Retrieve edges from the graph for a given key.
        pub fn generate_edges(&self, key: &K) -> Vec<Rc<Edge<K, H>>> {
            let mut arr = Vec::new();
            if let Some(node_arc) = self.map.get(key) {
                let node = node_arc.read().unwrap();
                let connections = node.into_connections(&self.map);
                let mut i = 0;
                for conn in connections {
                    let edge = conn.into_edge(i.clone()).clone();
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
        fn into_connections(&self, map: &HashMap<K, Arc<RwLock<Self>>>) -> Vec<Self::Item>;
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
}
