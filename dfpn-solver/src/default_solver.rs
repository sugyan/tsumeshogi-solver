use crate::impl_hashmap_table::HashMapTable;
use crate::solve::Solve;
use crate::{Node, Position, Table, U};

pub struct DefaultSolver<P, T = HashMapTable> {
    pub pos: P,
    table: T,
}

impl<P, T> DefaultSolver<P, T>
where
    P: Position,
    T: Table,
{
    pub fn new(pos: P) -> Self {
        Self {
            pos,
            table: T::default(),
        }
    }
}

impl<P, T> Solve<P, T> for DefaultSolver<P, T>
where
    P: Position,
    T: Table,
{
    fn hash_key(&self) -> u64 {
        self.pos.hash_key()
    }
    fn generate_legal_moves(&mut self, node: Node) -> Vec<(P::M, u64)> {
        self.pos.generate_legal_moves(node)
    }
    fn do_move(&mut self, m: P::M) {
        self.pos.do_move(m)
    }
    fn undo_move(&mut self, m: P::M) {
        self.pos.undo_move(m)
    }
    // ハッシュを引く (本当は優越関係が使える)
    fn look_up_hash(&self, key: &u64) -> (U, U) {
        self.table.look_up_hash(key)
    }
    // ハッシュに記録
    fn put_in_hash(&mut self, key: u64, value: (U, U)) {
        self.table.put_in_hash(key, value)
    }
}
