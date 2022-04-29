use dfpn::search::Search;
use dfpn::{Node, Position, Table, INF, U};

pub trait SearchOrCancel<P, T>: Search<P, T>
where
    P: Position,
    T: Table,
{
    fn cancel(&mut self) -> bool;

    // ルートでの反復深化
    fn dfpn_search(&mut self) {
        let hash = self.hash_key();
        let (pn, dn) = SearchOrCancel::mid(self, hash, INF - 1, INF - 1, Node::Or);
        if pn != INF && dn != INF {
            SearchOrCancel::mid(self, hash, INF, INF, Node::Or);
        }
    }
    // ノード n の展開
    fn mid(&mut self, hash: u64, phi: U, delta: U, node: Node) -> (U, U) {
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
        while !self.cancel() {
            // φ か δ がそのしきい値以上なら探索終了
            let sp = self.sum_phi(&children);
            let md = if sp >= INF - 1 {
                0
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
            self.do_move(m);
            SearchOrCancel::mid(self, h, phi_n_c, delta_n_c, !node);
            self.undo_move(m);
        }
        (0, 0)
    }
}
