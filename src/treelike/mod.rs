use std::{collections::HashMap, marker::PhantomData};

mod as_bytes;
mod node;

use as_bytes::AsFromBytes;
use node::Node;
#[allow(unused)]
type VersionedMapTreeLike<K, V> = VMapTree<K, V>;

pub struct VMapTree<K, V> {
    root: Node<V>,
    snapshots: HashMap<String, Node<V>>,

    _phantom_data: PhantomData<K>,
}
impl<K, V> VMapTree<K, V> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            root: Node::new(),
            snapshots: HashMap::new(),
            _phantom_data: PhantomData,
        }
    }
}

fn bytes<K: AsFromBytes>(input: &K) -> &[u8] {
    let bytes = input.as_bytes();
    if bytes.is_empty() {
        panic!("not bothering here with empty keys");
    }
    bytes
}

impl<K, V> Drop for VMapTree<K, V> {
    fn drop(&mut self) {
        for (_snapshot_tag, mut node) in self.snapshots.drain() {
            node::vacuum_clean(&mut node);
        }
        node::vacuum_clean(&mut self.root);
    }
}

impl<K, V> super::VersionedMap<K, V> for VMapTree<K, V>
where
    K: as_bytes::AsFromBytes,
    V: Clone,
{
    fn insert(&mut self, k: K, v: V) -> Option<V> {
        let iter = bytes(&k).iter();
        self.root.insert(iter, v)
    }
    fn get(&self, k: &K) -> Option<&V> {
        let iter = bytes(k).iter();
        self.root.get(iter)
    }
    fn remove(&mut self, k: &K) -> Option<V> {
        let iter = bytes(k).iter();
        let (_, res) = self.root.remove(iter);
        res
    }

    fn checkpoint(&mut self, tag: String) {
        if let Some(mut prev) = self.snapshots.insert(tag, self.root.clone()) {
            node::vacuum_clean(&mut prev);
        }
    }

    fn rollback(&mut self, tag: String) -> bool {
        if let Some(snapshot) = self.snapshots.get(&tag) {
            node::vacuum_clean(&mut self.root);
            self.root = snapshot.clone();
            return true;
        }
        false
    }

    fn prune(&mut self) {
        for (_snapshot_tag, mut node) in self.snapshots.drain() {
            node::vacuum_clean(&mut node);
        }
    }
}

#[cfg(test)]
use crate::test_helpers::common_tests::versioned_map_trait_tests;

#[cfg(test)]
versioned_map_trait_tests!(VMapTree);
