use crate::{Node, Position, Table};
use num_traits::{Bounded, One, SaturatingAdd, Zero};

// 「df-pnアルゴリズムの詰将棋を解くプログラムへの応用」
// https://ci.nii.ac.jp/naid/110002726401
pub trait Search<P, T>
where
    P: Position,
    T: Table,
{
    fn hash_key(&self) -> u64;
    fn generate_legal_moves(&mut self, node: Node) -> Vec<(P::M, u64)>;
    fn do_move(&mut self, m: P::M);
    fn undo_move(&mut self, m: P::M);
    // ハッシュを引く (本当は優越関係が使える)
    fn look_up_hash(&self, key: &u64) -> (T::U, T::U);
    // ハッシュに記録
    fn put_in_hash(&mut self, key: u64, value: (T::U, T::U));

    // ルートでの反復深化
    fn dfpn_search(&mut self) {
        let hash = self.hash_key();
        let (pn, dn) = self.mid(
            hash,
            T::U::max_value() - T::U::one(),
            T::U::max_value() - T::U::one(),
            Node::Or,
        );
        if pn != T::U::max_value() && dn != T::U::max_value() {
            self.mid(hash, T::U::max_value(), T::U::max_value(), Node::Or);
        }
    }
    // ノード n の展開
    fn mid(&mut self, hash: u64, phi: T::U, delta: T::U, node: Node) -> (T::U, T::U) {
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
            self.put_in_hash(hash, (T::U::max_value(), T::U::zero()));
            return match node {
                Node::Or => (T::U::max_value(), T::U::zero()),
                Node::And => (T::U::zero(), T::U::max_value()),
            };
        }
        // 3. ハッシュによるサイクル回避
        self.put_in_hash(hash, (delta, phi));
        // 4. 多重反復深化
        loop {
            // φ か δ がそのしきい値以上なら探索終了
            let sp = self.sum_phi(&children);
            let md = if sp >= T::U::max_value() - T::U::one() {
                T::U::zero()
            } else {
                self.min_delta(&children)
            };
            if phi <= md || delta <= sp {
                self.put_in_hash(hash, (md, sp));
                return match node {
                    Node::Or => (md, sp),
                    Node::And => (sp, md),
                };
            }
            let (best, phi_c, delta_c, delta_2) = self.select_child(&children);
            let phi_n_c = if phi_c == T::U::max_value() - T::U::one() {
                T::U::max_value()
            } else if delta >= T::U::max_value() - T::U::one() {
                T::U::max_value() - T::U::one()
            } else {
                delta + phi_c - sp
            };
            let delta_n_c = if delta_c == T::U::max_value() - T::U::one() {
                T::U::max_value()
            } else {
                phi.min(delta_2.saturating_add(&T::U::one()))
            };
            let (m, h) = best.expect("best move");
            self.do_move(m);
            self.mid(h, phi_n_c, delta_n_c, !node);
            self.undo_move(m);
        }
    }
    #[allow(clippy::type_complexity)]
    // 子ノードの選択
    fn select_child(
        &mut self,
        children: &[(P::M, u64)],
    ) -> (Option<(P::M, u64)>, T::U, T::U, T::U) {
        let (mut delta_c, mut delta_2) = (T::U::max_value(), T::U::max_value());
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
            if p == T::U::max_value() {
                return (best, phi_c.expect("phi_c"), delta_c, delta_2);
            }
        }
        (best, phi_c.expect("phi_c"), delta_c, delta_2)
    }
    // n の子ノード の δ の最小を計算
    fn min_delta(&mut self, children: &[(P::M, u64)]) -> T::U {
        let mut min = T::U::max_value();
        for &(_, h) in children {
            let (_, d) = self.look_up_hash(&h);
            min = min.min(d);
        }
        min
    }
    // nの子ノードのφの和を計算
    fn sum_phi(&mut self, children: &[(P::M, u64)]) -> T::U {
        let mut sum = T::U::zero();
        for &(_, h) in children {
            let (p, _) = self.look_up_hash(&h);
            sum = sum.saturating_add(&p);
        }
        sum
    }
}
