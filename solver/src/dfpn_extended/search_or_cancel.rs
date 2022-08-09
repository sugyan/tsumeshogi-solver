use dfpn::search::Search;
use dfpn::{Node, Position, Table};
use num_traits::{Bounded, One, SaturatingAdd, Zero};

pub trait SearchOrCancel<P, T>: Search<P, T>
where
    P: Position,
    T: Table,
{
    fn cancel(&mut self) -> bool;

    // ルートでの反復深化
    fn dfpn_search(&mut self) {
        let hash = self.hash_key();
        let (pn, dn) = SearchOrCancel::mid(
            self,
            hash,
            T::U::max_value() - T::U::one(),
            T::U::max_value() - T::U::one(),
            Node::Or,
        );
        if pn != T::U::max_value() && dn != T::U::max_value() {
            SearchOrCancel::mid(self, hash, T::U::max_value(), T::U::max_value(), Node::Or);
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
        while !self.cancel() {
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
            SearchOrCancel::mid(self, h, phi_n_c, delta_n_c, !node);
            self.undo_move(m);
        }
        (T::U::zero(), T::U::zero())
    }
}
