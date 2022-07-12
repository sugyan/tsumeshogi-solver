use dfpn::{Table, U};
use std::collections::HashMap;

#[derive(Default)]
pub struct HashMapTable {
    table: HashMap<u64, (U, U)>,
}

impl HashMapTable {
    pub fn new() -> Self {
        Self {
            table: HashMap::new(),
        }
    }
}

impl Table for HashMapTable {
    fn look_up_hash(&self, key: &u64) -> (U, U) {
        if let Some(&v) = self.table.get(key) {
            v
        } else {
            (1, 1)
        }
    }
    fn put_in_hash(&mut self, key: u64, value: (U, U)) {
        self.table.insert(key, value);
    }
}
