use crate::dfpn::dfpn_solver;
use crate::{INF, U};
use std::ops;
use yasai::Position;

#[derive(Clone, Copy, PartialEq)]
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

    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
}

pub trait DFPN<T>: dfpn_solver::Solve<T>
where
    T: Table,
{
    // ルートでの反復深化
    fn dfpn(&mut self, pos: Position) {
        let hash = self.set_position(pos);
        // ルートでの反復深化
        let (pn, dn) = self.mid(hash, INF - 1, INF - 1, Node::Or);
        if pn != INF && dn != INF {
            self.mid(hash, INF, INF, Node::Or);
        }
    }
}
