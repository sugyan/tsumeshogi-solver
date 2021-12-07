use crate::dfpn::dfpn_solver;
use crate::{INF, U};
use shogi::{Bitboard, Color, Move, MoveError, Piece, Position, Square};
use std::{fmt::Debug, hash::Hash};

#[derive(Clone, Copy, PartialEq)]
pub enum Node {
    Or,
    And,
}

impl Node {
    pub fn flip(self) -> Self {
        match self {
            Node::Or => Node::And,
            Node::And => Node::Or,
        }
    }
}

pub trait HashPosition: Default {
    type T: Eq + Hash + Copy + Debug;
    fn find_king(&self, c: Color) -> Option<Square>;
    fn hand(&self, p: Piece) -> u8;
    fn in_check(&self, color: Color) -> bool;
    fn make_move(&mut self, m: Move) -> Result<(), MoveError>;
    fn move_candidates(&self, sq: Square, p: Piece) -> Bitboard;
    fn piece_at(&self, sq: Square) -> &Option<Piece>;
    fn player_bb(&self, c: Color) -> &Bitboard;
    fn ply(&self) -> u16;
    fn side_to_move(&self) -> Color;
    fn unmake_move(&mut self) -> Result<(), MoveError>;

    fn set_position(&mut self, pos: Position);
    fn current_hash(&self) -> Self::T;
}

pub trait Table: Default {
    type T;
    fn look_up_hash(&self, key: &Self::T) -> (U, U);
    fn put_in_hash(&mut self, key: Self::T, value: (U, U));

    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
}

pub trait DFPN<P, T>: dfpn_solver::Solve<P, T>
where
    P: HashPosition,
    T: Table<T = P::T>,
{
    // ルートでの反復深化
    fn dfpn(&mut self, pos: Position) {
        let hash = self.set_position(pos);
        // ルートでの反復深化
        let (pn, dn) = self.mid(hash, INF - 1, INF - 1, Node::Or);
        if pn != INF && dn != INF {
            self.mid(hash, INF, INF, Node::Or);
        }
    }
}
