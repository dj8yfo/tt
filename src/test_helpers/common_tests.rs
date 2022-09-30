macro_rules! versioned_map_trait_tests {
    ($impl_name:ident) => {
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

                    let hashmap = $impl_name::new();
                    let mut under_test: Box<dyn crate::VersionedMap<String, u32>> =
                        Box::new(hashmap);

                    for (key, value) in entries.clone() {
                        assert_eq!(None, under_test.get(&key));
                        under_test.insert(key, value);
                    }
                    for (key, value) in &entries {
                        assert_eq!(Some(value), under_test.get(key));
                    }

                    TestResult::passed()
                }
                QuickCheck::new()
                    .quickcheck(property as fn(HashSet<String>) -> TestResult);
            }

            #[test]
            fn remove_inserted() {
                fn property(keys: HashSet<String>) -> TestResult {
                    let entries = attach_values(cartesian_product(keys));

                    let hashmap = $impl_name::new();
                    let mut under_test: Box<dyn crate::VersionedMap<String, u32>> =
                        Box::new(hashmap);

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
                QuickCheck::new()
                    .quickcheck(property as fn(HashSet<String>) -> TestResult);
            }

            #[test]
            fn checkpoint_rollback_union() {
                fn property(
                    keys_one: HashSet<String>,
                    keys_two: HashSet<String>,
                ) -> TestResult {
                    let mut keys_one = cartesian_product(keys_one);
                    let keys_two = cartesian_product(keys_two);

                    keys_one =
                        keys_one.difference(&keys_two).map(Clone::clone).collect();

                    let epochs: Vec<Vec<(String, u32)>> = [keys_one, keys_two]
                        .into_iter()
                        .map(attach_values)
                        .collect();

                    let hashmap = $impl_name::new();
                    let mut under_test: Box<dyn crate::VersionedMap<String, u32>> =
                        Box::new(hashmap);

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
                QuickCheck::new().quickcheck(
                    property as fn(HashSet<String>, HashSet<String>) -> TestResult,
                );
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
                    let keys_full =
                        keys_left.union(&keys_deleted).map(Clone::clone).collect();

                    let epochs: Vec<Vec<(String, u32)>> = [keys_full, keys_deleted]
                        .into_iter()
                        .map(attach_values)
                        .collect();

                    let hashmap = $impl_name::new();
                    let mut under_test: Box<dyn crate::VersionedMap<String, u32>> =
                        Box::new(hashmap);

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
                QuickCheck::new().quickcheck(
                    property as fn(HashSet<String>, HashSet<String>) -> TestResult,
                );
            }

            #[test]
            fn snapshots_prune() {
                fn property(keys_one: HashSet<String>) -> TestResult {
                    let entries_one = attach_values(cartesian_product(keys_one));

                    let hashmap = $impl_name::new();
                    let mut under_test: Box<dyn crate::VersionedMap<String, u32>> =
                        Box::new(hashmap);

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
                QuickCheck::new()
                    .quickcheck(property as fn(HashSet<String>) -> TestResult);
            }
        }
    };
}
pub(crate) use versioned_map_trait_tests;
