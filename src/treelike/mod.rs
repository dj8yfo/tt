use std::marker::PhantomData;

mod node;
mod as_bytes;

use as_bytes::AsFromBytes;
use node::Node;
#[allow(unused)]
type VersionedMapTreeLike<K, V> = VMapTree<K, V>;

pub struct VMapTree<K, V> {
    root: Node<V>,

    _phantom_data: PhantomData<K>,
}
impl<K, V> VMapTree<K, V> {

    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            root: Node::new(),
            _phantom_data: PhantomData,
        }
    }
}

fn bytes<K: AsFromBytes> (input: &K) -> &[u8] {
    let bytes = input.as_bytes();
    if bytes.is_empty(){
        panic!("not bothering here with empty keys");
    }
    bytes
}

impl<K, V> super::VersionedMap<K, V> for VMapTree<K, V> 
where K: as_bytes::AsFromBytes,
    V: Clone {

    fn insert(&mut self, k: K, v: V) -> Option<V> {
        let iter = bytes(&k).iter();
        let _prev_root = self.root.clone();
        // TODO: implement manual drop
        self.root.insert(iter, v)

    }
    fn get(&self, k: &K) -> Option<&V> {
        let iter = bytes(k).iter();
        self.root.get(iter)
    }
    fn remove(&mut self, k: &K) -> Option<V> {
        unimplemented!("");
    }

    fn checkpoint(&mut self, tag: String){
        unimplemented!("");
    }
    fn rollback(&mut self, tag: String) -> bool{
        unimplemented!("");
    }
    fn prune(&mut self){
        unimplemented!("");
    }

}


#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use quickcheck::{QuickCheck, TestResult};

    use super::*;

    use crate::test_helpers::{attach_values, cartesian_product};


    #[test]
    fn get_inserted() {
        fn property(keys: HashSet<String>) -> TestResult {
            let entries = attach_values(cartesian_product(keys));

            let hashmap = VMapTree::new();
            let mut under_test: Box<dyn crate::VersionedMap<String, u32>> = Box::new(hashmap);

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
}
