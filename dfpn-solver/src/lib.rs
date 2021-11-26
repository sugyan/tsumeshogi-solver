pub mod impl_default_hash;
pub mod impl_hashmap_table;
pub mod impl_vec_table;
pub mod impl_zobrist_hash;

use impl_default_hash::DefaultHashPosition;
use impl_hashmap_table::HashMapTable;
use shogi::{Bitboard, Color, Move, MoveError, Piece, PieceType, Position, Square};
use std::{fmt::Debug, hash::Hash};

type U = u32;
pub const INF: U = U::MAX;

#[derive(Clone, Copy, PartialEq)]
pub enum Node {
    Or,
    And,
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
    // ハッシュを引く (本当は優越関係が使える)
    fn look_up_hash(&self, key: &Self::T) -> (U, U);
    // ハッシュに記録
    fn put_in_hash(&mut self, key: Self::T, value: (U, U));

    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
}

#[derive(Default)]
pub struct Solver<P = DefaultHashPosition, T = HashMapTable> {
    pub pos: P,
    pub table: T,
    base_ply: u16,
}

impl<P, T> Solver<P, T>
where
    P: HashPosition,
    T: Table<T = P::T>,
{
    pub fn new(pos: P, table: T) -> Self {
        Self {
            pos,
            table,
            base_ply: 0,
        }
    }
    // 「df-pnアルゴリズムの詰将棋を解くプログラムへの応用」
    // https://ci.nii.ac.jp/naid/110002726401
    pub fn dfpn(&mut self, pos: Position) {
        self.base_ply = pos.ply();
        self.pos.set_position(pos);
        let hash = self.pos.current_hash();
        // ルートでの反復深化
        let (pn, dn) = self.mid(hash, &(INF - 1, INF - 1));
        if pn != INF && dn != INF {
            self.mid(hash, &(INF, INF));
        }
    }
    fn node(&self) -> Node {
        if (self.pos.ply() ^ self.base_ply) & 1 == 0 {
            Node::Or
        } else {
            Node::And
        }
    }
    fn phi(&self, pd: &(U, U)) -> U {
        match self.node() {
            Node::Or => pd.0,
            Node::And => pd.1,
        }
    }
    fn delta(&self, pd: &(U, U)) -> U {
        match self.node() {
            Node::Or => pd.1,
            Node::And => pd.0,
        }
    }
    // ノードの展開
    fn mid(&mut self, hash: P::T, pd: &(U, U)) -> (U, U) {
        // 1. ハッシュを引く
        let (p, d) = self.table.look_up_hash(&hash);
        if self.phi(pd) <= p || self.delta(pd) <= d {
            return match self.node() {
                Node::Or => (p, d),
                Node::And => (d, p),
            };
        }
        // 2. 合法手の生成
        let node = self.node();
        let children = generate_legal_moves(&mut self.pos, node);
        if children.is_empty() {
            // ?
            self.table.put_in_hash(hash, (INF, 0));
            return match self.node() {
                Node::Or => (INF, 0),
                Node::And => (0, INF),
            };
        }
        // 3. ハッシュによるサイクル回避
        match self.node() {
            Node::Or => self.table.put_in_hash(hash, (pd.1, pd.0)),
            Node::And => self.table.put_in_hash(hash, (pd.0, pd.1)),
        };
        // 4. 多重反復深化
        loop {
            // φ か δ がそのしきい値以上なら探索終了
            let md = self.min_delta(&children);
            let sp = self.sum_phi(&children);
            if self.phi(pd) <= md || self.delta(pd) <= sp {
                self.table.put_in_hash(hash, (md, sp));
                return match self.node() {
                    Node::Or => (md, sp),
                    Node::And => (sp, md),
                };
            }
            let (best, phi_c, delta_c, delta_2) = self.select_child(&children);
            let phi_n_c = if phi_c == INF - 1 {
                INF
            } else if self.delta(pd) >= INF - 1 {
                INF - 1
            } else {
                self.delta(pd) + phi_c - sp
            };
            let delta_n_c = if delta_c == INF - 1 {
                INF
            } else {
                (self.phi(pd)).min(delta_2.saturating_add(1))
            };
            let (m, h) = best.expect("best move");
            self.pos.make_move(m).expect("failed to make move");
            match self.node() {
                Node::Or => self.mid(h, &(phi_n_c, delta_n_c)),
                Node::And => self.mid(h, &(delta_n_c, phi_n_c)),
            };
            self.pos.unmake_move().expect("failed to unmake move");
        }
    }
    // 子ノードの選択
    fn select_child(&mut self, children: &[(Move, P::T)]) -> (Option<(Move, P::T)>, U, U, U) {
        let (mut delta_c, mut delta_2) = (INF, INF);
        let mut best = None;
        let mut phi_c = None; // not optional?
        for &(m, h) in children {
            let (p, d) = self.table.look_up_hash(&h);
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
    // n の子ノード の δ の最小を計算
    fn min_delta(&mut self, children: &[(Move, P::T)]) -> U {
        let mut min = INF;
        for &(_, h) in children {
            let (_, d) = self.table.look_up_hash(&h);
            min = min.min(d);
        }
        min
    }
    // nの子ノードのφの和を計算
    fn sum_phi(&mut self, children: &[(Move, P::T)]) -> U {
        let mut sum: U = 0;
        for &(_, h) in children {
            let (p, _) = self.table.look_up_hash(&h);
            sum = sum.saturating_add(p);
        }
        sum
    }
}

pub fn generate_legal_moves<P>(pos: &mut P, node: Node) -> Vec<(Move, P::T)>
where
    P: HashPosition,
{
    let mut children = Vec::new();
    // normal moves
    for from in *pos.player_bb(pos.side_to_move()) {
        if let Some(p) = *pos.piece_at(from) {
            for to in pos.move_candidates(from, p) {
                for promote in [true, false] {
                    let m = Move::Normal { from, to, promote };
                    if let Ok(h) = try_legal_move(pos, m, node) {
                        children.push((m, h));
                    }
                }
            }
        }
    }
    // drop moves
    let target_color = match node {
        Node::Or => pos.side_to_move().flip(),
        Node::And => pos.side_to_move(),
    };
    if let Some(king_sq) = pos.find_king(target_color) {
        match node {
            Node::Or => {
                for piece_type in PieceType::iter().filter(|pt| pt.is_hand_piece()) {
                    if pos.hand(Piece {
                        piece_type,
                        color: target_color.flip(),
                    }) == 0
                    {
                        continue;
                    }
                    // 玉をその駒で狙える位置のみ探索
                    for to in pos.move_candidates(
                        king_sq,
                        Piece {
                            piece_type,
                            color: target_color,
                        },
                    ) {
                        let m = Move::Drop { to, piece_type };
                        if let Ok(h) = try_legal_move(pos, m, node) {
                            children.push((m, h));
                        }
                    }
                }
            }
            Node::And => {
                // 玉から飛車角で狙われ得る位置の候補
                let mut candidates = &pos.move_candidates(
                    king_sq,
                    Piece {
                        piece_type: PieceType::Rook,
                        color: target_color,
                    },
                ) | &pos.move_candidates(
                    king_sq,
                    Piece {
                        piece_type: PieceType::Bishop,
                        color: target_color,
                    },
                );
                for piece_type in PieceType::iter().filter(|pt| pt.is_hand_piece()) {
                    if pos.hand(Piece {
                        piece_type,
                        color: target_color,
                    }) == 0
                    {
                        continue;
                    }
                    for to in candidates {
                        let m = Move::Drop { to, piece_type };
                        match try_legal_move(pos, m, node) {
                            Ok(h) => children.push((m, h)),
                            Err(MoveError::InCheck) => {
                                // 合駒として機能しない位置は候補から外す
                                candidates.clear_at(to);
                            }
                            Err(_) => {
                                // ignore
                            }
                        }
                    }
                }
            }
        }
    }
    children
}

fn try_legal_move<P>(pos: &mut P, m: Move, node: Node) -> Result<P::T, MoveError>
where
    P: HashPosition,
{
    match pos.make_move(m) {
        Ok(_) => {
            let mut hash = None;
            if node == Node::And || pos.in_check(pos.side_to_move()) {
                hash = Some(pos.current_hash());
            }
            pos.unmake_move().expect("failed to unmake move");
            if let Some(h) = hash {
                Ok(h)
            } else {
                Err(MoveError::Inconsistent("Not legal move for tsumeshogi"))
            }
        }
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::impl_default_hash::DefaultHashPosition;
    use crate::impl_hashmap_table::HashMapTable;
    use crate::impl_vec_table::VecTable;
    use crate::impl_zobrist_hash::ZobristHashPosition;
    use shogi::bitboard::Factory;
    use shogi::Position;

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
