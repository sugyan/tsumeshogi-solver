use crate::{Action, Board, Color, Hand, Move, PieceType, Position, Record, Square};
use encoding_rs::SHIFT_JIS;
use nom::branch::alt;
use nom::bytes::complete::{is_a, is_not, tag};
use nom::character::complete::{
    digit1, line_ending, none_of, not_line_ending, one_of, space0, space1,
};
use nom::combinator::{map, map_res, opt, value};
use nom::multi::{fill, many0, separated_list0};
use nom::sequence::{delimited, pair, preceded, separated_pair, terminated};
use nom::{IResult, Parser};
use std::{collections::HashMap, error::Error, fmt};

#[derive(Debug)]
pub enum KifError {
    ParseError(),
    EncodingNotShiftJISError(),
    EncodingError(),
}

impl fmt::Display for KifError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            KifError::ParseError() => write!(f, "failed to parse"),
            KifError::EncodingNotShiftJISError() => write!(f, "input was not SHIFT-JIS"),
            KifError::EncodingError() => write!(f, "failed to decode input"),
        }
    }
}

impl Error for KifError {}

pub fn parse_kif(bytes: &[u8]) -> Result<Record, KifError> {
    let (cow, encoding_used, had_errors) = SHIFT_JIS.decode(bytes);
    if encoding_used != SHIFT_JIS {
        return Err(KifError::EncodingNotShiftJISError());
    }
    if had_errors {
        return Err(KifError::EncodingError());
    }

    if let Ok((_, record)) = record(&cow) {
        Ok(record)
    } else {
        Err(KifError::ParseError())
    }
}

fn comment_line(input: &str) -> IResult<&str, &str> {
    terminated(preceded(tag("#"), not_line_ending), line_ending)(input)
}

fn attributes(input: &str) -> IResult<&str, Vec<(&str, &str)>> {
    many0(terminated(
        separated_pair(is_not("\r\n："), tag("："), not_line_ending),
        line_ending,
    ))(input)
}

fn piece(input: &str) -> IResult<&str, PieceType> {
    alt((
        value(PieceType::Pawn, tag("歩")),
        value(PieceType::Lance, tag("香")),
        value(PieceType::Knight, tag("桂")),
        value(PieceType::Silver, tag("銀")),
        value(PieceType::Gold, tag("金")),
        value(PieceType::Bishop, tag("角")),
        value(PieceType::Rook, tag("飛")),
        value(PieceType::King, tag("玉")),
        value(PieceType::ProPawn, tag("と")),
        value(PieceType::ProLance, alt((tag("杏"), tag("成香")))),
        value(PieceType::ProKnight, alt((tag("圭"), tag("成桂")))),
        value(PieceType::ProSilver, alt((tag("全"), tag("成銀")))),
        value(PieceType::Horse, tag("馬")),
        value(PieceType::Dragon, is_a("龍竜")),
    ))(input)
}

fn japanese_numerals(input: &str) -> IResult<&str, u8> {
    alt((
        value(11, tag("十一")),
        value(12, tag("十二")),
        value(13, tag("十三")),
        value(14, tag("十四")),
        value(15, tag("十五")),
        value(16, tag("十六")),
        value(17, tag("十七")),
        value(18, tag("十八")),
        value(1, tag("一")),
        value(2, tag("二")),
        value(3, tag("三")),
        value(4, tag("四")),
        value(5, tag("五")),
        value(6, tag("六")),
        value(7, tag("七")),
        value(8, tag("八")),
        value(9, tag("九")),
        value(10, tag("十")),
    ))(input)
}

fn piece_cell(input: &str) -> IResult<&str, Option<(Color, PieceType)>> {
    let (input, color) =
        alt((value(Color::Black, tag(" ")), value(Color::White, tag("v"))))(input)?;
    let (input, piece_type) = piece(input)?;
    Ok((input, Some((color, piece_type))))
}

fn board_piece(input: &str) -> IResult<&str, Option<(Color, PieceType)>> {
    alt((value(None, tag(" ・")), piece_cell))(input)
}

fn board_row(input: &str) -> IResult<&str, [Option<(Color, PieceType)>; 9]> {
    let mut ret = [None; 9];
    let (input, _) = tag("|")(input)?;
    let (input, _) = fill(board_piece, &mut ret)(input)?;
    let (input, _) = tag("|")(input)?;
    let (input, _) = one_of("一二三四五六七八九")(input)?;
    Ok((input, ret))
}

fn board(input: &str) -> IResult<&str, Board> {
    let (input, _) = terminated(tag("  ９ ８ ７ ６ ５ ４ ３ ２ １"), line_ending)(input)?;
    let (input, _) = terminated(tag("+---------------------------+"), line_ending)(input)?;
    let (input, r1) = terminated(board_row, line_ending)(input)?;
    let (input, r2) = terminated(board_row, line_ending)(input)?;
    let (input, r3) = terminated(board_row, line_ending)(input)?;
    let (input, r4) = terminated(board_row, line_ending)(input)?;
    let (input, r5) = terminated(board_row, line_ending)(input)?;
    let (input, r6) = terminated(board_row, line_ending)(input)?;
    let (input, r7) = terminated(board_row, line_ending)(input)?;
    let (input, r8) = terminated(board_row, line_ending)(input)?;
    let (input, r9) = terminated(board_row, line_ending)(input)?;
    let (input, _) = terminated(tag("+---------------------------+"), line_ending)(input)?;
    Ok((input, Board([r1, r2, r3, r4, r5, r6, r7, r8, r9])))
}

fn hand_piece(input: &str) -> IResult<&str, (PieceType, u8)> {
    let (input, hand_piece) = pair(
        piece,
        map(opt(japanese_numerals), |o: Option<u8>| o.unwrap_or(1)),
    )(input)?;
    Ok((input, hand_piece))
}

fn hands(input: &str) -> IResult<&str, [u8; 7]> {
    let (input, hands) = separated_list0(tag("　"), hand_piece)(input)?;
    Ok((
        input,
        hands.iter().fold([0; 7], |mut acc, &(piece_type, num)| {
            acc[piece_type.index()] = num;
            acc
        }),
    ))
}

fn move_from(input: &str) -> IResult<&str, Square> {
    let (input, (file, rank)) = alt((
        delimited(
            tag("("),
            map_res(digit1, |s: &str| s.parse::<u8>()).map(|d| (d / 10, d % 10)),
            tag(")"),
        ),
        value((0, 0), tag("打")),
    ))(input)?;
    Ok((input, Square::new(file, rank)))
}

fn move_to(input: &str) -> IResult<&str, Square> {
    let (input, (file, rank)) = alt((
        pair(
            alt((
                value(1, tag("１")),
                value(2, tag("２")),
                value(3, tag("３")),
                value(4, tag("４")),
                value(5, tag("５")),
                value(6, tag("６")),
                value(7, tag("７")),
                value(8, tag("８")),
                value(9, tag("９")),
            )),
            japanese_numerals,
        ),
        value((0, 0), tag("同　")),
    ))(input)?;
    Ok((input, Square::new(file, rank)))
}

fn move_action_move(input: &str) -> IResult<&str, Action> {
    let (input, to) = move_to(input)?;
    let (input, piece_type) = piece(input)?;
    let (input, promote) = map(opt(tag("成")), |o| o.is_some())(input)?;
    let (input, from) = move_from(input)?;
    Ok((
        input,
        Action::Move(
            Color::Black,
            from,
            to,
            if promote {
                piece_type.promote().expect("promote")
            } else {
                piece_type
            },
        ),
    ))
}

fn move_action(input: &str) -> IResult<&str, Action> {
    let (input, action) = alt((
        move_action_move,
        value(Action::Toryo, tag("投了")),
        value(Action::Chudan, tag("中断")),
        value(Action::Sennichite, tag("千日手")),
        value(Action::TimeUp, tag("切れ負け")),
        value(Action::Jishogi, tag("持将棋")),
        value(Action::Tsumi, tag("詰み")),
        // TODO: 反則、その他
        value(Action::Error, is_not(" ")),
    ))(input)?;
    Ok((input, action))
}

fn move_line(input: &str) -> IResult<&str, Move> {
    let (input, num) = map_res(digit1, |s: &str| s.parse::<usize>())(input)?;
    let (input, (mut action, _)) = preceded(
        space1,
        separated_pair(move_action, tag(" "), not_line_ending),
    )(input)?;
    // fix color...
    if let Action::Move(color, _, _, _) = &mut action {
        *color = if num % 2 == 1 {
            Color::Black
        } else {
            Color::White
        };
    }
    Ok((
        input,
        Move {
            action,
            time: None,
            comments: Vec::new(),
        },
    ))
}

fn move_(input: &str) -> IResult<&str, Move> {
    let (input, (mut m, comments)) = pair(
        terminated(preceded(space0, move_line), line_ending),
        many0(alt((
            terminated(preceded(tag("*"), not_line_ending), line_ending),
            terminated(preceded(tag("&"), not_line_ending), line_ending), // TODO: しおり
        ))),
    )(input)?;
    m.comments = comments.iter().map(|s| s.to_string()).collect();
    Ok((input, m))
}

fn moves(input: &str) -> IResult<&str, Vec<Move>> {
    let (input, mut moves) = many0(move_)(input)?;
    // TODO: 変化？
    let (input, _) = many0(terminated(not_line_ending, line_ending))(input)?;
    // 「同」
    for i in 1..moves.len() {
        if let (Action::Move(_, _, prev_to, _), Action::Move(_, _, curr_to, _)) =
            (moves[i - 1].action, &mut moves[i].action)
        {
            if *curr_to == (Square { file: 0, rank: 0 }) {
                *curr_to = prev_to;
            }
        }
    }
    Ok((input, moves))
}

fn record(input: &str) -> IResult<&str, Record> {
    let mut hm = HashMap::new();
    let (input, _) = many0(comment_line)(input)?;
    let (input, attrs) = attributes(input)?;
    for (key, value) in attrs {
        hm.insert(key, value);
    }
    // board
    let (input, _) = many0(comment_line)(input)?;
    let (input, board) = opt(board)(input)?;
    let (input, _) = many0(comment_line)(input)?;
    let (input, attrs) = attributes(input)?;
    for (key, value) in attrs {
        hm.insert(key, value);
    }
    let (input, _) = many0(comment_line)(input)?;
    // hands
    let mut hand = Hand::default();
    for (k, v) in hm {
        match k {
            "先手の持駒" => {
                if let Ok((_, h)) = hands(v) {
                    hand.0[Color::Black.index()] = h;
                }
            }
            "後手の持駒" => {
                if let Ok((_, h)) = hands(v) {
                    hand.0[Color::White.index()] = h;
                }
            }
            _ => {}
        }
    }
    // TODO: extra lines
    let (input, _) = many0(terminated(
        preceded(none_of(" 0123456789"), not_line_ending),
        line_ending,
    ))(input)?;
    // moves
    let (input, moves) = moves(input)?;
    Ok((
        input,
        Record {
            pos: Position {
                drop_pieces: Vec::new(),
                board: board.unwrap_or_default(),
                hand,
                side_to_move: Color::Black,
            },
            moves,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Color::*;
    use crate::PieceType::*;

    #[test]
    fn parse_comment() {
        assert_eq!(
            comment_line(
                &r"
# ---- Kifu for Windows95 V3.53 棋譜ファイル ----
"[1..]
            ),
            Result::Ok(("", " ---- Kifu for Windows95 V3.53 棋譜ファイル ----"))
        );
    }

    #[test]
    fn parse_attributes() {
        assert_eq!(
            attributes(
                &r"
開始日時：1999/07/15(木) 19:07:12
終了日時：1999/07/15(木) 19:07:17
手合割：平手
"[1..]
            ),
            Result::Ok((
                "",
                vec![
                    ("開始日時", "1999/07/15(木) 19:07:12"),
                    ("終了日時", "1999/07/15(木) 19:07:17"),
                    ("手合割", "平手"),
                ]
            ))
        );
    }

    #[test]
    fn parse_board() {
        assert_eq!(
            board(
                &r"
  ９ ８ ７ ６ ５ ４ ３ ２ １
+---------------------------+
|v香v桂v銀v金v玉v金v銀v桂v香|一
| ・v飛 ・ ・ ・ ・ ・v角 ・|二
|v歩v歩v歩v歩v歩v歩v歩v歩v歩|三
| ・ ・ ・ ・ ・ ・ ・ ・ ・|四
| ・ ・ ・ ・ ・ ・ ・ ・ ・|五
| ・ ・ ・ ・ ・ ・ ・ ・ ・|六
| 歩 歩 歩 歩 歩 歩 歩 歩 歩|七
| ・ 角 ・ ・ ・ ・ ・ 飛 ・|八
| 香 桂 銀 金 玉 金 銀 桂 香|九
+---------------------------+
"[1..]
            ),
            Result::Ok(("", Board::default()))
        );
    }

    #[test]
    fn parse_moves_1() {
        assert_eq!(
            moves(
                &r"
1 ７六歩(77) ( 0:16/00:00:16)
2 ３四歩(33) ( 0:00/00:00:00)
3 中断 ( 0:03/ 0:00:19)
"[1..]
            ),
            Result::Ok((
                "",
                vec![
                    Move {
                        action: Action::Move(Black, Square::new(7, 7), Square::new(7, 6), Pawn),
                        time: None,
                        comments: Vec::new(),
                    },
                    Move {
                        action: Action::Move(White, Square::new(3, 3), Square::new(3, 4), Pawn),
                        time: None,
                        comments: Vec::new(),
                    },
                    Move {
                        action: Action::Chudan,
                        time: None,
                        comments: Vec::new(),
                    }
                ]
            ))
        );
    }

    #[test]
    fn parse_moves_2() {
        assert_eq!(
            moves(
                &r"
   1 ２二金打     ( 0:00/00:00:00) ( 0:00/00:00:00)
   2 同　玉(12)   ( 0:00/00:00:00) ( 0:00/00:00:00)
   3 ３二飛打     ( 0:00/00:00:00) ( 0:00/00:00:00)
   4 同　玉(22)   ( 0:00/00:00:00) ( 0:00/00:00:00)
   5 ４二龍(53)   ( 0:00/00:00:00) ( 0:00/00:00:00)
*正解図
変化：1手
   1 ３二飛打     ( 0:00/00:00:00) ( 0:00/00:00:00)
   2 ２二香打     ( 0:00/00:00:00) ( 0:00/00:00:00)
   3 同　飛成(32) ( 0:00/00:00:00) ( 0:00/00:00:00)
   4 同　玉(12)   ( 0:00/00:00:00) ( 0:00/00:00:00)
   5 ３二金打     ( 0:00/00:00:00) ( 0:00/00:00:00)
   6 １二玉(22)   ( 0:00/00:00:00) ( 0:00/00:00:00)
*失敗図
"[1..]
            ),
            Ok((
                "",
                vec![
                    Move {
                        action: Action::Move(Black, Square::new(0, 0), Square::new(2, 2), Gold,),
                        time: None,
                        comments: Vec::new(),
                    },
                    Move {
                        action: Action::Move(White, Square::new(1, 2), Square::new(2, 2), King,),
                        time: None,
                        comments: Vec::new(),
                    },
                    Move {
                        action: Action::Move(Black, Square::new(0, 0), Square::new(3, 2), Rook,),
                        time: None,
                        comments: Vec::new(),
                    },
                    Move {
                        action: Action::Move(White, Square::new(2, 2), Square::new(3, 2), King,),
                        time: None,
                        comments: Vec::new(),
                    },
                    Move {
                        action: Action::Move(Black, Square::new(5, 3), Square::new(4, 2), Dragon,),
                        time: None,
                        comments: vec![String::from("正解図")],
                    },
                ]
            ))
        );
    }

    #[test]
    fn parse_record() {
        assert_eq!(
            record(
                &r"
# ---- Kifu for Windows95 V3.53 棋譜ファイル ----
開始日時：1999/07/15(木) 19:07:12
終了日時：1999/07/15(木) 19:07:17
手合割：平手
先手：先手の対局者名
後手：後手の対局者名
手数----指手---------消費時間-- # この行は、なくてもいい
1 ７六歩(77) ( 0:16/00:00:16)
2 ３四歩(33) ( 0:00/00:00:00)
3 中断 ( 0:03/ 0:00:19)
"[1..],
            ),
            Result::Ok((
                "",
                Record {
                    pos: Position::default(),
                    moves: vec![
                        Move {
                            action: Action::Move(Black, Square::new(7, 7), Square::new(7, 6), Pawn),
                            time: None,
                            comments: Vec::new(),
                        },
                        Move {
                            action: Action::Move(White, Square::new(3, 3), Square::new(3, 4), Pawn),
                            time: None,
                            comments: Vec::new(),
                        },
                        Move {
                            action: Action::Chudan,
                            time: None,
                            comments: Vec::new(),
                        }
                    ]
                }
            ))
        );
    }

    #[test]
    fn parse_record_tsume() {
        assert_eq!(
            record(
                &r"
# ---- 柿木将棋形式棋譜ファイル
# ----   generated by IS-SHOGI
対局日：2019/04/04(木) 19:25:06
終了日時：2019/04/04(木) 19:25:06
手合割：詰将棋　
後手の持駒：飛　角二　金二　銀二　桂三　香三　歩十七　
  ９ ８ ７ ６ ５ ４ ３ ２ １
+---------------------------+
| ・ ・ ・ ・ ・ ・ ・v桂v香|一
| ・ ・ ・ ・ ・ ・ 銀 ・v玉|二
| ・ ・ ・ ・ ・ ・v歩 ・v金|三
| ・ ・ ・ ・ ・ 龍 ・ ・ ・|四
| ・ ・ ・ ・ ・ ・ ・ ・ ・|五
| ・ ・ ・ ・ ・ ・ ・ ・ ・|六
| ・ ・ ・ ・ ・ ・ ・ ・ ・|七
| ・ ・ ・ ・ ・ ・ ・ ・ ・|八
| ・ ・ ・ ・ ・ ・ ・ ・ ・|九
+---------------------------+
先手の持駒：金　銀　
先手番
先手：
後手：
手数----指手---------消費時間--
   1 ２一銀(32)   ( 0:00/00:00:00) ( 0:00/00:00:00)
   2 同　玉(12)   ( 0:00/00:00:00) ( 0:00/00:00:00)
   3 ４一龍(44)   ( 0:00/00:00:00) ( 0:00/00:00:00)
   4 ３一金打     ( 0:00/00:00:00) ( 0:00/00:00:00)
   5 ３二銀打     ( 0:00/00:00:00) ( 0:00/00:00:00)
   6 １二玉(21)   ( 0:00/00:00:00) ( 0:00/00:00:00)
*失敗図
変化：1手
   1 ２三銀打     ( 0:00/00:00:00) ( 0:00/00:00:00)
   2 同　金(13)   ( 0:00/00:00:00) ( 0:00/00:00:00)
   3 １四龍(44)   ( 0:00/00:00:00) ( 0:00/00:00:00)
   4 同　金(23)   ( 0:00/00:00:00) ( 0:00/00:00:00)
   5 ２三金打     ( 0:00/00:00:00) ( 0:00/00:00:00)
*正解図
#--separator--
"[1..]
            ),
            Ok((
                "",
                Record {
                    pos: Position {
                        drop_pieces: Vec::new(),
                        board: Board([
                            [
                                None,
                                None,
                                None,
                                None,
                                None,
                                None,
                                None,
                                Some((White, Knight)),
                                Some((White, Lance)),
                            ],
                            [
                                None,
                                None,
                                None,
                                None,
                                None,
                                None,
                                Some((Black, Silver)),
                                None,
                                Some((White, King)),
                            ],
                            [
                                None,
                                None,
                                None,
                                None,
                                None,
                                None,
                                Some((White, Pawn)),
                                None,
                                Some((White, Gold)),
                            ],
                            [
                                None,
                                None,
                                None,
                                None,
                                None,
                                Some((Black, Dragon)),
                                None,
                                None,
                                None
                            ],
                            [None, None, None, None, None, None, None, None, None],
                            [None, None, None, None, None, None, None, None, None],
                            [None, None, None, None, None, None, None, None, None],
                            [None, None, None, None, None, None, None, None, None],
                            [None, None, None, None, None, None, None, None, None]
                        ]),
                        hand: Hand([[0, 0, 0, 1, 1, 0, 0], [17, 3, 3, 2, 2, 2, 1]]),
                        side_to_move: Black,
                    },
                    moves: vec![
                        Move {
                            action: Action::Move(
                                Black,
                                Square::new(3, 2),
                                Square::new(2, 1),
                                Silver,
                            ),
                            time: None,
                            comments: Vec::new(),
                        },
                        Move {
                            action: Action::Move(White, Square::new(1, 2), Square::new(2, 1), King,),
                            time: None,
                            comments: Vec::new(),
                        },
                        Move {
                            action: Action::Move(
                                Black,
                                Square::new(4, 4),
                                Square::new(4, 1),
                                Dragon,
                            ),
                            time: None,
                            comments: Vec::new(),
                        },
                        Move {
                            action: Action::Move(White, Square::new(0, 0), Square::new(3, 1), Gold,),
                            time: None,
                            comments: Vec::new(),
                        },
                        Move {
                            action: Action::Move(
                                Black,
                                Square::new(0, 0),
                                Square::new(3, 2),
                                Silver,
                            ),
                            time: None,
                            comments: Vec::new(),
                        },
                        Move {
                            action: Action::Move(White, Square::new(2, 1), Square::new(1, 2), King,),
                            time: None,
                            comments: vec![String::from("失敗図")],
                        },
                    ]
                }
            ))
        );
    }
}
