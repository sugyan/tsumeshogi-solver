use crate::{Table, U};

pub struct VecTable {
    table: Vec<Option<(U, U)>>,
    mask: usize,
}

impl VecTable {
    pub fn new(bits: u32) -> Self {
        Self {
            table: vec![None; 1 << bits],
            mask: (1 << bits) - 1,
        }
    }
}

impl Default for VecTable {
    fn default() -> Self {
        Self::new(16)
    }
}

impl Table for VecTable {
    fn look_up_hash(&self, key: &u64) -> (U, U) {
        self.table[(*key as usize) & self.mask].unwrap_or((1, 1))
    }
    fn put_in_hash(&mut self, key: u64, value: (U, U)) {
        self.table[(key as usize) & self.mask] = Some(value);
    }
}
