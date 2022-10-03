use std::cmp::{Ord, Ordering};
use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;

#[allow(unused)]
#[derive(Eq, Clone, Copy)]
enum Version {
    Actual(u64),
    Link { linked_to: u64, this: u64 },
}

impl Version {
    fn num(&self) -> u64 {
        match self {
            Self::Actual(num) => *num,
            Self::Link { this, .. } => *this,
        }
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        self.num().cmp(&other.num())
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        self.num() == other.num()
    }
}

#[allow(unused)]
type VersionedMapTreelikeNoTrie<K, V> = VMapNoTrie<K, V>;

#[allow(unused)]
struct VMapNoTrie<K, V> {
    current_ver: u64,
    state: HashMap<K, BTreeMap<Version, Option<V>>>,
}

impl<K, V> VMapNoTrie<K, V> {
    fn new() -> Self {
        Self {
            current_ver: 0,
            state: HashMap::new(),
        }
    }

    fn get_version_mut(
        v_map: &mut BTreeMap<Version, Option<V>>,
        mut version: u64,
    ) -> (&Version, &mut Option<V>) {
        loop {
            let (vers, _) = v_map
                .range(..=Version::Actual(version))
                .rev()
                .next()
                .unwrap();
            if let Version::Link { linked_to, .. } = *vers {
                version = linked_to;
                continue;
            } else {
                break;
            }
        }
        v_map
            .range_mut(..=Version::Actual(version))
            .rev()
            .next()
            .unwrap()
    }
    fn get_version(
        v_map: &BTreeMap<Version, Option<V>>,
        mut version: u64,
    ) -> (&Version, &Option<V>) {
        loop {
            let (vers, _) = v_map
                .range(..=Version::Actual(version))
                .rev()
                .next()
                .unwrap();
            if let Version::Link { linked_to, .. } = *vers {
                version = linked_to;
                continue;
            } else {
                break;
            }
        }
        v_map
            .range(..=Version::Actual(version))
            .rev()
            .next()
            .unwrap()
    }
}

impl<K, V> super::VersionedMap<K, V> for VMapNoTrie<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    fn insert(&mut self, k: K, v: V) -> Option<V> {
        match self.state.get_mut(&k) {
            None => {
                let mut v_map = BTreeMap::new();
                v_map.insert(Version::Actual(self.current_ver), Some(v));
                self.state.insert(k, v_map);
                None
            }
            Some(v_map) => {
                let (v_prev, val) =
                    VMapNoTrie::<K, V>::get_version_mut(v_map, self.current_ver);
                if v_prev.num() == self.current_ver {
                    val.replace(v)
                } else {
                    let res = val.clone();
                    v_map.insert(Version::Actual(self.current_ver), Some(v));
                    res
                }
            }
        }
    }
    fn get(&self, k: &K) -> Option<&V> {
        match self.state.get(k) {
            None => None,
            Some(v_map) => {
                let (_, val) = VMapNoTrie::<K, V>::get_version(v_map, self.current_ver);
                val.as_ref()
            }
        }
    }
    fn remove(&mut self, k: &K) -> Option<V> {
        match self.state.get_mut(k) {
            None => None,
            Some(v_map) => {
                let (v_prev, val) =
                    VMapNoTrie::<K, V>::get_version_mut(v_map, self.current_ver);
                if v_prev.num() == self.current_ver {
                    val.take()
                } else {
                    let res = val.clone();
                    v_map.insert(Version::Actual(self.current_ver), None);
                    res
                }
            }
        }
    }

    fn checkpoint(&mut self, tag: String) {
        unimplemented!();
    }

    fn rollback(&mut self, tag: String) -> bool {
        unimplemented!();
    }

    fn prune(&mut self) {
        unimplemented!();
    }
}

#[cfg(test)]
use crate::test_helpers::common_tests::versioned_map_trait_tests;

#[cfg(test)]
versioned_map_trait_tests!(VMapNoTrie);
