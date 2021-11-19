use crate::{Table, U};
use std::collections::HashMap;
use std::hash::Hash;

#[derive(Default)]
pub struct HashMapTable<T = u64> {
    table: HashMap<T, (U, U)>,
}

impl<T> HashMapTable<T> {
    pub fn new() -> Self {
        Self {
            table: HashMap::new(),
        }
    }
}

impl<V> Table for HashMapTable<V>
where
    V: Default + Eq + Hash,
{
    type T = V;
    fn look_up_hash(&self, key: &Self::T) -> (U, U) {
        if let Some(&v) = self.table.get(key) {
            v
        } else {
            (1, 1)
        }
    }
    fn put_in_hash(&mut self, key: Self::T, value: (U, U)) {
        self.table.insert(key, value);
    }
    fn len(&self) -> usize {
        self.table.len()
    }
    fn is_empty(&self) -> bool {
        self.table.is_empty()
    }
}
