pub mod trivial;
pub trait Mappy<K, V: Clone> {
    fn insert(&mut self, k: K, v: V) -> Option<V>;
    fn get(&self, k: &K) -> Option<&V>;
    fn remove(&mut self, k: &K) -> Option<V>;

    fn checkpoint(&mut self, tag: String);
    fn rollback(&mut self, tag: String) -> bool;
    fn prune(&mut self);
}
