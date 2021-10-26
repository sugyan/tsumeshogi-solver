use shogi::bitboard::Factory;
use shogi::{Color, Move, MoveError, Piece, PieceType, Position, Square};

pub fn solve(pos: &mut Position) -> String {
    Factory::init();

    let moves = valid_moves(pos, Color::Black);
    println!("solve {}", pos.to_sfen());
    for m in &moves {
        println!("{}", m);
    }
    String::new()
}

fn valid_moves(pos: &mut Position, color: Color) -> Vec<Move> {
    let &bb = pos.player_bb(color);
    let mut moves = Vec::new();
    match color {
        Color::Black => {
            // normal moves
            for from in bb {
                if let Some(p) = pos.piece_at(from) {
                    for to in pos.move_candidates(from, *p) {
                        for promote in [true, false] {
                            let m = Move::Normal { from, to, promote };
                            match pos.make_move(m) {
                                Ok(_) => {
                                    if pos.in_check(Color::White) && !pos.in_check(Color::Black) {
                                        moves.push(m);
                                    }
                                    pos.unmake_move().expect("failed to unmake move");
                                }
                                Err(MoveError::Inconsistent(_)) => {
                                    // ignore
                                }
                                Err(e) => panic!("move error {}: {}", m, e),
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
                            if pos.in_check(Color::White) && !pos.in_check(Color::Black) {
                                moves.push(m);
                            }
                            pos.unmake_move().expect("failed to unmake move");
                        }
                        Err(MoveError::Inconsistent(_)) => {
                            // ignore
                        }
                        Err(e) => panic!("move error {}: {}", m, e),
                    }
                }
            }
        }
        Color::White => {}
    }
    moves
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
