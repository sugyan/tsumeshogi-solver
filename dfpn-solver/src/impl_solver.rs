use crate::dfpn::dfpn_solver;
use crate::impl_hashmap_table::HashMapTable;
use crate::{generate_legal_moves, Node, Table, DFPN, U};
use yasai::{Move, Position};

#[derive(Default)]
pub struct Solver<T = HashMapTable> {
    pub pos: Position,
    pub table: T,
}

impl<T> Solver<T>
where
    T: Table,
{
    pub fn new(pos: Position, table: T) -> Self {
        Self { pos, table }
    }
}

impl<T> DFPN<T> for Solver<T> where T: Table {}

impl<T> dfpn_solver::Solve<T> for Solver<T>
where
    T: Table,
{
    fn set_position(&mut self, pos: Position) -> u64 {
        self.pos = pos;
        self.pos.key()
    }
    fn do_move(&mut self, m: Move) {
        self.pos.do_move(m);
    }
    fn undo_move(&mut self, m: Move) {
        self.pos.undo_move(m);
    }
    fn look_up_hash(&self, key: &u64) -> (U, U) {
        self.table.look_up_hash(key)
    }
    fn put_in_hash(&mut self, key: u64, value: (U, U)) {
        self.table.put_in_hash(key, value);
    }
    fn generate_legal_moves(&mut self, node: Node) -> Vec<(Move, u64)> {
        generate_legal_moves(&mut self.pos, node)
    }
}
