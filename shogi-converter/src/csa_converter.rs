use crate::{Board, Color, PieceType, Position, Record, Square};
use csa::{
    Color as CsaColor, GameRecord, PieceType as CsaPieceType, Position as CsaPosition,
    Square as CsaSquare,
};

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
        Self {
            drop_pieces: cp
                .drop_pieces
                .into_iter()
                .map(|(s, pt)| (s.into(), pt.into()))
                .collect(),
            bulk: cp.bulk.map(|b| {
                let mut board = Board::default();
                for (i, &row) in b.iter().enumerate() {
                    for (j, &col) in row.iter().enumerate() {
                        board[i][j] = col.map(|(c, pt)| (Color::from(c), PieceType::from(pt)));
                    }
                }
                board
            }),
            add_pieces: cp
                .add_pieces
                .into_iter()
                .map(|(c, s, pt)| (c.into(), s.into(), pt.into()))
                .collect(),
            side_to_move: Color::from(cp.side_to_move),
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
            CsaPieceType::All => PieceType::All,
        }
    }
}
