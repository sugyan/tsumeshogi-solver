use crate::dfpn::dfpn_solver;
use crate::impl_default_hash::DefaultHashPosition;
use crate::impl_hashmap_table::HashMapTable;
use crate::{generate_legal_moves, HashPosition, Node, Table, DFPN, INF, U};
use shogi::{Move, MoveError, Position};

#[derive(Default)]
pub struct ExtendedSolver<P = DefaultHashPosition, T = HashMapTable> {
    pub pos: P,
    pub table: T,
}

impl<P, T> ExtendedSolver<P, T>
where
    P: HashPosition,
    T: Table<T = P::T>,
{
    pub fn new(pos: P, table: T) -> Self {
        Self { pos, table }
    }
}

impl<P, T> DFPN<P, T> for ExtendedSolver<P, T>
where
    P: HashPosition,
    T: Table<T = P::T>,
{
}

impl<P, T> dfpn_solver::Solve<P, T> for ExtendedSolver<P, T>
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
    fn mid(&mut self, hash: P::T, phi: U, delta: U, node: Node) -> (U, U) {
        // 1. ハッシュを引く
        let (p, d) = self.look_up_hash(&hash);
        if phi <= p || delta <= d {
            return match node {
                Node::Or => (p, d),
                Node::And => (d, p),
            };
        }
        // 2. 合法手の生成
        let children = self.generate_legal_moves(node);
        // 攻方の場合のみ 1手詰を探索する
        if node == Node::Or {
            let mut mate = false;
            for &(m, h) in &children {
                self.make_move(m).expect("failed to make move");
                let len = self.generate_legal_moves(Node::And).len();
                self.unmake_move().expect("failed to unmake move");
                if len == 0 {
                    self.put_in_hash(h, (INF, 0));
                    mate = true;
                } else {
                    self.put_in_hash(h, (1, len as U));
                }
            }
            if mate {
                self.put_in_hash(hash, (0, INF));
                return (0, INF);
            }
        }
        if children.is_empty() {
            // ?
            self.put_in_hash(hash, (INF, 0));
            return match node {
                Node::Or => (INF, 0),
                Node::And => (0, INF),
            };
        }
        // 3. ハッシュによるサイクル回避
        self.put_in_hash(hash, (delta, phi));
        // 4. 多重反復深化
        loop {
            // φ か δ がそのしきい値以上なら探索終了
            let md = self.min_delta(&children);
            let sp = self.sum_phi(&children);
            if phi <= md || delta <= sp {
                self.put_in_hash(hash, (md, sp));
                return match node {
                    Node::Or => (md, sp),
                    Node::And => (sp, md),
                };
            }
            let (best, phi_c, delta_c, delta_2) = self.select_child(&children);
            let phi_n_c = if phi_c == INF - 1 {
                INF
            } else if delta >= INF - 1 {
                INF - 1
            } else {
                delta + phi_c - sp
            };
            let delta_n_c = if delta_c == INF - 1 {
                INF
            } else {
                phi.min(delta_2.saturating_add(1))
            };
            let (m, h) = best.expect("best move");
            self.make_move(m).expect("failed to make move");
            self.mid(h, phi_n_c, delta_n_c, node.flip());
            self.unmake_move().expect("failed to unmake move");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::impl_vec_table::VecTable;
    use crate::impl_zobrist_hash::ZobristHashPosition;
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

        let mut solver =
            ExtendedSolver::new(DefaultHashPosition::default(), HashMapTable::default());
        assert!(solver.table.is_empty());
        solver.dfpn(example_position());
        assert_eq!(194, solver.table.len());
        assert_eq!(
            (0, INF),
            solver.table.look_up_hash(&solver.pos.current_hash())
        );
    }

    #[test]
    fn test_impl_zobrist_hashmap() {
        Factory::init();

        let mut solver = ExtendedSolver::new(
            ZobristHashPosition::default(),
            HashMapTable::<u64>::default(),
        );
        assert!(solver.table.is_empty());
        solver.dfpn(example_position());
        assert_eq!(194, solver.table.len());
        assert_eq!(
            (0, INF),
            solver.table.look_up_hash(&solver.pos.current_hash())
        );
    }

    #[test]
    fn test_impl_zobrist_vec() {
        Factory::init();

        let mut solver = ExtendedSolver::new(ZobristHashPosition::default(), VecTable::new(16));
        assert!(solver.table.is_empty());
        solver.dfpn(example_position());
        assert_eq!(194, solver.table.len());
        assert_eq!(
            (0, INF),
            solver.table.look_up_hash(&solver.pos.current_hash())
        );
    }

    #[test]
    fn test_reverse() {
        Factory::init();

        let mut solver: ExtendedSolver = ExtendedSolver::default();
        assert!(solver.table.is_empty());
        solver.dfpn(example_position_reverse());
        assert!(!solver.table.is_empty());
        assert_eq!(
            (0, INF),
            solver.table.look_up_hash(&solver.pos.current_hash())
        );
    }
}
