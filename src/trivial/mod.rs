use std::collections::HashMap;
use std::hash::Hash;

#[allow(unused)]
#[derive(Hash, Eq, PartialEq)]
enum Version {
    Latest,
    Tagged(String),
}

#[allow(unused)]
type VersionedMapTrivial<K, V> = VMapTriv<K, V>;

#[allow(unused)]
pub struct VMapTriv<K, V> {
    inner: HashMap<Version, HashMap<K, V>>,
}

pub trait Mappy<K, V: Clone> {
    fn insert(&mut self, k: K, v: V) -> Option<V>;
    fn get(&self, k: &K) -> Option<&V>;
    fn remove(&mut self, k: &K) -> Option<V>;

    fn checkpoint(&mut self, tag: String);
    fn rollback(&mut self, tag: String) -> bool;
    fn prune(&mut self);
}

impl<K, V> VMapTriv<K, V> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let mut map = HashMap::<Version, HashMap<K, V>>::new();
        map.insert(Version::Latest, HashMap::new());
        Self { inner: map }
    }
}

impl<K, V> Mappy<K, V> for VMapTriv<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,

{
    #[allow(unused)]
    fn insert(&mut self, k: K, v: V) -> Option<V> {
        let latest_map = self.inner.get_mut(&Version::Latest).unwrap();

        latest_map.insert(k, v)
    }

    #[allow(unused)]
    fn get(&self, k: &K) -> Option<&V> {
        let latest_map = self.inner.get(&Version::Latest).unwrap();
        latest_map.get(k)
    }

    #[allow(unused)]
    fn remove(&mut self, k: &K) -> Option<V>
    where
        K: Hash + Eq,
    {
        let latest_map = self.inner.get_mut(&Version::Latest).unwrap();

        latest_map.remove(k)
    }

    fn checkpoint(&mut self, tag: String) {
        let latest_map = self.inner.get(&Version::Latest).unwrap();
        let snapshot = latest_map.clone();

        self.inner.insert(Version::Tagged(tag), snapshot);

    }

    fn rollback(&mut self, tag: String) -> bool {
        if let Some(snapshot) = self.inner.get(&Version::Tagged(tag)) {
            self.inner.insert(Version::Latest, snapshot.clone());
            return true
        }
        false
    }

    fn prune(&mut self) {
        let latest = self.inner.remove(&Version::Latest).unwrap();

        for (_k, _v) in self.inner.drain() {
        }
        self.inner.insert(Version::Latest, latest);
    }

}

#[cfg(test)]
mod tests {
    use quickcheck::{QuickCheck, TestResult};
    use rand::{thread_rng, Rng};

    use super::*;

    fn common_prefixes() -> Vec<String> {
        vec![
            "".to_owned(),
            "a".to_owned(),
            "ab".to_owned(),
            "abc".to_owned(),
            "abcd".to_owned(),
        ]
    }

    #[allow(clippy::map_flatten)]
    #[allow(clippy::manual_retain)]
    fn cartesian_product(mut input: Vec<String>) -> Vec<(String, u32)> {
        let prefixes = common_prefixes();
        let mut rng = thread_rng();

        input = input
            .into_iter()
            .filter(|item_y| !item_y.is_empty())
            .collect();

        let mut product: Vec<String> = prefixes
            .iter()
            .map(|item_x| {
                input.iter().map(move |item_y| {
                    let mut new_str = item_x.clone();
                    let pushed = &item_y;
                    new_str.push_str(pushed);
                    new_str
                })
            })
            .flatten()
            .collect();
        product.sort_unstable();
        product.dedup();

        product
            .into_iter()
            .map(|key| (key, rng.next_u32()))
            .collect()
    }
    #[test]
    fn get_what_you_give_continuous() {
        fn property(keys: Vec<String>) -> TestResult {
            let entries = cartesian_product(keys);
            let entries_to_remove = entries.clone();

            let hashmap = VMapTriv::new();
            let mut system_under_test: Box<dyn Mappy<String, u32>> = Box::new(hashmap);
            for entry in entries.into_iter() {
                let (key, value) = entry;
                let k_clone = key.clone();

                system_under_test.insert(key, value);
                assert_eq!(Some(&value), system_under_test.get(&k_clone));
            }

            for entry in entries_to_remove.into_iter() {
                let (key, value) = entry;

                let val_removed = system_under_test.remove(&key);
                assert_eq!(Some(value), val_removed);
                assert_eq!(None, system_under_test.get(&key));
            }

            TestResult::passed()
        }
        // quickcheck doesn't work with closures, unfortunately
        QuickCheck::new().quickcheck(property as fn(Vec<String>) -> TestResult);
    }
}
