use crate::HashPosition;
use rand::distributions::{Distribution, Standard};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use shogi::{Bitboard, Color, Move, MoveError, Piece, PieceType, Position, Square};
use std::{hash::Hash, ops::BitXorAssign};

pub struct ZobristHashPosition<T> {
    pos: Position,
    table_board: [[[T; 2]; 14]; 81],
    table_hand: [[[T; 19]; 2]; 14],
    table_turn: [T; 2],
    hash: T,
}

impl<T> ZobristHashPosition<T>
where
    T: Default + Copy + BitXorAssign,
    Standard: Distribution<T>,
{
    pub fn new(p: &Position) -> Self {
        let mut pos = Position::new();
        pos.set_sfen(&p.to_sfen())
            .expect("failed to parse SFEN string");

        // init table
        let mut rng = SmallRng::seed_from_u64(0);
        let mut table_board = [[[T::default(); 2]; 14]; 81];
        let mut table_hand = [[[T::default(); 19]; 2]; 14];
        let mut table_turn = [T::default(); 2];
        Square::iter().for_each(|sq| {
            PieceType::iter().for_each(|piece_type| {
                Color::iter().for_each(|color| {
                    table_board[sq.index()][piece_type.index()][color.index()] = rng.gen();
                });
            });
        });
        PieceType::iter().for_each(|piece_type| {
            Color::iter().for_each(|color| {
                (0..=18).for_each(|num| {
                    table_hand[piece_type.index()][color.index()][num] = rng.gen();
                });
            });
        });
        Color::iter().for_each(|color| {
            table_turn[color.index()] = rng.gen();
        });
        // calcualte hash for the position
        let mut hash = T::default();
        Square::iter().for_each(|sq| {
            if let Some(p) = pos.piece_at(sq) {
                hash ^= table_board[sq.index()][p.piece_type.index()][p.color.index()];
            }
        });
        PieceType::iter().for_each(|piece_type| {
            Color::iter().for_each(|color| {
                let num = pos.hand(Piece { piece_type, color });
                hash ^= table_hand[piece_type.index()][color.index()][num as usize];
            });
        });
        hash ^= table_turn[pos.side_to_move().index()];
        Self {
            pos,
            table_board,
            table_hand,
            table_turn,
            hash,
        }
    }
}

impl<V> HashPosition for ZobristHashPosition<V>
where
    V: Copy + Eq + Hash + BitXorAssign,
{
    type T = V;
    fn hand(&self, p: Piece) -> u8 {
        self.pos.hand(p)
    }
    fn in_check(&self, c: Color) -> bool {
        self.pos.in_check(c)
    }
    fn make_move(&mut self, m: Move) -> Result<(), MoveError> {
        match self.pos.make_move(m) {
            Ok(_) => {
                // TODO: update hash
                match m {
                    Move::Normal { from, to, promote } => {
                        println!("{} {} {}", from, to, promote);
                    }
                    Move::Drop { to, piece_type } => {
                        println!("{} {}", to, piece_type);
                    }
                }
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
    fn move_candidates(&self, sq: Square, p: Piece) -> Bitboard {
        self.pos.move_candidates(sq, p)
    }
    fn piece_at(&self, sq: Square) -> &Option<Piece> {
        self.pos.piece_at(sq)
    }
    fn player_bb(&self, c: Color) -> &Bitboard {
        self.pos.player_bb(c)
    }
    fn side_to_move(&self) -> Color {
        self.pos.side_to_move()
    }
    fn unmake_move(&mut self) -> Result<(), MoveError> {
        let last = self.pos.move_history().last().unwrap();
        match self.pos.unmake_move() {
            Ok(_) => {
                // TODO: update hash
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
    fn to_hash(&self) -> V {
        self.hash
    }
}
