use shogi::{Bitboard, Color, Move, MoveError, Piece, PieceType, Position, Square};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

pub trait ShogiPosition {
    fn hand(&self, p: Piece) -> u8;
    fn in_check(&self, c: Color) -> bool;
    fn make_move(&mut self, m: Move) -> Result<(), MoveError>;
    fn move_candidates(&self, sq: Square, p: Piece) -> Bitboard;
    fn piece_at(&self, sq: Square) -> &Option<Piece>;
    fn player_bb(&self, c: Color) -> &Bitboard;
    fn side_to_move(&self) -> Color;
    fn unmake_move(&mut self) -> Result<(), MoveError>;
}

pub trait HashablePosition: ShogiPosition {
    fn look_up_hash(&self) -> (u32, u32);
    fn put_in_hash(&mut self, value: (u32, u32));
}

// NaiveHashPosition

pub struct NaiveHashPosition {
    pos: PositionWrapper,
    table: HashMap<u64, (u32, u32)>,
}

impl NaiveHashPosition {
    pub fn new(pos: Position) -> Self {
        Self {
            pos: PositionWrapper(pos),
            table: HashMap::new(),
        }
    }
}

impl HashablePosition for NaiveHashPosition {
    fn look_up_hash(&self) -> (u32, u32) {
        *self.table.get(&u64::from(&self.pos)).unwrap_or(&(1, 1))
    }
    fn put_in_hash(&mut self, value: (u32, u32)) {
        self.table.insert(u64::from(&self.pos), value);
    }
}

impl ShogiPosition for NaiveHashPosition {
    fn hand(&self, p: Piece) -> u8 {
        self.pos.0.hand(p)
    }
    fn in_check(&self, c: Color) -> bool {
        self.pos.0.in_check(c)
    }
    fn make_move(&mut self, m: Move) -> Result<(), MoveError> {
        self.pos.0.make_move(m)
    }
    fn move_candidates(&self, sq: Square, p: Piece) -> Bitboard {
        self.pos.0.move_candidates(sq, p)
    }
    fn piece_at(&self, sq: Square) -> &Option<Piece> {
        self.pos.0.piece_at(sq)
    }
    fn player_bb(&self, c: Color) -> &Bitboard {
        self.pos.0.player_bb(c)
    }
    fn side_to_move(&self) -> Color {
        self.pos.0.side_to_move()
    }
    fn unmake_move(&mut self) -> Result<(), MoveError> {
        self.pos.0.unmake_move()
    }
}

// PositionWrapper for NaiveHashPosition

struct PositionWrapper(Position);

impl From<&PositionWrapper> for u64 {
    fn from(pr: &PositionWrapper) -> Self {
        let mut s = DefaultHasher::new();
        pr.hash(&mut s);
        s.finish()
    }
}

impl Hash for PositionWrapper {
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

impl PartialEq for PositionWrapper {
    fn eq(&self, other: &Self) -> bool {
        u64::from(self) == u64::from(other)
    }
}

impl Eq for PositionWrapper {}

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
