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
    hash_history: Vec<T>,
}

impl<T> ZobristHashPosition<T>
where
    T: Default + Copy + BitXorAssign,
    Standard: Distribution<T>,
{
    pub fn new(pos: Position) -> Self {
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
            hash_history: vec![hash],
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
        let (prev_from, prev_to) = if let Move::Normal {
            from,
            to,
            promote: _,
        } = m
        {
            (*self.pos.piece_at(from), *self.pos.piece_at(to))
        } else {
            (None, None)
        };
        match self.pos.make_move(m) {
            Ok(_) => {
                let mut hash = self.to_hash();
                match m {
                    Move::Normal {
                        from,
                        to,
                        promote: _,
                    } => {
                        if let Some(p) = prev_from {
                            hash ^= self.table_board[from.index()][p.piece_type.index()]
                                [p.color.index()];
                        }
                        if let Some(p) = prev_to {
                            hash ^=
                                self.table_board[to.index()][p.piece_type.index()][p.color.index()];
                        }
                        if let Some(p) = self.pos.piece_at(to) {
                            hash ^=
                                self.table_board[to.index()][p.piece_type.index()][p.color.index()];
                        }
                    }
                    Move::Drop { to, piece_type } => {
                        if let Some(p) = self.pos.piece_at(to) {
                            hash ^=
                                self.table_board[to.index()][p.piece_type.index()][p.color.index()];
                        }
                        let color = self.pos.side_to_move().flip();
                        let num = self.pos.hand(Piece { piece_type, color }) as usize;
                        hash ^= self.table_hand[piece_type.index()][color.index()][num];
                        hash ^= self.table_hand[piece_type.index()][color.index()][num + 1];
                    }
                }
                Color::iter().for_each(|color| hash ^= self.table_turn[color.index()]);
                self.hash_history.push(hash);
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
        match self.pos.unmake_move() {
            Ok(_) => {
                self.hash_history.pop();
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
    fn to_hash(&self) -> V {
        *self.hash_history.last().expect("latest hash has not found")
    }
}
