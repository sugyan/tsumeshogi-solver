use crate::search::Search;
use crate::{Node, Position, Table};

pub struct DefaultSearcher<P, T> {
    pub pos: P,
    table: T,
}

impl<P, T> DefaultSearcher<P, T>
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

impl<P, T> Search<P, T> for DefaultSearcher<P, T>
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
    fn look_up_hash(&self, key: &u64) -> (T::U, T::U) {
        self.table.look_up_hash(key)
    }
    fn put_in_hash(&mut self, key: u64, value: (T::U, T::U)) {
        self.table.put_in_hash(key, value)
    }
}
