// 「df-pnアルゴリズムの詰将棋を解くプログラムへの応用」
// https://ci.nii.ac.jp/naid/110002726401
pub mod dfpn_solver {
    use crate::{HashPosition, Node, Table, INF, U};
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
