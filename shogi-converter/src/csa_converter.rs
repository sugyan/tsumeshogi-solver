use crate::{Action, Board, Color, Hand, Move, PieceType, Position, Record, Square};
use csa::{
    Action as CsaAction, Color as CsaColor, GameRecord, MoveRecord, PieceType as CsaPieceType,
    Position as CsaPosition, Square as CsaSquare,
};

impl From<GameRecord> for Record {
    fn from(record: GameRecord) -> Self {
        Self {
            pos: Position::from(record.start_pos),
            moves: record.moves.into_iter().map(|m| m.into()).collect(),
        }
    }
}

impl From<CsaPosition> for Position {
    fn from(cp: CsaPosition) -> Self {
        let drop_pieces = cp
            .drop_pieces
            .into_iter()
            .filter_map(|(s, pt)| pt.try_into().map(|pt: PieceType| (s.into(), pt)).ok())
            .collect::<Vec<_>>();
        let board = cp.bulk.map_or(Board::default(), |b| {
            let mut board = Board::default();
            for (i, &row) in b.iter().enumerate() {
                for (j, &col) in row.iter().enumerate() {
                    board.0[i][j] =
                        col.map(|(c, pt)| pt.try_into().map(|pt| (Color::from(c), pt)).unwrap());
                }
            }
            board
        });
        let mut hand = Hand::default();
        cp.add_pieces
            .iter()
            .filter(|&(_, _, pt)| *pt != CsaPieceType::All)
            .for_each(|&(c, _, pt)| {
                if let Ok(pt) = PieceType::try_from(pt) {
                    hand.0[Color::from(c).index()][pt.index()] += 1;
                }
            });
        let side_to_move = Color::from(cp.side_to_move);
        // `AL`
        if let Some(&(c, _, _)) = cp
            .add_pieces
            .iter()
            .find(|(_, _, pt)| *pt == CsaPieceType::All)
        {
            let mut remains = [18, 4, 4, 4, 4, 2, 2];
            board
                .0
                .iter()
                .flat_map(|row| {
                    row.iter()
                        .filter_map(|&o| o.filter(|(_, pt)| pt.unpromoted().is_hand_piece()))
                })
                .for_each(|(_, pt)| {
                    remains[pt.unpromoted().index()] -= 1;
                });
            drop_pieces.iter().for_each(|(_, pt)| {
                remains[pt.index()] -= 1;
            });
            hand.0.iter().for_each(|h| {
                PieceType::iter()
                    .filter(|pt| pt.is_hand_piece())
                    .for_each(|pt| {
                        remains[pt.index()] -= h[pt.index()];
                    });
            });
            PieceType::iter()
                .filter(|pt| pt.is_hand_piece())
                .for_each(|pt| {
                    hand.0[Color::from(c).index()][pt.index()] += remains[pt.index()];
                })
        }
        Self {
            drop_pieces,
            board,
            hand,
            side_to_move,
        }
    }
}

impl From<MoveRecord> for Move {
    fn from(mr: MoveRecord) -> Self {
        Self {
            action: mr.action.into(),
            time: mr.time,
            comments: Vec::new(),
        }
    }
}

impl From<CsaAction> for Action {
    fn from(ca: CsaAction) -> Self {
        match ca {
            CsaAction::Move(color, from, to, piece_type) => Action::Move(
                Color::from(color),
                Square::from(from),
                Square::from(to),
                PieceType::try_from(piece_type).unwrap(),
            ),
            CsaAction::Toryo => Action::Toryo,
            CsaAction::Chudan => Action::Chudan,
            CsaAction::Sennichite => Action::Sennichite,
            CsaAction::TimeUp => Action::TimeUp,
            CsaAction::IllegalMove => Action::IllegalMove,
            CsaAction::IllegalAction(color) => Action::IllegalAction(Color::from(color)),
            CsaAction::Jishogi => Action::Jishogi,
            CsaAction::Kachi => Action::Kachi,
            CsaAction::Hikiwake => Action::Hikiwake,
            CsaAction::Matta => Action::Matta,
            CsaAction::Tsumi => Action::Tsumi,
            CsaAction::Fuzumi => Action::Fuzumi,
            CsaAction::Error => Action::Error,
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

impl TryFrom<CsaPieceType> for PieceType {
    type Error = ();

    fn try_from(cpt: CsaPieceType) -> Result<Self, Self::Error> {
        match cpt {
            CsaPieceType::Pawn => Ok(PieceType::Pawn),
            CsaPieceType::Lance => Ok(PieceType::Lance),
            CsaPieceType::Knight => Ok(PieceType::Knight),
            CsaPieceType::Silver => Ok(PieceType::Silver),
            CsaPieceType::Gold => Ok(PieceType::Gold),
            CsaPieceType::Bishop => Ok(PieceType::Bishop),
            CsaPieceType::Rook => Ok(PieceType::Rook),
            CsaPieceType::King => Ok(PieceType::King),
            CsaPieceType::ProPawn => Ok(PieceType::ProPawn),
            CsaPieceType::ProLance => Ok(PieceType::ProLance),
            CsaPieceType::ProKnight => Ok(PieceType::ProKnight),
            CsaPieceType::ProSilver => Ok(PieceType::ProSilver),
            CsaPieceType::Horse => Ok(PieceType::Horse),
            CsaPieceType::Dragon => Ok(PieceType::Dragon),
            CsaPieceType::All => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use csa::parse_csa;
    use std::time::Duration;

    #[test]
    fn test_from() {
        let csa_game_record = parse_csa(
            &r#"
'----------棋譜ファイルの例"example.csa"-----------------
'バージョン
V2.2
'対局者名
N+NAKAHARA
N-YONENAGA
'棋譜情報
'棋戦名
$EVENT:13th World Computer Shogi Championship
'対局場所
$SITE:KAZUSA ARC
'開始日時
$START_TIME:2003/05/03 10:30:00
'終了日時
$END_TIME:2003/05/03 11:11:05
'持ち時間:25分、切れ負け
$TIME_LIMIT:00:25+00
'戦型:矢倉
$OPENING:YAGURA
'平手の局面
P1-KY-KE-GI-KI-OU-KI-GI-KE-KY
P2 * -HI *  *  *  *  * -KA * 
P3-FU-FU-FU-FU-FU-FU-FU-FU-FU
P4 *  *  *  *  *  *  *  *  * 
P5 *  *  *  *  *  *  *  *  * 
P6 *  *  *  *  *  *  *  *  * 
P7+FU+FU+FU+FU+FU+FU+FU+FU+FU
P8 * +KA *  *  *  *  * +HI * 
P9+KY+KE+GI+KI+OU+KI+GI+KE+KY
'先手番
+
'指し手と消費時間
+2726FU
T12
-3334FU
T6
%CHUDAN
'---------------------------------------------------------
"#[1..],
        )
        .expect("failed to parse CSA string");
        assert_eq!(
            Record {
                pos: Position::default(),
                moves: vec![
                    Move {
                        action: Action::Move(
                            Color::Black,
                            Square::new(2, 7),
                            Square::new(2, 6),
                            PieceType::Pawn,
                        ),
                        time: Some(Duration::from_secs(12)),
                        comments: Vec::new(),
                    },
                    Move {
                        action: Action::Move(
                            Color::White,
                            Square::new(3, 3),
                            Square::new(3, 4),
                            PieceType::Pawn,
                        ),
                        time: Some(Duration::from_secs(6)),
                        comments: Vec::new(),
                    },
                    Move {
                        action: Action::Chudan,
                        time: None,
                        comments: Vec::new(),
                    },
                ],
            },
            Record::from(csa_game_record),
        );
    }
}
