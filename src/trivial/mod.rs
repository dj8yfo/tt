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

impl<K, V> super::VersionedMap<K, V> for VMapTriv<K, V>
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
use crate::test_helpers::common_tests::versioned_map_trait_tests;

#[cfg(test)]
versioned_map_trait_tests!(VMapTriv);
