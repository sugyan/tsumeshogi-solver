mod impl_naive_position;

use impl_naive_position::NaiveHashPosition;
use shogi::bitboard::Factory;
use shogi::{Bitboard, Color, Move, MoveError, Piece, PieceType, Position, Square};
use std::collections::HashMap;

type Num = u16;
const MAX: Num = Num::MAX;

pub fn solve(pos: &Position) {
    Factory::init();

    // copy the position
    let sfen = pos.to_sfen();
    let mut pos = Position::new();
    pos.set_sfen(&sfen).expect("failed to parse SFEN string");

    let mut solver = Solver::new(NaiveHashPosition::new(pos));
    solver.solve();
}

trait HashablePosition {
    fn hand(&self, p: Piece) -> u8;
    fn in_check(&self, c: Color) -> bool;
    fn make_move(&mut self, m: Move) -> Result<(), MoveError>;
    fn move_candidates(&self, sq: Square, p: Piece) -> Bitboard;
    fn piece_at(&self, sq: Square) -> &Option<Piece>;
    fn player_bb(&self, c: Color) -> &Bitboard;
    fn side_to_move(&self) -> Color;
    fn unmake_move(&mut self) -> Result<(), MoveError>;
    fn to_hash(&self) -> u64;
}

struct Solver<T> {
    pos: T,
    table: HashMap<u64, (Num, Num)>,
}

impl<T> Solver<T>
where
    T: HashablePosition,
{
    fn new(pos: T) -> Self {
        Self {
            pos,
            table: HashMap::new(),
        }
    }
    fn solve(&mut self) {
        self.mid((MAX - 1, MAX - 1));
    }
    fn mid(&mut self, (phi, delta): (Num, Num)) {
        let (p, d) = self.look_up_hash();
        if phi < p || delta < d {
            // TODO
            return;
        }

        let children = generate_legal_moves(&mut self.pos);
        if children.is_empty() {
            println!("empty!!");
            // TODO
            return;
        }
        self.put_in_hash((phi, delta));
        loop {
            let md = self.min_delta(&children);
            let sp = self.sum_phi(&children);

            let (best, phi_c, delta_c, delta_2) = self.select_child(&children);
            println!(
                "select: {}",
                best.map_or(String::from("None"), |m| m.to_string())
            );
            // let phi_n_c = if phi_c == MAX - 1 {
            //     MAX
            // } else if delta >= MAX - 1 {
            //     MAX - 1
            // } else {
            //     delta + phi_c - phi_sum
            // };
            if let Some(m) = best {
                self.pos.make_move(m).expect("failed to make move");
                self.mid((phi_c, delta_c));
            }
            break;
        }

        println!("{:?}", (p, d));
    }
    fn select_child(&mut self, children: &[(Move, u64)]) -> (Option<Move>, Num, Num, Num) {
        let (mut delta_c, mut delta_2) = (MAX, MAX);
        let mut best = None;
        let mut phi_c = None; // not optional?
        for &(m, _) in children {
            self.pos.make_move(m).expect("failed to make move");
            let (p, d) = self.look_up_hash();
            self.pos.unmake_move().expect("failed to unmake move");
            if d < delta_c {
                best = Some(m);
                delta_2 = delta_c;
                phi_c = Some(p);
                delta_c = d;
            } else if d < delta_2 {
                delta_2 = d;
            }
            if p == MAX {
                return (best, phi_c.expect("phi_c"), delta_c, delta_2);
            }
        }
        (best, phi_c.expect("phi_c"), delta_c, delta_2)
    }

    fn look_up_hash(&self) -> (Num, Num) {
        *self.table.get(&self.pos.to_hash()).unwrap_or(&(1, 1))
    }
    fn put_in_hash(&mut self, value: (Num, Num)) {
        self.table.insert(self.pos.to_hash(), value);
    }
    fn min_delta(&mut self, children: &[(Move, u64)]) -> Num {
        // TODO
        0
    }
    fn sum_phi(&mut self, children: &[(Move, u64)]) -> Num {
        // TODO
        0
    }
}

fn generate_legal_moves<T>(pos: &mut T) -> Vec<(Move, u64)>
where
    T: HashablePosition,
{
    let color = pos.side_to_move();
    let &bb = pos.player_bb(color);
    let mut children = Vec::new();
    // normal moves
    for from in bb {
        if let Some(p) = pos.piece_at(from) {
            for to in pos.move_candidates(from, *p) {
                for promote in [true, false] {
                    let m = Move::Normal { from, to, promote };
                    match pos.make_move(m) {
                        Ok(()) => {
                            if color == Color::White || pos.in_check(Color::White) {
                                children.push((m, pos.to_hash()));
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
                        children.push((m, pos.to_hash()));
                    }
                    pos.unmake_move().expect("failed to unmake move");
                }
                Err(_) => {
                    // ignore
                }
            }
        }
    }
    children
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
