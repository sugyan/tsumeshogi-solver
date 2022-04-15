use crate::{Table, U};

pub struct VecTable {
    table: Vec<Option<(U, U)>>,
    mask: usize,
    len: usize,
}

impl VecTable {
    pub fn new(bits: u32) -> Self {
        Self {
            table: vec![None; 1 << bits],
            mask: (1 << bits) - 1,
            len: 0,
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
        if self.table[(key as usize) & self.mask].is_none() {
            self.len += 1;
        }
        self.table[(key as usize) & self.mask] = Some(value);
    }
    fn len(&self) -> usize {
        self.len
    }
    fn is_empty(&self) -> bool {
        self.len == 0
    }
}
