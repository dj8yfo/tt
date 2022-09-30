use rand::{thread_rng, Rng};
use std::collections::HashSet;

pub mod common_tests;

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
pub fn cartesian_product(mut input: HashSet<String>) -> HashSet<String> {
    let prefixes = common_prefixes();

    input = input
        .into_iter()
        // this limit is to avoid stack overflows when testing without --release flag
        // as the chains get pretty long
        // and debug info appears to clog stack on recursive operations
        .filter(|item_y| !item_y.is_empty() && item_y.len() < 100)
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
}

pub fn attach_values(input: HashSet<String>) -> Vec<(String, u32)> {
    let mut rng = thread_rng();
    input.into_iter().map(|key| (key, rng.next_u32())).collect()
}
