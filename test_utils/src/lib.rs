use std::collections::HashSet;
use std::hash::Hash;
use std::fmt::Debug;

pub fn assert_set_equality<T>(got: Vec<T>, expected: Vec<T>)
    where T: Clone + Eq + Hash + Debug {
    let got_hash: HashSet<T> = got.iter().cloned().collect();
    let expected_hash: HashSet<T> = expected.iter().cloned().collect();
    if got_hash != expected_hash {
        let unwanted: HashSet<&T> = got_hash.difference(&expected_hash).collect();
        let needed: HashSet<&T> = expected_hash.difference(&got_hash).collect();
        panic!("set inequality! expected len {}, got {}\nmissing {:?}\nunwanted {:?}",
            expected_hash.len(), got_hash.len(), needed, unwanted);
    }
}
