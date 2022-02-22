use crate::dfpn::dfpn_solver;
use crate::impl_default_hash::DefaultHashPosition;
use crate::impl_hashmap_table::HashMapTable;
use crate::{generate_legal_moves, HashPosition, Node, Table, DFPN, U};
use shogi::{Move, MoveError, Position};

#[derive(Default)]
pub struct Solver<P = DefaultHashPosition, T = HashMapTable> {
    pub pos: P,
    pub table: T,
}

impl<P, T> Solver<P, T>
where
    P: HashPosition,
    T: Table<T = P::T>,
{
    pub fn new(pos: P, table: T) -> Self {
        Self { pos, table }
    }
}

impl<P, T> DFPN<P, T> for Solver<P, T>
where
    P: HashPosition,
    T: Table<T = P::T>,
{
}

impl<P, T> dfpn_solver::Solve<P, T> for Solver<P, T>
where
    P: HashPosition,
    T: Table<T = P::T>,
{
    fn set_position(&mut self, pos: Position) -> P::T {
        self.pos.set_position(pos);
        self.pos.current_hash()
    }
    fn make_move(&mut self, m: Move) -> Result<(), MoveError> {
        self.pos.make_move(m)
    }
    fn unmake_move(&mut self) -> Result<(), MoveError> {
        self.pos.unmake_move()
    }
    fn look_up_hash(&self, key: &T::T) -> (U, U) {
        self.table.look_up_hash(key)
    }
    fn put_in_hash(&mut self, key: T::T, value: (U, U)) {
        self.table.put_in_hash(key, value);
    }
    fn generate_legal_moves(&mut self, node: Node) -> Vec<(Move, P::T)> {
        generate_legal_moves(&mut self.pos, node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::impl_vec_table::VecTable;
    use crate::impl_zobrist_hash::ZobristHashPosition;
    use crate::INF;
    use shogi::bitboard::Factory;

    fn example_position() -> Position {
        let mut pos = Position::new();
        pos.set_sfen("3sks3/9/4S4/9/1+B7/9/9/9/9 b S2rb4g4n4l18p 1")
            .expect("failed to parse SFEN string");
        pos
    }

    fn example_position_reverse() -> Position {
        let mut pos = Position::new();
        pos.set_sfen("9/9/9/9/7+b1/9/4s4/9/3SKS3 w 2RB4G4N4L18Ps 1")
            .expect("failed to parse SFEN string");
        pos
    }

    #[test]
    fn test_impl_default_hashmap() {
        Factory::init();

        let mut solver = Solver::new(DefaultHashPosition::default(), HashMapTable::default());
        assert!(solver.table.is_empty());
        solver.dfpn(example_position());
        assert_eq!(171, solver.table.len());
        assert_eq!(
            (0, INF),
            solver.table.look_up_hash(&solver.pos.current_hash())
        );
    }

    #[test]
    fn test_impl_zobrist_hashmap() {
        Factory::init();

        let mut solver = Solver::new(
            ZobristHashPosition::default(),
            HashMapTable::<u64>::default(),
        );
        assert!(solver.table.is_empty());
        solver.dfpn(example_position());
        assert_eq!(171, solver.table.len());
        assert_eq!(
            (0, INF),
            solver.table.look_up_hash(&solver.pos.current_hash())
        );
    }

    #[test]
    fn test_impl_zobrist_vec() {
        Factory::init();

        let mut solver = Solver::new(ZobristHashPosition::default(), VecTable::new(16));
        assert!(solver.table.is_empty());
        solver.dfpn(example_position());
        assert_eq!(171, solver.table.len());
        assert_eq!(
            (0, INF),
            solver.table.look_up_hash(&solver.pos.current_hash())
        );
    }

    #[test]
    fn test_reverse() {
        Factory::init();

        let mut solver = Solver::<DefaultHashPosition, HashMapTable>::default();
        assert!(solver.table.is_empty());
        solver.dfpn(example_position_reverse());
        assert!(!solver.table.is_empty());
        assert_eq!(
            (0, INF),
            solver.table.look_up_hash(&solver.pos.current_hash())
        );
    }
}
