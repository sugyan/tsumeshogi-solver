use crate::Node;
use num_traits::{PrimInt, SaturatingAdd, Unsigned};
use shogi_core::Move;

pub trait Position {
    type M: Copy + Into<Move>;

    fn hash_key(&self) -> u64;
    fn generate_legal_moves(&mut self, node: Node) -> Vec<(Self::M, u64)>;
    fn do_move(&mut self, m: Self::M);
    fn undo_move(&mut self, m: Self::M);
}

pub trait Table: Default {
    type U: Unsigned + PrimInt + SaturatingAdd;

    fn look_up_hash(&self, key: &u64) -> (Self::U, Self::U);
    fn put_in_hash(&mut self, key: u64, value: (Self::U, Self::U));
}
