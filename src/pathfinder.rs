use crate::graph::{Connection, Graph, IntoConnections};
use crate::path::{Path, SharedPath};
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::hash::Hash;
use std::ops::Add;
use std::rc::Rc;

pub trait Pathfind<K, V, H>
where
    K: Clone + Hash + Eq + Ord,
    V: IntoConnections<K, H> + Default + Eq + Hash + Clone + Ord,
    H: Eq + Ord + Add<Output = H> + Hash + Default + Clone,
    for<'a> H: Add<&'a H, Output = H> + PartialOrd<&'a H>,
{
    fn a_star(&mut self, from: &K, to: &K) -> Vec<SharedPath<K, H>>;
    fn bellman_ford(&self, source: &K, target: &K) -> Option<SharedPath<K, H>>;
}

impl<K, V, H> Pathfind<K, V, H> for Graph<K, V, H>
where
    K: Clone + Hash + Eq + Ord,
    V: IntoConnections<K, H> + Default + Eq + Hash + Clone + Ord,
    H: Eq + Ord + Add<Output = H> + Hash + Default + Clone,
    for<'a> H: Add<&'a H, Output = H> + PartialOrd<&'a H>,
{
    //a star assumes weights are aways positive
    fn a_star(&mut self, from: &K, to: &K) -> Vec<SharedPath<K, H>> {
        // For demonstration, we're simply updating edges and gathering the connected nodes.
        // In a real pathfinding algorithm, you would continue processing until you reach `to`.
        let mut closed_nodes = HashSet::<K>::new();
        let mut open_paths = BinaryHeap::<SharedPath<K, H>>::new();
        let mut all_paths = Vec::new();
        let edges = self.generate_edges(from);

        for e in edges {
            let mut v = SharedPath {
                inner: Rc::new(Path {
                    total: e.h.clone(),
                    path: vec![e.clone()],
                    nodes: HashSet::from([e.a.clone(), e.b.clone()]),
                }),
            };
            open_paths.push(v);
        }

        while let Some(open) = open_paths.pop() {
            // Get the last edge in the current path.
            if let Some(last) = open.inner.path.last() {
                if closed_nodes.contains(&last.b) {
                    continue;
                }
                closed_nodes.insert(last.b.clone());
                // If the destination of the last edge is the target, we have found a solution.
                if &last.b == to {
                    all_paths.push(open.clone());
                    break;
                }
                // Clone the current path for reuse when extending with each new edge.
                let base_path = open.clone();
                let new_paths = self.propagate_path(base_path);
                // For each new edge, extend the current path.
                for o in new_paths {
                    open_paths.push(o);
                }
            }
        }
        all_paths
    }

    fn bellman_ford(&self, source: &K, target: &K) -> Option<SharedPath<K, H>> {
        // This map will hold the best (lowest cost) path found so far for each node.
        let mut best_paths: HashMap<K, SharedPath<K, H>> = HashMap::new();

        // Initialize the source: cost is zero, and path is empty.
        let init_path = Path {
            total: H::default(), // Assume H::default() is 0.
            nodes: std::collections::HashSet::from([source.clone()]),
            path: Vec::new(),
        };
        best_paths.insert(source.clone(), SharedPath::new(init_path));

        let num_nodes = self.map.len();

        // Relax all edges |V| - 1 times.
        for _ in 0..(num_nodes - 1) {
            let mut updated = false;
            // Iterate over all nodes in the graph.
            // Collect the keys from best_paths into a temporary Vec.
            let keys: Vec<K> = best_paths.keys().cloned().collect();

            for node_key in keys {
                // Clone out the current path to drop the immutable borrow.
                if let Some(current_path) = best_paths.get(&node_key).cloned() {
                    let outgoing_edges = self.generate_edges(&node_key);
                    for edge in outgoing_edges {
                        // Skip if the path already contains the destination (to avoid cycles).
                        if current_path.inner.nodes.contains(&edge.b) {
                            continue;
                        }
                        let new_total = current_path.inner.total.clone() + edge.h.clone();
                        // Check if we need to update best_paths for edge.b.
                        let relax = match best_paths.get(&edge.b).cloned() {
                            Some(existing_path) => new_total < existing_path.inner.total,
                            None => true,
                        };
                        if relax {
                            let mut new_nodes = current_path.inner.nodes.clone();
                            new_nodes.insert(edge.b.clone());
                            let mut new_edges = current_path.inner.path.clone();
                            new_edges.push(edge.clone());
                            let new_path = Path {
                                total: new_total,
                                nodes: new_nodes,
                                path: new_edges,
                            };
                            best_paths.insert(edge.b.clone(), SharedPath::new(new_path));
                            updated = true;
                        }
                    }
                }
            }

            // If no update occurred in this pass, then the best paths have stabilized.
            if !updated {
                break;
            }
        }

        // Optionally, you might run one more pass to detect negative cycles.
        // For now, we assume there are no negative cycles.

        best_paths.remove(target)
    }
}
