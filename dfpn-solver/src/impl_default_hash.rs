use crate::HashPosition;
use shogi::{Bitboard, Color, Move, MoveError, Piece, PieceType, Position, Square};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub struct DefaultHashPosition(Position);

impl DefaultHashPosition {
    pub fn new(pos: Position) -> Self {
        Self(pos)
    }
}

impl HashPosition for DefaultHashPosition {
    type T = u64;
    fn hand(&self, p: Piece) -> u8 {
        self.0.hand(p)
    }
    fn in_check(&self, c: Color) -> bool {
        self.0.in_check(c)
    }
    fn make_move(&mut self, m: Move) -> Result<(), MoveError> {
        self.0.make_move(m)
    }
    fn move_candidates(&self, sq: Square, p: Piece) -> Bitboard {
        self.0.move_candidates(sq, p)
    }
    fn piece_at(&self, sq: Square) -> &Option<Piece> {
        self.0.piece_at(sq)
    }
    fn player_bb(&self, c: Color) -> &Bitboard {
        self.0.player_bb(c)
    }
    fn side_to_move(&self) -> Color {
        self.0.side_to_move()
    }
    fn unmake_move(&mut self) -> Result<(), MoveError> {
        self.0.unmake_move()
    }
    fn to_hash(&self) -> u64 {
        let mut s = DefaultHasher::new();
        self.hash(&mut s);
        s.finish()
    }
}

impl Hash for DefaultHashPosition {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Square::iter().for_each(|sq| {
            self.0.piece_at(sq).map_or(28, |p| p8(p)).hash(state);
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
