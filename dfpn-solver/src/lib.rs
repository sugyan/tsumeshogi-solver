mod position;

use position::{HashablePosition, NaiveHashPosition, ShogiPosition};
use shogi::bitboard::Factory;
use shogi::{Color, Move, Piece, PieceType, Position, Square};

pub fn solve(pos: &Position) {
    Factory::init();

    // copy the position
    let sfen = pos.to_sfen();
    let mut pos = Position::new();
    pos.set_sfen(&sfen).expect("failed to parse SFEN string");

    let mut solver = Solver::new(NaiveHashPosition::new(pos));
    solver.solve();
}

struct Solver<T> {
    pos: T,
}

impl<T: HashablePosition> Solver<T> {
    fn new(pos: T) -> Self {
        Self { pos }
    }
    fn solve(&mut self) {
        for &m in &valid_moves(&mut self.pos) {
            self.pos.make_move(m).expect("failed to make move");
            println!("{}", m);
            self.pos.unmake_move().expect("failed to unmake move");
        }
        self.mid();
    }
    fn mid(&mut self) {
        let (p, d) = self.pos.look_up_hash();
        println!("{:?}", (p, d));
    }
}

fn valid_moves<T: ShogiPosition>(pos: &mut T) -> Vec<Move> {
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
