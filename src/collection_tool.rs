use std::collections::HashSet;
use std::hash::Hash;

pub struct CollectionTool;
impl CollectionTool {
    pub fn intersect<T>(a: &mut HashSet<T>, b: &mut HashSet<T>) -> HashSet<T>
    where
        T: Hash,
        T: Eq,
    {
        let intersected_set: HashSet<T> = a.iter().filter_map(|v| b.take(v)).collect();
        return intersected_set;
    }

    // pub fn difference<T>(a: &mut HashSet<T>, b: &mut HashSet<T>) -> HashSet<T>
    // where
    // T: Hash
    // T: Eq,
    // {
    // let resulting_set: HashSet<T> = a.iter().filter_map(|v|
    // if b.contains(v) {
    // None
    // } else {
    // Some(v)
    // }).collect();
    // let mut resulting_set = HashSet::new();
    // let mut referencing_set = HashSet::new();

    // for value in a.iter() {
    // if !b.contains(value) {
    // referencing_set.insert(value);
    // }
    // }

    // for reference_value in referencing_set.iter() {
    // if let Some(taken_value) = a.take(reference_value.cloned()) {
    // resulting_set.insert(taken_value);
    // }
    // }

    // return resulting_set;
    // }
}
