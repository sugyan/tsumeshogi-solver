use crate::{Board, Color, PieceType, Position, Record, Square};
use csa::{
    Color as CsaColor, GameRecord, PieceType as CsaPieceType, Position as CsaPosition,
    Square as CsaSquare,
};
use std::collections::HashMap;

impl From<GameRecord> for Record {
    fn from(record: GameRecord) -> Self {
        Self {
            start_pos: Position::from(record.start_pos),
            moves: Vec::new(),
        }
    }
}

impl From<CsaPosition> for Position {
    fn from(cp: CsaPosition) -> Self {
        let drop_pieces = cp
            .drop_pieces
            .into_iter()
            .map(|(s, pt)| (s.into(), pt.into()))
            .collect::<Vec<_>>();
        let bulk = cp.bulk.map(|b| {
            let mut board = Board::default();
            for (i, &row) in b.iter().enumerate() {
                for (j, &col) in row.iter().enumerate() {
                    board[i][j] = col.map(|(c, pt)| (Color::from(c), PieceType::from(pt)));
                }
            }
            board
        });
        let mut add_pieces = cp
            .add_pieces
            .iter()
            .filter_map(|&(c, s, pt)| {
                if pt != CsaPieceType::All {
                    Some((c.into(), s.into(), pt.into()))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        let side_to_move = Color::from(cp.side_to_move);
        // `AL`
        if let Some(&(color, _, _)) = cp
            .add_pieces
            .iter()
            .find(|(_, _, pt)| *pt == CsaPieceType::All)
        {
            let mut counts = [
                (PieceType::King, 2),
                (PieceType::Rook, 2),
                (PieceType::Bishop, 2),
                (PieceType::Gold, 4),
                (PieceType::Silver, 4),
                (PieceType::Knight, 4),
                (PieceType::Lance, 4),
                (PieceType::Pawn, 18),
            ]
            .into_iter()
            .collect::<HashMap<_, _>>();
            if let Some(b) = bulk {
                b.iter()
                    .flat_map(|row| row.iter().filter_map(|&o| o))
                    .for_each(|(_, pt)| {
                        let c = counts.get_mut(&pt.unpromoted()).expect("unknown piece");
                        *c -= 1;
                        assert!(*c >= 0, "invalid pieces count");
                    })
            }
            drop_pieces.iter().for_each(|(_, pt)| {
                let c = counts.get_mut(pt).expect("unknown drop_piece");
                *c -= 1;
                assert!(*c >= 0, "invalid pieces count");
            });
            add_pieces.iter().for_each(|(_, _, pt)| {
                let c = counts.get_mut(pt).expect("unknown add_piece");
                *c -= 1;
                assert!(*c >= 0, "invalid pieces count");
            });
            [
                PieceType::Rook,
                PieceType::Bishop,
                PieceType::Gold,
                PieceType::Silver,
                PieceType::Knight,
                PieceType::Lance,
                PieceType::Pawn,
            ]
            .iter()
            .for_each(|pt| {
                for _ in 0..*counts.get(pt).unwrap_or(&0) {
                    add_pieces.push((color.into(), Square { file: 0, rank: 0 }, *pt));
                }
            })
        }
        Self {
            drop_pieces,
            bulk,
            add_pieces,
            side_to_move,
        }
    }
}

impl From<CsaColor> for Color {
    fn from(cc: CsaColor) -> Self {
        match cc {
            CsaColor::Black => Color::Black,
            CsaColor::White => Color::White,
        }
    }
}

impl From<CsaSquare> for Square {
    fn from(cs: CsaSquare) -> Self {
        Square {
            file: cs.file,
            rank: cs.rank,
        }
    }
}

impl From<CsaPieceType> for PieceType {
    fn from(cpt: CsaPieceType) -> Self {
        match cpt {
            CsaPieceType::Pawn => PieceType::Pawn,
            CsaPieceType::Lance => PieceType::Lance,
            CsaPieceType::Knight => PieceType::Knight,
            CsaPieceType::Silver => PieceType::Silver,
            CsaPieceType::Gold => PieceType::Gold,
            CsaPieceType::Bishop => PieceType::Bishop,
            CsaPieceType::Rook => PieceType::Rook,
            CsaPieceType::King => PieceType::King,
            CsaPieceType::ProPawn => PieceType::ProPawn,
            CsaPieceType::ProLance => PieceType::ProLance,
            CsaPieceType::ProKnight => PieceType::ProKnight,
            CsaPieceType::ProSilver => PieceType::ProSilver,
            CsaPieceType::Horse => PieceType::Horse,
            CsaPieceType::Dragon => PieceType::Dragon,
            CsaPieceType::All => PieceType::Extra,
        }
    }
}
