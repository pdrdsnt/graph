use std::hash::{Hash, Hasher};

#[derive(Clone,PartialEq, Eq, PartialOrd, Ord)]
pub struct Edge<K, H>
where
    K: Hash + Clone + Ord,
    H: Eq + Ord + Hash + Clone
{
    pub id: usize,
    pub a: K,
    pub b: K,
    pub h: H,
}

impl<K, H> Hash for Edge<K, H>
where
    K: Hash + Clone + Ord,
    H: Eq + Ord + Hash + Clone
{
    fn hash<T: Hasher>(&self, state: &mut T) {
        // The order of these calls is important.
        self.id.hash(state);
        self.a.hash(state);
        self.b.hash(state);
    }
}