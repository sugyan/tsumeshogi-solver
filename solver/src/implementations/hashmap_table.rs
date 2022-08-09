use dfpn::Table;
use num_traits::{PrimInt, SaturatingAdd, Unsigned};
use std::collections::HashMap;

#[derive(Default)]
pub struct HashMapTable<U = u32> {
    table: HashMap<u64, (U, U)>,
}

impl<U> HashMapTable<U> {
    pub fn new() -> Self {
        Self {
            table: HashMap::new(),
        }
    }
}

impl<U> Table for HashMapTable<U>
where
    U: Unsigned + PrimInt + SaturatingAdd + Default,
{
    type U = U;

    fn look_up_hash(&self, key: &u64) -> (U, U) {
        if let Some(&v) = self.table.get(key) {
            v
        } else {
            (Self::U::one(), Self::U::one())
        }
    }
    fn put_in_hash(&mut self, key: u64, value: (U, U)) {
        self.table.insert(key, value);
    }
}
