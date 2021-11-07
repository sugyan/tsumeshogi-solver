use crate::{Table, U};

pub struct VecTable {
    table: Vec<(U, U)>,
    mask: usize,
    len: usize,
}

impl VecTable {
    pub fn new(bits: u32) -> Self {
        Self {
            table: vec![(1, 1); 1 << bits],
            mask: (1 << bits) - 1,
            len: 0,
        }
    }
}

impl Table for VecTable {
    type T = usize;
    fn look_up_hash(&self, key: &Self::T) -> (U, U) {
        self.table[*key & self.mask]
    }
    fn put_in_hash(&mut self, key: Self::T, value: (U, U)) {
        if self.look_up_hash(&key) == (1, 1) {
            self.len += 1;
        }
        self.table[key & self.mask] = value;
    }
    fn len(&self) -> usize {
        self.len
    }
}
