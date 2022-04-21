use crate::CalculateResult;
use dfpn_solver::Node;
use dfpn_solver::Position as _;
use shogi::{Color, Move, MoveError, Piece, PieceType, Position, Square};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub struct ShogiPosition(Position);

impl ShogiPosition {
    fn try_legal_move(&mut self, m: Move, node: Node) -> Result<u64, MoveError> {
        match self.0.make_move(m) {
            Ok(_) => {
                let mut hash = None;
                if node == Node::And || self.0.in_check(self.0.side_to_move()) {
                    hash = Some(self.hash_key());
                }
                self.0.unmake_move().expect("failed to unmake move");
                if let Some(h) = hash {
                    Ok(h)
                } else {
                    Err(MoveError::Inconsistent("Not legal move for tsumeshogi"))
                }
            }
            Err(e) => Err(e),
        }
    }
    fn p8(p: Piece) -> u8 {
        let piece_type = match p.piece_type {
            PieceType::King => 0,
            PieceType::Rook => 1,
            PieceType::Bishop => 2,
            PieceType::Gold => 3,
            PieceType::Silver => 4,
            PieceType::Knight => 5,
            PieceType::Lance => 6,
            PieceType::Pawn => 7,
            PieceType::ProRook => 8,
            PieceType::ProBishop => 9,
            PieceType::ProSilver => 10,
            PieceType::ProKnight => 11,
            PieceType::ProLance => 12,
            PieceType::ProPawn => 13,
        };
        let color = match p.color {
            Color::Black => 0,
            Color::White => 14,
        };
        piece_type + color
    }
}

impl From<&str> for ShogiPosition {
    fn from(sfen: &str) -> Self {
        let mut pos = Position::new();
        pos.set_sfen(sfen).expect("failed to set sfen");
        ShogiPosition(pos)
    }
}

impl dfpn_solver::Position for ShogiPosition {
    type M = Move;

    fn hash_key(&self) -> u64 {
        let mut s = DefaultHasher::new();
        self.hash(&mut s);
        s.finish()
    }
    fn generate_legal_moves(&mut self, node: dfpn_solver::Node) -> Vec<(Self::M, u64)> {
        let mut moves = Vec::new();
        // normal moves
        for from in *self.0.player_bb(self.0.side_to_move()) {
            if let Some(p) = *self.0.piece_at(from) {
                for to in self.0.move_candidates(from, p) {
                    for promote in [true, false] {
                        let m = Move::Normal { from, to, promote };
                        if let Ok(h) = self.try_legal_move(m, node) {
                            moves.push((m, h));
                        }
                    }
                }
            }
        }
        // drop moves
        let target_color = match node {
            Node::Or => self.0.side_to_move().flip(),
            Node::And => self.0.side_to_move(),
        };
        if let Some(king_sq) = self.0.find_king(target_color) {
            match node {
                Node::Or => {
                    for piece_type in PieceType::iter().filter(|pt| pt.is_hand_piece()) {
                        if self.0.hand(Piece {
                            piece_type,
                            color: target_color.flip(),
                        }) == 0
                        {
                            continue;
                        }
                        // 玉をその駒で狙える位置のみ探索
                        for to in self.0.move_candidates(
                            king_sq,
                            Piece {
                                piece_type,
                                color: target_color,
                            },
                        ) {
                            let m = Move::Drop { to, piece_type };
                            if let Ok(h) = self.try_legal_move(m, node) {
                                moves.push((m, h));
                            }
                        }
                    }
                }
                Node::And => {
                    // 玉から飛車角で狙われ得る位置の候補
                    let mut candidates = &self.0.move_candidates(
                        king_sq,
                        Piece {
                            piece_type: PieceType::Rook,
                            color: target_color,
                        },
                    ) | &self.0.move_candidates(
                        king_sq,
                        Piece {
                            piece_type: PieceType::Bishop,
                            color: target_color,
                        },
                    );
                    for piece_type in PieceType::iter().filter(|pt| pt.is_hand_piece()) {
                        if self.0.hand(Piece {
                            piece_type,
                            color: target_color,
                        }) == 0
                        {
                            continue;
                        }
                        for to in candidates {
                            let m = Move::Drop { to, piece_type };
                            match self.try_legal_move(m, node) {
                                Ok(h) => moves.push((m, h)),
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
        moves
    }
    fn do_move(&mut self, m: Self::M) {
        self.0.make_move(m).expect("failed to make move");
    }
    fn undo_move(&mut self, _m: Self::M) {
        self.0.unmake_move().expect("failed to unmake move");
    }
}

impl Hash for ShogiPosition {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Square::iter().for_each(|sq| {
            self.0
                .piece_at(sq)
                .map_or(28, ShogiPosition::p8)
                .hash(state);
        });
        PieceType::iter().for_each(|piece_type| {
            Color::iter().for_each(|color| self.0.hand(Piece { piece_type, color }).hash(state))
        });
        match self.0.side_to_move() {
            Color::Black => 0.hash(state),
            Color::White => 1.hash(state),
        };
    }
}

impl CalculateResult for ShogiPosition {
    fn calculate_result_and_score(&mut self, moves: &[Self::M]) -> (Vec<String>, usize) {
        let (mut ret, mut len) = (Vec::new(), moves.len());
        let mut total_hands = PieceType::iter()
            .filter_map(|piece_type| {
                if piece_type.is_hand_piece() {
                    Some(self.0.hand(Piece {
                        piece_type,
                        color: self.0.side_to_move().flip(),
                    }))
                } else {
                    None
                }
            })
            .sum::<u8>();
        while len > 2 {
            if let (
                Move::Drop {
                    to: drop_to,
                    piece_type: _,
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
                    if let Some(piece_type) = drops[to.index()].take() {
                        if self.0.hand(Piece {
                            piece_type,
                            color: self.0.side_to_move().flip(),
                        }) > 0
                        {
                            // TODO: 候補から除外したいが このパターンだけが候補になる場合もある
                            zero = true;
                        }
                    }
                }
            } else if let Move::Drop { to, piece_type } = m {
                drops[to.index()] = Some(*piece_type);
            }
            ret.push(m.to_string());
        }
        let score = if zero {
            0
        } else {
            len * 100 - total_hands as usize
        };
        (ret, score)
    }
}
