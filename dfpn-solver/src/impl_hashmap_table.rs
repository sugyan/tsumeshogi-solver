use crate::{Table, U};
use std::collections::HashMap;
use std::hash::Hash;

pub struct HashMapTable<T>
where
    T: Eq + Hash,
{
    table: HashMap<T, (U, U)>,
}

impl<T> HashMapTable<T>
where
    T: Eq + Hash,
{
    pub fn new() -> Self {
        Self {
            table: HashMap::new(),
        }
    }
}

impl<T> Default for HashMapTable<T>
where
    T: Eq + Hash,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<V> Table for HashMapTable<V>
where
    V: Eq + Hash,
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
