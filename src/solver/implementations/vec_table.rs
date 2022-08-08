use dfpn::Table;
use num_traits::{PrimInt, SaturatingAdd, Unsigned};

pub struct VecTable<U = u32> {
    table: Vec<Option<(U, U)>>,
    mask: usize,
}

impl<U> VecTable<U>
where
    U: Clone,
{
    pub fn new(bits: u32) -> Self {
        Self {
            table: vec![None; 1 << bits],
            mask: (1 << bits) - 1,
        }
    }
}

impl<U> Default for VecTable<U>
where
    U: Clone,
{
    fn default() -> Self {
        Self::new(16)
    }
}

impl<U> Table for VecTable<U>
where
    U: Unsigned + PrimInt + SaturatingAdd + Default,
{
    type U = U;

    fn look_up_hash(&self, key: &u64) -> (U, U) {
        self.table[(*key as usize) & self.mask].unwrap_or((Self::U::one(), Self::U::one()))
    }
    fn put_in_hash(&mut self, key: u64, value: (U, U)) {
        self.table[(key as usize) & self.mask] = Some(value);
    }
}
