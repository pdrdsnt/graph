use std::{
    cmp::Ordering, collections::{HashMap, HashSet}, hash::{Hash, Hasher}, rc::Rc
};

use crate::edge::Edge;

#[derive(Clone)]
pub struct Path<K,H>
where
    K: Hash + Clone + Ord,
    H: Eq + Ord + Hash + Default + Clone,
    
{
    pub total: H, // Total cost of the path
    pub path: Vec<Rc<Edge<K,H>>>, // Sequence of edges
    pub nodes: HashSet<K>,
}

// -- Hash Implementation --
impl<K,H> Hash for Path<K,H>
where
    K: Hash + Clone + Ord,
    H: Eq + Ord + Hash + Default + Clone,
    
{
    fn hash<T: Hasher>(&self, state: &mut T) {
        self.total.hash(state); // Hash the total cost
        for edge in &self.path {
            edge.a.hash(state); // Hash the start node of the edge
            edge.b.hash(state); // Hash the end node of the edge
            edge.h.hash(state); // Hash the edge cost
        }
    }
}

// -- Equality Implementation --
impl<K, H> PartialEq for Path<K, H>
where
    K: Hash + Clone + Ord,
    H: Eq + Ord + Hash + Default + Clone,
    
{
    fn eq(&self, other: &Self) -> bool {
        self.total == other.total && self.path == other.path
    }
}

impl<K, H> Eq for  Path<K, H>
where
    K: Hash + Clone + Ord,
    H: Eq + Ord + Hash + Default + Clone,  
{}

// -- Ordering Implementation --
impl<K, H> Ord for Path<K, H>
where
    K: Hash + Clone + Ord,
    H: Eq + Ord + Hash + Default + Clone,
    
{
    fn cmp(&self, other: &Self) -> Ordering {
        other.total.cmp(&self.total) // Reverse order for min-heap behavior
    }
}

impl<K, H> PartialOrd for  Path<K, H>
where
    K: Hash + Clone + Ord,
    H: Eq + Ord + Hash + Default + Clone,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

//this is needed to use Rc<Path> inside a Binary Heap
// -- Define the SharedPath Wrapper --
#[derive(Clone)]
pub struct SharedPath<K, H>
where
    K: Hash + Clone + Ord,
    H: Eq + Ord + Hash + Default + Clone,
{
    pub inner: Rc<Path<K, H>>,
}

impl<K, H> SharedPath<K, H>
where
    K: Hash + Clone + Ord,
    H: Eq + Ord + Hash + Default + Clone,
{
    /// Create a new SharedPath from a Path
    pub fn new(path: Path<K, H>) -> Self {
        SharedPath {
            inner: Rc::new(path),
        }
    }
}

// -- Implement Ordering for SharedPath --
// We delegate comparison to the inner Path so that the BinaryHeap will order based on
// the content of Path rather than the pointer address.
impl<K, H> PartialEq for SharedPath<K, H>
where
    K: Hash + Clone + Ord,
    H: Eq + Ord + Hash + Default + Clone,
{
    fn eq(&self, other: &Self) -> bool {
        *self.inner == *other.inner
    }
}

impl<K, H> Eq for SharedPath<K, H> where 
    K: Hash + Clone + Ord,
    H: Eq + Ord + Hash + Default + Clone
{}

impl<K, H> PartialOrd for SharedPath<K, H>
where
    K: Hash + Clone + Ord,
    H: Eq + Ord + Hash + Default + Clone,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.inner.cmp(&other.inner))
    }
}

impl<K, H> Ord for SharedPath<K, H>
where
    K: Hash + Clone + Ord,
    H: Eq + Ord + Hash + Default + Clone,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.inner.cmp(&other.inner)
    }
}