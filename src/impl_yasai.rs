use crate::CalculateResult;
use dfpn_solver::Node;
use yasai::{Move, MoveType, PieceType, Position};

pub struct YasaiPosition(Position);

impl From<&str> for YasaiPosition {
    fn from(sfen: &str) -> Self {
        let mut pos = shogi::Position::new();
        pos.set_sfen(sfen).expect("failed to set sfen");

        let board = yasai::Square::ALL.map(|sq| {
            pos.piece_at(shogi::Square::from_index(sq.index() as u8).unwrap())
                .map(|p| {
                    let color = match p.color {
                        shogi::Color::Black => yasai::Color::Black,
                        shogi::Color::White => yasai::Color::White,
                    };
                    let piece_type = match p.piece_type {
                        shogi::PieceType::King => yasai::PieceType::OU,
                        shogi::PieceType::Rook => yasai::PieceType::HI,
                        shogi::PieceType::Bishop => yasai::PieceType::KA,
                        shogi::PieceType::Gold => yasai::PieceType::KI,
                        shogi::PieceType::Silver => yasai::PieceType::GI,
                        shogi::PieceType::Knight => yasai::PieceType::KE,
                        shogi::PieceType::Lance => yasai::PieceType::KY,
                        shogi::PieceType::Pawn => yasai::PieceType::FU,
                        shogi::PieceType::ProRook => yasai::PieceType::RY,
                        shogi::PieceType::ProBishop => yasai::PieceType::UM,
                        shogi::PieceType::ProSilver => yasai::PieceType::NG,
                        shogi::PieceType::ProKnight => yasai::PieceType::NK,
                        shogi::PieceType::ProLance => yasai::PieceType::NY,
                        shogi::PieceType::ProPawn => yasai::PieceType::TO,
                    };
                    yasai::Piece::from_cp(color, piece_type)
                })
        });
        let mut hand_nums = [[0; yasai::PieceType::NUM_HAND]; yasai::Color::NUM];
        for c in yasai::Color::ALL {
            for (i, &pt) in yasai::PieceType::ALL_HAND.iter().enumerate() {
                let piece_type = match pt {
                    yasai::PieceType::FU => shogi::PieceType::Pawn,
                    yasai::PieceType::KY => shogi::PieceType::Lance,
                    yasai::PieceType::KE => shogi::PieceType::Knight,
                    yasai::PieceType::GI => shogi::PieceType::Silver,
                    yasai::PieceType::KI => shogi::PieceType::Gold,
                    yasai::PieceType::KA => shogi::PieceType::Bishop,
                    yasai::PieceType::HI => shogi::PieceType::Rook,
                    _ => unreachable!(),
                };
                let color = match c {
                    yasai::Color::Black => shogi::Color::Black,
                    yasai::Color::White => shogi::Color::White,
                };
                hand_nums[c.index()][i] = pos.hand(shogi::Piece { piece_type, color });
            }
        }
        let side_to_move = match pos.side_to_move() {
            shogi::Color::Black => yasai::Color::Black,
            shogi::Color::White => yasai::Color::White,
        };
        let ply = pos.ply() as u32;
        Self(Position::new(board, hand_nums, side_to_move, ply))
    }
}

impl dfpn_solver::Position for YasaiPosition {
    type M = Move;

    fn hash_key(&self) -> u64 {
        self.0.key()
    }
    fn generate_legal_moves(&mut self, node: Node) -> Vec<(Self::M, u64)> {
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
    fn do_move(&mut self, m: Self::M) {
        self.0.do_move(m);
    }
    fn undo_move(&mut self, m: Self::M) {
        self.0.undo_move(m);
    }
}

impl CalculateResult for YasaiPosition {
    fn calculate_result_and_score(&mut self, moves: &[Self::M]) -> (Vec<String>, usize) {
        let (mut ret, mut len) = (Vec::new(), moves.len());
        let mut total_hands = PieceType::ALL_HAND
            .map(|pt| self.0.hand(!self.0.side_to_move()).num(pt))
            .iter()
            .sum::<u8>();
        // 最終2手が「合駒→同」の場合は、合駒無効の詰みなので削除
        while len > 2 {
            if let (
                MoveType::Drop {
                    to: drop_to,
                    piece: _,
                },
                MoveType::Normal {
                    from: _,
                    to: move_to,
                    is_promotion: _,
                    piece: _,
                },
            ) = (moves[len - 2].move_type(), moves[len - 1].move_type())
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
        for (i, &m) in (0..len).zip(moves) {
            if i % 2 == 0 {
                if let MoveType::Normal {
                    from: _,
                    to,
                    is_promotion: _,
                    piece: _,
                } = m.move_type()
                {
                    if let Some(piece_type) = drops[to.index()].take() {
                        if self.0.hand(!self.0.side_to_move()).num(piece_type) > 0 {
                            // TODO: 候補から除外したいが このパターンだけが候補になる場合もある
                            zero = true;
                        }
                    }
                }
            } else if let MoveType::Drop { to, piece } = m.move_type() {
                drops[to.index()] = Some(piece.piece_type());
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
