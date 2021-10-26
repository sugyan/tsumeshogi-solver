use shogi::bitboard::Factory;
use shogi::{Color, Move, Piece, PieceType, Position, Square};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

#[derive(Default)]
pub struct Solver {
    table: HashMap<u64, (u64, u64)>,
}

impl Solver {
    pub fn new() -> Self {
        Factory::init();
        Default::default()
    }
    pub fn solve(&mut self, pos: &mut Position) -> String {
        println!("{}", pos.to_sfen());
        for &m in &valid_moves(pos) {
            pos.make_move(m).expect("failed to make move");
            print!("{} -> {:>20}", m, hash(pos));
            pos.unmake_move().expect("failed to unmake move");
            println!(" -> {}", hash(pos));
        }
        self.mid(pos);
        String::new()
    }

    fn mid(&mut self, pos: &mut Position) {
        self.table.insert(hash(pos), (0, 0));
    }
}

fn p64(p: Piece) -> u64 {
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

// TODO
fn hash(pos: &Position) -> u64 {
    let mut hasher = DefaultHasher::new();
    Square::iter().for_each(|sq| {
        pos.piece_at(sq).map_or(28, |p| p64(p)).hash(&mut hasher);
    });
    PieceType::iter().for_each(|piece_type| {
        Color::iter().for_each(|color| {
            pos.hand(Piece { piece_type, color }).hash(&mut hasher);
        })
    });
    match pos.side_to_move() {
        Color::Black => 0.hash(&mut hasher),
        Color::White => 1.hash(&mut hasher),
    };
    hasher.finish()
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
