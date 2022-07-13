use crate::SearchOrCancel;
use dfpn::search::Search;
use dfpn::{Node, Position, Table, U};
use instant::Instant;
use shogi_core::Move;
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Clone, Copy, Debug)]
pub enum CanceledError {
    #[error("time limit exceeded")]
    Timeout,
}

pub struct CancelableSearcher<P, T> {
    pub pos: P,
    table: T,
    timeout: Option<Duration>,
    started: Instant,
    error: Option<CanceledError>,
}

impl<P, T> CancelableSearcher<P, T>
where
    P: Position,
    T: Table,
{
    pub fn new(pos: P, timeout: Option<Duration>) -> Self {
        Self {
            pos,
            table: T::default(),
            timeout,
            started: Instant::now(),
            error: None,
        }
    }
    pub fn dfpn_search(&mut self) -> Result<(), CanceledError> {
        self.started = Instant::now();
        self.error = None;
        SearchOrCancel::dfpn_search(self);
        self.error.map_or(Ok(()), Result::Err)
    }
}

impl<P, T> Search<P, T> for CancelableSearcher<P, T>
where
    P: Position,
    T: Table,
{
    fn hash_key(&self) -> u64 {
        self.pos.hash_key()
    }
    fn generate_legal_moves(&mut self, node: Node) -> Vec<(Move, u64)> {
        self.pos.generate_legal_moves(node)
    }
    fn do_move(&mut self, m: Move) {
        self.pos.do_move(m)
    }
    fn undo_move(&mut self, m: Move) {
        self.pos.undo_move(m)
    }
    fn look_up_hash(&self, key: &u64) -> (U, U) {
        self.table.look_up_hash(key)
    }
    fn put_in_hash(&mut self, key: u64, value: (U, U)) {
        self.table.put_in_hash(key, value)
    }
}

impl<P, T> SearchOrCancel<P, T> for CancelableSearcher<P, T>
where
    P: Position,
    T: Table,
{
    fn cancel(&mut self) -> bool {
        if let Some(timeout) = self.timeout {
            if self.started.elapsed() > timeout {
                self.error = Some(CanceledError::Timeout);
            }
        }
        self.error.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shogi_core::{Color, Piece, Square};
    use std::collections::HashMap;

    struct InfinityPosition(u64);

    impl Position for InfinityPosition {
        fn hash_key(&self) -> u64 {
            self.0
        }
        fn generate_legal_moves(&mut self, _node: Node) -> Vec<(Move, u64)> {
            vec![
                (
                    Move::Drop {
                        to: Square::SQ_1A,
                        piece: Piece::B_P,
                    },
                    self.0 + 1,
                ),
                (
                    Move::Drop {
                        to: Square::SQ_1A,
                        piece: Piece::W_P,
                    },
                    self.0 + 2,
                ),
            ]
        }
        fn do_move(&mut self, m: Move) {
            if let Move::Drop { to: _, piece } = m {
                self.0 += match piece.color() {
                    Color::Black => 1,
                    Color::White => 2,
                };
            }
        }
        fn undo_move(&mut self, m: Move) {
            if let Move::Drop { to: _, piece } = m {
                self.0 -= match piece.color() {
                    Color::Black => 1,
                    Color::White => 2,
                };
            }
        }
    }

    #[derive(Default)]
    struct HashMapTable {
        table: HashMap<u64, (U, U)>,
    }

    impl Table for HashMapTable {
        fn look_up_hash(&self, key: &u64) -> (U, U) {
            if let Some(&v) = self.table.get(key) {
                v
            } else {
                (1, 1)
            }
        }
        fn put_in_hash(&mut self, key: u64, value: (U, U)) {
            self.table.insert(key, value);
        }
    }

    #[test]
    fn timeout() {
        let mut searcher: CancelableSearcher<_, HashMapTable> =
            CancelableSearcher::new(InfinityPosition(1), Some(Duration::from_millis(10)));
        match searcher.dfpn_search() {
            Err(CanceledError::Timeout) => {}
            _ => panic!("expected timeout"),
        }
    }
}
