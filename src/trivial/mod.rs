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

impl<K, V> VMapTriv<K, V> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let mut map = HashMap::<Version, HashMap<K, V>>::new();
        map.insert(Version::Latest, HashMap::new());
        Self { inner: map }
    }
}

impl<K, V> super::Mappy<K, V> for VMapTriv<K, V>
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
            return true;
        }
        false
    }

    fn prune(&mut self) {
        let latest = self.inner.remove(&Version::Latest).unwrap();

        for (_k, _v) in self.inner.drain() {}
        self.inner.insert(Version::Latest, latest);
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

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
    fn cartesian_product(mut input: HashSet<String>) -> Vec<(String, u32)> {
        let prefixes = common_prefixes();
        let mut rng = thread_rng();

        input = input
            .into_iter()
            .filter(|item_y| !item_y.is_empty())
            .collect();

        let product: HashSet<String> = prefixes
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

        product
            .into_iter()
            .map(|key| (key, rng.next_u32()))
            .collect()
    }

    #[test]
    fn get_inserted() {
        fn property(keys: HashSet<String>) -> TestResult {
            let entries = cartesian_product(keys);

            let hashmap = VMapTriv::new();
            let mut under_test: Box<dyn crate::Mappy<String, u32>> = Box::new(hashmap);

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
            let entries = cartesian_product(keys);

            let hashmap = VMapTriv::new();
            let mut under_test: Box<dyn crate::Mappy<String, u32>> = Box::new(hashmap);

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
            let keys_one = keys_one.difference(&keys_two).map(Clone::clone).collect();

            let epochs: Vec<Vec<(String, u32)>> = [keys_one, keys_two]
                .into_iter()
                .map(cartesian_product)
                .collect();

            let hashmap = VMapTriv::new();
            let mut under_test: Box<dyn crate::Mappy<String, u32>> = Box::new(hashmap);

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
            let keys_left: HashSet<String> = keys_left
                .difference(&keys_deleted)
                .map(Clone::clone)
                .collect();
            let keys_full = keys_left.union(&keys_deleted).map(Clone::clone).collect();

            let epochs: Vec<Vec<(String, u32)>> = [keys_full, keys_deleted]
                .into_iter()
                .map(cartesian_product)
                .collect();

            let hashmap = VMapTriv::new();
            let mut under_test: Box<dyn crate::Mappy<String, u32>> = Box::new(hashmap);

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
            let entries_one = cartesian_product(keys_one);

            let hashmap = VMapTriv::new();
            let mut under_test: Box<dyn crate::Mappy<String, u32>> = Box::new(hashmap);

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
