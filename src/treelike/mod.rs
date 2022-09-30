use std::{marker::PhantomData, collections::HashMap};

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
            return true
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
mod tests {
    use std::collections::HashSet;

    use quickcheck::{QuickCheck, TestResult};

    use super::VMapTree;

    use crate::test_helpers::{attach_values, cartesian_product};
    

    #[test]
    fn get_inserted() {
        fn property(keys: HashSet<String>) -> TestResult {
            let entries = attach_values(cartesian_product(keys));

            let treelike = VMapTree::new();
            let mut under_test: Box<dyn crate::VersionedMap<String, u32>> =
                Box::new(treelike);

            for (key, value) in entries.clone() {
                assert_eq!(None, under_test.get(&key));
                under_test.insert(key, value);
            }
            for (key, value) in &entries {
                assert_eq!(Some(value), under_test.get(key));
            }

            TestResult::passed()
        }
        // quickcheck doesn't work with closures, unfortunately
        QuickCheck::new().quickcheck(property as fn(HashSet<String>) -> TestResult);
    }

    #[test]
    fn remove_inserted() {
        fn property(keys: HashSet<String>) -> TestResult {
            let entries = attach_values(cartesian_product(keys));

            let treelike = VMapTree::new();
            let mut under_test: Box<dyn crate::VersionedMap<String, u32>> =
                Box::new(treelike);

            for (key, value) in entries.clone() {
                under_test.insert(key, value);
            }

            for (key, value) in entries {
                let val_removed = under_test.remove(&key);
                assert_eq!(Some(value), val_removed);
                assert_eq!(None, under_test.get(&key));
            }

            TestResult::passed()
        }
        // quickcheck doesn't work with closures, unfortunately
        QuickCheck::new().quickcheck(property as fn(HashSet<String>) -> TestResult);
    }

    #[test]
    fn checkpoint_rollback_union() {
        fn property(keys_one: HashSet<String>, keys_two: HashSet<String>) -> TestResult {
            let mut keys_one = cartesian_product(keys_one);
            let keys_two = cartesian_product(keys_two);

            keys_one = keys_one.difference(&keys_two).map(Clone::clone).collect();

            let epochs: Vec<Vec<(String, u32)>> = [keys_one, keys_two]
                .into_iter()
                .map(attach_values)
                .collect();

            let treelike = VMapTree::new();
            let mut under_test: Box<dyn crate::VersionedMap<String, u32>> =
                Box::new(treelike);

            under_test.checkpoint("EMPTY".to_owned());

            for (key, value) in epochs[0].clone() {
                under_test.insert(key, value);
            }

            under_test.checkpoint("ONE".to_owned());

            for (key, value) in epochs[1].clone() {
                under_test.insert(key, value);
            }

            under_test.checkpoint("TWO".to_owned());

            for entries in &epochs {
                for (k, v) in entries {
                    assert_eq!(Some(v), under_test.get(k))
                }
            }
            assert!(under_test.rollback("EMPTY".to_owned()));
            for entries in &epochs {
                for (k, _v) in entries {
                    assert_eq!(None, under_test.get(k))
                }
            }
            assert!(under_test.rollback("ONE".to_owned()));
            for (k, v) in &epochs[0] {
                assert_eq!(Some(v), under_test.get(k))
            }
            for (k, _v) in &epochs[1] {
                assert_eq!(None, under_test.get(k))
            }

            assert!(under_test.rollback("TWO".to_owned()));
            for entries in &epochs {
                for (k, v) in entries {
                    assert_eq!(Some(v), under_test.get(k))
                }
            }

            TestResult::passed()
        }
        // quickcheck doesn't work with closures, unfortunately
        QuickCheck::new()
            .quickcheck(property as fn(HashSet<String>, HashSet<String>) -> TestResult);
    }

    #[test]
    fn checkpoint_rollback_difference() {
        fn property(
            keys_left: HashSet<String>,
            keys_deleted: HashSet<String>,
        ) -> TestResult {
            let mut keys_left = cartesian_product(keys_left);
            let keys_deleted = cartesian_product(keys_deleted);

            keys_left = keys_left
                .difference(&keys_deleted)
                .map(Clone::clone)
                .collect();
            let keys_full = keys_left.union(&keys_deleted).map(Clone::clone).collect();

            let epochs: Vec<Vec<(String, u32)>> = [keys_full, keys_deleted]
                .into_iter()
                .map(attach_values)
                .collect();

            let treelike = VMapTree::new();
            let mut under_test: Box<dyn crate::VersionedMap<String, u32>> =
                Box::new(treelike);

            for (key, value) in epochs[0].clone().into_iter() {
                under_test.insert(key, value);
            }

            under_test.checkpoint("ONE".to_owned());

            for (key, _v) in &epochs[1] {
                under_test.remove(key);
            }

            under_test.checkpoint("TWO".to_owned());

            assert!(under_test.rollback("ONE".to_owned()));
            for (k, v) in &epochs[0] {
                assert_eq!(Some(v), under_test.get(k));
            }

            assert!(under_test.rollback("TWO".to_owned()));
            for (k, _v) in &epochs[1] {
                assert_eq!(None, under_test.get(k));
            }

            TestResult::passed()
        }
        // quickcheck doesn't work with closures, unfortunately
        QuickCheck::new()
            .quickcheck(property as fn(HashSet<String>, HashSet<String>) -> TestResult);
    }

    #[test]
    fn snapshots_prune() {
        fn property(keys_one: HashSet<String>) -> TestResult {
            let entries_one = attach_values(cartesian_product(keys_one));

            let treelike = VMapTree::new();
            let mut under_test: Box<dyn crate::VersionedMap<String, u32>> =
                Box::new(treelike);

            under_test.checkpoint("EMPTY".to_owned());

            for (key, value) in entries_one.clone().into_iter() {
                under_test.insert(key, value);
            }

            under_test.checkpoint("ONE".to_owned());

            under_test.prune();
            assert!(!under_test.rollback("EMPTY".to_owned()));

            for (k, v) in &entries_one {
                assert_eq!(Some(v), under_test.get(k))
            }
            assert!(!under_test.rollback("ONE".to_owned()));

            TestResult::passed()
        }
        // quickcheck doesn't work with closures, unfortunately
        QuickCheck::new().quickcheck(property as fn(HashSet<String>) -> TestResult);
    }


}
