use shogi::bitboard::Factory;
use shogi::{Color, Move, Piece, PieceType, Position, Square};

pub fn solve(pos: &mut Position) -> String {
    Factory::init();

    println!("solve {}", pos.to_sfen());
    for &m in &valid_moves(pos) {
        println!("{}", m);
        pos.make_move(m).expect("failed to make move");
        for &m in &valid_moves(pos) {
            println!(" -> {}", m);
        }
        pos.unmake_move().expect("failed to unmake move");
    }
    String::new()
}

fn valid_moves(pos: &mut Position) -> Vec<Move> {
    let color = pos.side_to_move();
    let &bb = pos.player_bb(color);
    let mut moves = Vec::new();
    // normal moves
    for from in bb {
        if let Some(p) = pos.piece_at(from) {
            for to in pos.move_candidates(from, *p) {
                for promote in [true, false] {
                    let m = Move::Normal { from, to, promote };
                    match pos.make_move(m) {
                        Ok(()) => {
                            if color == Color::White || pos.in_check(Color::White) {
                                moves.push(m);
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
                        moves.push(m);
                    }
                    pos.unmake_move().expect("failed to unmake move");
                }
                Err(_) => {
                    // ignore
                }
            }
        }
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
