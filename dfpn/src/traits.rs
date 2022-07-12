use crate::{Node, U};
use shogi_core::Move;

pub trait Position {
    fn hash_key(&self) -> u64;
    fn generate_legal_moves(&mut self, node: Node) -> Vec<(Move, u64)>;
    fn do_move(&mut self, m: Move);
    fn undo_move(&mut self, m: Move);
}

pub trait Table: Default {
    fn look_up_hash(&self, key: &u64) -> (U, U);
    fn put_in_hash(&mut self, key: u64, value: (U, U));
}
