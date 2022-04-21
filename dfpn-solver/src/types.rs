use crate::U;
use std::ops;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Node {
    Or,
    And,
}

impl ops::Not for Node {
    type Output = Node;

    fn not(self) -> Self::Output {
        match self {
            Node::Or => Node::And,
            Node::And => Node::Or,
        }
    }
}

pub trait Position {
    type M: Copy + PartialEq;

    fn hash_key(&self) -> u64;
    fn generate_legal_moves(&mut self, node: Node) -> Vec<(Self::M, u64)>;
    fn do_move(&mut self, m: Self::M);
    fn undo_move(&mut self, m: Self::M);
}

pub trait Table: Default {
    fn look_up_hash(&self, key: &u64) -> (U, U);
    fn put_in_hash(&mut self, key: u64, value: (U, U));
}

pub trait DFPN<P, T>
where
    P: Position,
    T: Table,
{
    fn dfpn(&mut self);
}
