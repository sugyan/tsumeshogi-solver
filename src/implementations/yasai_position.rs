use crate::solver::CalculateResult;
use dfpn::Node;
use shogi_core::{Hand, Move, PartialPosition, ToUsi};
use yasai::Position;

pub struct YasaiPosition(Position);

impl From<PartialPosition> for YasaiPosition {
    fn from(pos: PartialPosition) -> Self {
        Self(Position::new(pos))
    }
}

impl dfpn::Position for YasaiPosition {
    fn hash_key(&self) -> u64 {
        self.0.key()
    }
    fn generate_legal_moves(&mut self, node: Node) -> Vec<(Move, u64)> {
        let mut moves = Vec::new();
        for m in self.0.legal_moves() {
            if node == Node::And || self.0.is_check_move(m) {
                self.0.do_move(m);
                moves.push((m, self.0.key()));
                self.0.undo_move(m);
            }
        }
        moves
    }
    fn do_move(&mut self, m: Move) {
        self.0.do_move(m);
    }
    fn undo_move(&mut self, m: Move) {
        self.0.undo_move(m);
    }
}

impl CalculateResult for YasaiPosition {
    fn calculate_result_and_score(&self, moves: &[Move]) -> (Vec<String>, usize) {
        let (mut ret, mut len) = (Vec::new(), moves.len());
        let mut total_hands = Hand::all_hand_pieces()
            .filter_map(|pk| self.0.hand(self.0.side_to_move().flip()).count(pk))
            .sum::<u8>();
        // 最終2手が「合駒→同」の場合は、合駒無効の詰みなので削除
        while len > 2 {
            if let (
                Move::Drop {
                    to: drop_to,
                    piece: _,
                },
                Move::Normal {
                    from: _,
                    to: move_to,
                    promote: _,
                },
            ) = (moves[len - 2], moves[len - 1])
            {
                if drop_to == move_to {
                    len -= 2;
                    total_hands -= 1;
                    continue;
                }
            }
            break;
        }
        // 1. 玉方が合駒として打った駒が後に取られて
        // 2. 最終的に攻方の持駒に入っている
        // を満たす場合、無駄合駒とみなす
        let mut drops = vec![None; 81];
        let mut zero = false;
        for (i, m) in moves.iter().enumerate().take(len) {
            if i % 2 == 0 {
                if let Move::Normal {
                    from: _,
                    to,
                    promote: _,
                } = m
                {
                    if let Some(piece_type) = drops[to.array_index()].take() {
                        if self
                            .0
                            .hand(self.0.side_to_move().flip())
                            .count(piece_type)
                            .unwrap_or_default()
                            > 0
                        {
                            // TODO: 候補から除外したいが このパターンだけが候補になる場合もある
                            zero = true;
                        }
                    }
                }
            } else if let Move::Drop { to, piece } = m {
                drops[to.array_index()] = Some(piece.piece_kind());
            }
            ret.push(m.to_usi_owned());
        }
        let score = if zero {
            0
        } else {
            len * 100 - total_hands as usize
        };
        (ret, score)
    }
}
