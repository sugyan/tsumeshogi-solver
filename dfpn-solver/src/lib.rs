mod impl_naive_position;

use impl_naive_position::NaiveHashPosition;
use shogi::bitboard::Factory;
use shogi::{Bitboard, Color, Move, MoveError, Piece, PieceType, Position, Square};
use std::collections::HashMap;

type Num = u16;
const MAX: Num = Num::MAX;

pub fn solve(pos: &Position) {
    Factory::init();

    let mut solver = Solver::<NaiveHashPosition>::new(pos.into());
    solver.solve();
}

trait HashablePosition {
    fn hand(&self, p: Piece) -> u8;
    fn in_check(&self, c: Color) -> bool;
    fn make_move(&mut self, m: Move) -> Result<(), MoveError>;
    fn move_candidates(&self, sq: Square, p: Piece) -> Bitboard;
    fn piece_at(&self, sq: Square) -> &Option<Piece>;
    fn player_bb(&self, c: Color) -> &Bitboard;
    fn side_to_move(&self) -> Color;
    fn unmake_move(&mut self) -> Result<(), MoveError>;
    fn to_hash(&self) -> u64;
}

struct Solver<T> {
    pos: T,
    table: HashMap<u64, (Num, Num)>,
}

#[derive(Debug, Default)]
struct PD {
    pn: Num,
    dn: Num,
}

impl<T> Solver<T>
where
    T: HashablePosition,
{
    fn new(pos: T) -> Self {
        Self {
            pos,
            table: HashMap::new(),
        }
    }
    fn solve(&mut self) {
        // ルートでの反復深化
        let mut pd = PD::default();
        self.set_phi(&mut pd, MAX - 1);
        self.set_delta(&mut pd, MAX - 1);
        self.mid(&mut pd);
        if self.get_phi(&pd) != MAX && self.get_delta(&pd) != MAX {
            self.set_phi(&mut pd, MAX);
            self.set_delta(&mut pd, MAX);
            self.mid(&mut pd);
        }
    }
    fn get_phi(&self, pd: &PD) -> Num {
        match self.pos.side_to_move() {
            Color::Black => pd.pn,
            Color::White => pd.dn,
        }
    }
    fn get_delta(&self, pd: &PD) -> Num {
        match self.pos.side_to_move() {
            Color::Black => pd.dn,
            Color::White => pd.pn,
        }
    }
    fn set_phi(&self, pd: &mut PD, val: Num) {
        match self.pos.side_to_move() {
            Color::Black => pd.pn = val,
            Color::White => pd.dn = val,
        }
    }
    fn set_delta(&self, pd: &mut PD, val: Num) {
        match self.pos.side_to_move() {
            Color::Black => pd.dn = val,
            Color::White => pd.pn = val,
        }
    }
    // ノードの展開
    fn mid(&mut self, pd: &mut PD) {
        // 1. ハッシュを引く
        let (p, d) = self.look_up_hash(&self.pos.to_hash());
        if self.get_phi(pd) < p || self.get_delta(pd) < d {
            self.set_phi(pd, p);
            self.set_delta(pd, d);
            return;
        }
        // 2. 合法手の生成
        let children = generate_legal_moves(&mut self.pos);
        if children.is_empty() {
            // ?
            self.set_phi(pd, MAX);
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
            if self.get_phi(pd) < md || self.get_delta(pd) < sp {
                self.set_phi(pd, md);
                self.set_delta(pd, sp);
                self.put_in_hash((self.get_phi(pd), self.get_delta(pd)));
                return;
            }
            let (best, phi_c, delta_c, delta_2) = self.select_child(&children);
            let phi_n_c = if phi_c == MAX - 1 {
                MAX
            } else if self.get_delta(pd) >= MAX - 1 {
                MAX - 1
            } else {
                self.get_delta(pd) + phi_c - sp
            };
            let delta_n_c = if delta_c == MAX - 1 {
                MAX
            } else {
                (self.get_phi(pd)).min(delta_2.saturating_add(1))
            };
            let m = best.expect("best move");
            self.pos.make_move(m).expect("failed to make move");
            let mut pd_c = PD::default();
            self.set_phi(&mut pd_c, phi_n_c);
            self.set_delta(&mut pd_c, delta_n_c);
            self.mid(&mut pd_c);
            self.pos.unmake_move().expect("failed to unmake move");
        }
    }
    // 子ノードの選択
    fn select_child(&mut self, children: &[(Move, u64)]) -> (Option<Move>, Num, Num, Num) {
        let (mut delta_c, mut delta_2) = (MAX, MAX);
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
            if p == MAX {
                return (best, phi_c.expect("phi_c"), delta_c, delta_2);
            }
        }
        (best, phi_c.expect("phi_c"), delta_c, delta_2)
    }
    // ハッシュを引く (本当は優越関係が使える)
    fn look_up_hash(&self, key: &u64) -> (Num, Num) {
        *self.table.get(key).unwrap_or(&(1, 1))
    }
    // ハッシュに記録
    fn put_in_hash(&mut self, value: (Num, Num)) {
        self.table.insert(self.pos.to_hash(), value);
    }
    // n の子ノード の δ の最小を計算
    fn min_delta(&mut self, children: &[(Move, u64)]) -> Num {
        let mut min = MAX;
        for &(_, h) in children {
            let (_, d) = self.look_up_hash(&h);
            min = min.min(d);
        }
        min
    }
    // nの子ノードのφの和を計算
    fn sum_phi(&mut self, children: &[(Move, u64)]) -> Num {
        let mut sum: Num = 0;
        for &(_, h) in children {
            let (p, _) = self.look_up_hash(&h);
            sum = sum.saturating_add(p);
        }
        sum
    }
}

fn generate_legal_moves<T>(pos: &mut T) -> Vec<(Move, u64)>
where
    T: HashablePosition,
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
                    match pos.make_move(m) {
                        Ok(()) => {
                            if color == Color::White || pos.in_check(Color::White) {
                                children.push((m, pos.to_hash()));
                            }
                            pos.unmake_move().expect("failed to unmake move");
                        }
                        Err(_) => {
                            // ignore
                        }
                    }
                }
            }
        }
    }
    // drop moves
    for piece_type in PieceType::iter().filter(|p| p.is_hand_piece()) {
        if pos.hand(Piece { piece_type, color }) == 0 {
            continue;
        }
        for to in Square::iter() {
            let m = Move::Drop { to, piece_type };
            match pos.make_move(m) {
                Ok(_) => {
                    if color == Color::White || pos.in_check(Color::White) {
                        children.push((m, pos.to_hash()));
                    }
                    pos.unmake_move().expect("failed to unmake move");
                }
                Err(_) => {
                    // ignore
                }
            }
        }
    }
    children
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
