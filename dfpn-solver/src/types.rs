use crate::U;
use std::ops;
use yasai::Position;

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

pub trait Table: Default {
    fn look_up_hash(&self, key: &u64) -> (U, U);
    fn put_in_hash(&mut self, key: u64, value: (U, U));
}

pub trait DFPN<T>
where
    T: Table,
{
    // ルートでの反復深化
    fn dfpn(&mut self, pos: Position);
}
