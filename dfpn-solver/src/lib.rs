pub mod impl_default_hash;
pub mod impl_hashmap_table;
pub mod impl_vec_table;
pub mod impl_zobrist_hash;

use shogi::{Bitboard, Color, Move, MoveError, Piece, PieceType, Square};
use std::hash::Hash;

type U = u32;
pub const INF: U = U::MAX;

pub trait HashPosition {
    type T: Eq + Hash + Copy;
    fn hand(&self, p: Piece) -> u8;
    fn in_check(&self, c: Color) -> bool;
    fn make_move(&mut self, m: Move) -> Result<(), MoveError>;
    fn move_candidates(&self, sq: Square, p: Piece) -> Bitboard;
    fn piece_at(&self, sq: Square) -> &Option<Piece>;
    fn player_bb(&self, c: Color) -> &Bitboard;
    fn side_to_move(&self) -> Color;
    fn unmake_move(&mut self) -> Result<(), MoveError>;
    fn to_hash(&self) -> Self::T;
}

pub trait Table {
    type T;
    fn look_up_hash(&self, key: &Self::T) -> (U, U);
    fn put_in_hash(&mut self, key: Self::T, value: (U, U));
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
}

pub struct Solver<HP, T> {
    pub hp: HP,
    pub t: T,
}

#[derive(Debug, Default)]
struct PD {
    pn: U,
    dn: U,
}

impl<HP, T> Solver<HP, T>
where
    HP: HashPosition,
    T: Table<T = HP::T>,
{
    pub fn new(hp: HP, t: T) -> Self {
        Self { hp, t }
    }
    // 「df-pnアルゴリズムの詰将棋を解くプログラムへの応用」
    // https://ci.nii.ac.jp/naid/110002726401
    pub fn dfpn(&mut self) {
        // ルートでの反復深化
        let mut pd = PD::default();
        self.set_phi(&mut pd, INF - 1);
        self.set_delta(&mut pd, INF - 1);
        self.mid(&mut pd);
        if self.get_phi(&pd) != INF && self.get_delta(&pd) != INF {
            self.set_phi(&mut pd, INF);
            self.set_delta(&mut pd, INF);
            self.mid(&mut pd);
        }
    }
    fn get_phi(&self, pd: &PD) -> U {
        match self.hp.side_to_move() {
            Color::Black => pd.pn,
            Color::White => pd.dn,
        }
    }
    fn get_delta(&self, pd: &PD) -> U {
        match self.hp.side_to_move() {
            Color::Black => pd.dn,
            Color::White => pd.pn,
        }
    }
    fn set_phi(&self, pd: &mut PD, val: U) {
        match self.hp.side_to_move() {
            Color::Black => pd.pn = val,
            Color::White => pd.dn = val,
        }
    }
    fn set_delta(&self, pd: &mut PD, val: U) {
        match self.hp.side_to_move() {
            Color::Black => pd.dn = val,
            Color::White => pd.pn = val,
        }
    }
    // ノードの展開
    fn mid(&mut self, pd: &mut PD) {
        // 1. ハッシュを引く
        let (p, d) = self.look_up_hash(&self.hp.to_hash());
        if self.get_phi(pd) <= p || self.get_delta(pd) <= d {
            self.set_phi(pd, p);
            self.set_delta(pd, d);
            return;
        }
        // 2. 合法手の生成
        let children = generate_legal_moves(&mut self.hp);
        if children.is_empty() {
            // ?
            self.set_phi(pd, INF);
            self.set_delta(pd, 0);
            self.put_in_hash((self.get_phi(pd), self.get_delta(pd)));
            return;
        }
        // 3. ハッシュによるサイクル回避
        self.put_in_hash((self.get_phi(pd), self.get_delta(pd)));
        // 4. 多重反復深化
        loop {
            // φ か δ がそのしきい値以上なら探索終了
            let md = self.min_delta(&children);
            let sp = self.sum_phi(&children);
            if self.get_phi(pd) <= md || self.get_delta(pd) <= sp {
                self.set_phi(pd, md);
                self.set_delta(pd, sp);
                self.put_in_hash((self.get_phi(pd), self.get_delta(pd)));
                return;
            }
            let (best, phi_c, delta_c, delta_2) = self.select_child(&children);
            let phi_n_c = if phi_c == INF - 1 {
                INF
            } else if self.get_delta(pd) >= INF - 1 {
                INF - 1
            } else {
                self.get_delta(pd) + phi_c - sp
            };
            let delta_n_c = if delta_c == INF - 1 {
                INF
            } else {
                (self.get_phi(pd)).min(delta_2.saturating_add(1))
            };
            let m = best.expect("best move");
            self.hp.make_move(m).expect("failed to make move");
            let mut pd_c = PD::default();
            self.set_phi(&mut pd_c, phi_n_c);
            self.set_delta(&mut pd_c, delta_n_c);
            self.mid(&mut pd_c);
            self.hp.unmake_move().expect("failed to unmake move");
        }
    }
    // 子ノードの選択
    fn select_child(&mut self, children: &[(Move, HP::T)]) -> (Option<Move>, U, U, U) {
        let (mut delta_c, mut delta_2) = (INF, INF);
        let mut best = None;
        let mut phi_c = None; // not optional?
        for &(m, h) in children {
            let (p, d) = self.look_up_hash(&h);
            if d < delta_c {
                best = Some(m);
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
    fn look_up_hash(&self, key: &HP::T) -> (U, U) {
        self.t.look_up_hash(key)
    }
    // ハッシュに記録
    fn put_in_hash(&mut self, value: (U, U)) {
        self.t.put_in_hash(self.hp.to_hash(), value);
    }
    // n の子ノード の δ の最小を計算
    fn min_delta(&mut self, children: &[(Move, HP::T)]) -> U {
        let mut min = INF;
        for &(_, h) in children {
            let (_, d) = self.look_up_hash(&h);
            min = min.min(d);
        }
        min
    }
    // nの子ノードのφの和を計算
    fn sum_phi(&mut self, children: &[(Move, HP::T)]) -> U {
        let mut sum: U = 0;
        for &(_, h) in children {
            let (p, _) = self.look_up_hash(&h);
            sum = sum.saturating_add(p);
        }
        sum
    }
}

pub fn generate_legal_moves<HP>(pos: &mut HP) -> Vec<(Move, HP::T)>
where
    HP: HashPosition,
{
    let color = pos.side_to_move();
    let &bb = pos.player_bb(color);
    let mut children = Vec::new();
    // normal moves
    for from in bb {
        if let Some(p) = pos.piece_at(from) {
            for to in pos.move_candidates(from, *p) {
                for promote in [true, false] {
                    let m = Move::Normal { from, to, promote };
                    if pos.make_move(m).is_ok() {
                        if color == Color::White || pos.in_check(Color::White) {
                            children.push((m, pos.to_hash()));
                        }
                        pos.unmake_move().expect("failed to unmake move");
                    }
                }
            }
        }
    }
    // drop moves
    if let Some(king_sq) = pos.player_bb(Color::White).into_iter().find(|sq| {
        // want to use pos.find_king()...
        pos.piece_at(*sq)
            == &Some(Piece {
                piece_type: PieceType::King,
                color: Color::White,
            })
    }) {
        match color {
            Color::Black => {
                for piece_type in PieceType::iter().filter(|pt| pt.is_hand_piece()) {
                    if pos.hand(Piece { piece_type, color }) == 0 {
                        continue;
                    }
                    // 玉をその駒で狙える位置のみ探索
                    for to in pos.move_candidates(
                        king_sq,
                        Piece {
                            piece_type,
                            color: Color::White,
                        },
                    ) {
                        let m = Move::Drop { to, piece_type };
                        if pos.make_move(m).is_ok() {
                            if color == Color::White || pos.in_check(Color::White) {
                                children.push((m, pos.to_hash()));
                            }
                            pos.unmake_move().expect("failed to unmake move");
                        }
                    }
                }
            }
            Color::White => {
                // 玉から飛車角で狙われ得る位置の候補
                let mut candidates = &pos.move_candidates(
                    king_sq,
                    Piece {
                        piece_type: PieceType::Rook,
                        color: Color::White,
                    },
                ) | &pos.move_candidates(
                    king_sq,
                    Piece {
                        piece_type: PieceType::Bishop,
                        color: Color::White,
                    },
                );
                for piece_type in PieceType::iter().filter(|pt| pt.is_hand_piece()) {
                    if pos.hand(Piece { piece_type, color }) == 0 {
                        continue;
                    }
                    for to in candidates {
                        let m = Move::Drop { to, piece_type };
                        match pos.make_move(m) {
                            Ok(_) => {
                                if color == Color::White || pos.in_check(Color::White) {
                                    children.push((m, pos.to_hash()));
                                }
                                pos.unmake_move().expect("failed to unmake move");
                            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::impl_default_hash::DefaultHashPosition;
    use crate::impl_hashmap_table::HashMapTable;
    use crate::impl_vec_table::VecTable;
    use crate::impl_zobrist_hash::ZobristHashPosition;
    use shogi::bitboard::Factory;
    use shogi::Position;

    #[test]
    fn test_impl() {
        Factory::init();
        let sfen = "3sks3/9/4S4/9/1+B7/9/9/9/9 b S2rb4g4n4l18p 1";

        // default + hashmap
        {
            let mut pos = Position::new();
            pos.set_sfen(sfen).expect("failed to parse SFEN string");
            let mut solver = Solver::new(DefaultHashPosition::new(pos), HashMapTable::new());
            assert!(solver.t.is_empty());
            solver.dfpn();
            assert_eq!(171, solver.t.len());
        }
        // zobrist + hashmap
        {
            let mut pos = Position::new();
            pos.set_sfen(sfen).expect("failed to parse SFEN string");
            let mut solver = Solver::new(ZobristHashPosition::<u64>::new(pos), HashMapTable::new());
            assert!(solver.t.is_empty());
            solver.dfpn();
            assert_eq!(171, solver.t.len());
        }
        // zobrist + vec
        {
            let mut pos = Position::new();
            pos.set_sfen(sfen).expect("failed to parse SFEN string");
            let mut solver = Solver::new(ZobristHashPosition::new(pos), VecTable::new(16));
            assert!(solver.t.is_empty());
            solver.dfpn();
            assert_eq!(171, solver.t.len());
        }
    }
}
