use crate::{HashPosition, Node};
use shogi::{Move, MoveError, Piece, PieceType};

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
