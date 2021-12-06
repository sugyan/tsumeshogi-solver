// 「df-pnアルゴリズムの詰将棋を解くプログラムへの応用」
// https://ci.nii.ac.jp/naid/110002726401
use crate::impl_default_hash::DefaultHashPosition;
use crate::impl_hashmap_table::HashMapTable;
use crate::{generate_legal_moves, INF, U};
use shogi::{Bitboard, Color, Move, MoveError, Piece, Position, Square};
use std::{fmt::Debug, hash::Hash};

#[derive(Clone, Copy, PartialEq)]
pub enum Node {
    Or,
    And,
}

impl Node {
    pub fn flip(self) -> Self {
        match self {
            Node::Or => Node::And,
            Node::And => Node::Or,
        }
    }
}

pub trait HashPosition: Default {
    type T: Eq + Hash + Copy + Debug;
    fn find_king(&self, c: Color) -> Option<Square>;
    fn hand(&self, p: Piece) -> u8;
    fn in_check(&self, color: Color) -> bool;
    fn make_move(&mut self, m: Move) -> Result<(), MoveError>;
    fn move_candidates(&self, sq: Square, p: Piece) -> Bitboard;
    fn piece_at(&self, sq: Square) -> &Option<Piece>;
    fn player_bb(&self, c: Color) -> &Bitboard;
    fn ply(&self) -> u16;
    fn side_to_move(&self) -> Color;
    fn unmake_move(&mut self) -> Result<(), MoveError>;

    fn set_position(&mut self, pos: Position);
    fn current_hash(&self) -> Self::T;
}

pub trait Table: Default {
    type T;
    fn look_up_hash(&self, key: &Self::T) -> (U, U);
    fn put_in_hash(&mut self, key: Self::T, value: (U, U));

    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
}

mod dfpn_solver {
    use super::{HashPosition, Node, Table, INF, U};
    use shogi::{Move, MoveError, Position};

    pub trait Solve<P, T>
    where
        P: HashPosition,
        T: Table<T = P::T>,
    {
        fn set_position(&mut self, pos: Position) -> P::T;
        fn make_move(&mut self, m: Move) -> Result<(), MoveError>;
        fn unmake_move(&mut self) -> Result<(), MoveError>;
        fn generate_legal_moves(&mut self, node: Node) -> Vec<(Move, P::T)>;
        // ノード n の展開
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
        // 子ノードの選択
        fn select_child(&mut self, children: &[(Move, P::T)]) -> (Option<(Move, P::T)>, U, U, U) {
            let (mut delta_c, mut delta_2) = (INF, INF);
            let mut best = None;
            let mut phi_c = None; // not optional?
            for &(m, h) in children {
                let (p, d) = self.look_up_hash(&h);
                if d < delta_c {
                    best = Some((m, h));
                    delta_2 = delta_c;
                    phi_c = Some(p);
                    delta_c = d;
                } else if d < delta_2 {
                    delta_2 = d;
                }
                if p == INF {
                    return (best, phi_c.expect("phi_c"), delta_c, delta_2);
                }
            }
            (best, phi_c.expect("phi_c"), delta_c, delta_2)
        }
        // ハッシュを引く (本当は優越関係が使える)
        fn look_up_hash(&self, key: &T::T) -> (U, U);
        // ハッシュに記録
        fn put_in_hash(&mut self, key: T::T, value: (U, U));
        // n の子ノード の δ の最小を計算
        fn min_delta(&mut self, children: &[(Move, P::T)]) -> U {
            let mut min = INF;
            for &(_, h) in children {
                let (_, d) = self.look_up_hash(&h);
                min = min.min(d);
            }
            min
        }
        // nの子ノードのφの和を計算
        fn sum_phi(&mut self, children: &[(Move, P::T)]) -> U {
            let mut sum: U = 0;
            for &(_, h) in children {
                let (p, _) = self.look_up_hash(&h);
                sum = sum.saturating_add(p);
            }
            sum
        }
    }
}

pub trait DFPN<P, T>: dfpn_solver::Solve<P, T>
where
    P: HashPosition,
    T: Table<T = P::T>,
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

        let mut solver: Solver = Solver::default();
        assert!(solver.table.is_empty());
        solver.dfpn(example_position_reverse());
        assert!(!solver.table.is_empty());
        assert_eq!(
            (0, INF),
            solver.table.look_up_hash(&solver.pos.current_hash())
        );
    }
}
