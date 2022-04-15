use dfpn_solver::{generate_legal_moves, Node, Solver, Table, DFPN, INF};
use std::collections::HashSet;
use yasai::{Move, MoveType, PieceType, Position};

pub fn pos2pos(pos: &shogi::Position) -> yasai::Position {
    let board = yasai::Square::ALL.map(|sq| {
        pos.piece_at(shogi::Square::from_index(sq.index() as u8).unwrap())
            .map(|p| {
                let color = match p.color {
                    shogi::Color::Black => yasai::Color::Black,
                    shogi::Color::White => yasai::Color::White,
                };
                let piece_type = match p.piece_type {
                    shogi::PieceType::King => yasai::PieceType::OU,
                    shogi::PieceType::Rook => yasai::PieceType::HI,
                    shogi::PieceType::Bishop => yasai::PieceType::KA,
                    shogi::PieceType::Gold => yasai::PieceType::KI,
                    shogi::PieceType::Silver => yasai::PieceType::GI,
                    shogi::PieceType::Knight => yasai::PieceType::KE,
                    shogi::PieceType::Lance => yasai::PieceType::KY,
                    shogi::PieceType::Pawn => yasai::PieceType::FU,
                    shogi::PieceType::ProRook => yasai::PieceType::RY,
                    shogi::PieceType::ProBishop => yasai::PieceType::UM,
                    shogi::PieceType::ProSilver => yasai::PieceType::NG,
                    shogi::PieceType::ProKnight => yasai::PieceType::NK,
                    shogi::PieceType::ProLance => yasai::PieceType::NY,
                    shogi::PieceType::ProPawn => yasai::PieceType::TO,
                };
                yasai::Piece::from_cp(color, piece_type)
            })
    });
    let mut hand_nums = [[0; yasai::PieceType::NUM_HAND]; yasai::Color::NUM];
    for c in yasai::Color::ALL {
        for (i, &pt) in yasai::PieceType::ALL_HAND.iter().enumerate() {
            let piece_type = match pt {
                yasai::PieceType::FU => shogi::PieceType::Pawn,
                yasai::PieceType::KY => shogi::PieceType::Lance,
                yasai::PieceType::KE => shogi::PieceType::Knight,
                yasai::PieceType::GI => shogi::PieceType::Silver,
                yasai::PieceType::KI => shogi::PieceType::Gold,
                yasai::PieceType::KA => shogi::PieceType::Bishop,
                yasai::PieceType::HI => shogi::PieceType::Rook,
                _ => unreachable!(),
            };
            let color = match c {
                yasai::Color::Black => shogi::Color::Black,
                yasai::Color::White => shogi::Color::White,
            };
            hand_nums[c.index()][i] = pos.hand(shogi::Piece { piece_type, color });
        }
    }
    let side_to_move = match pos.side_to_move() {
        shogi::Color::Black => yasai::Color::Black,
        shogi::Color::White => yasai::Color::White,
    };
    let ply = pos.ply() as u32;
    yasai::Position::new(board, hand_nums, side_to_move, ply)
}

pub fn solve(pos: Position) -> Vec<Move> {
    let mut solver: Solver = Solver::default();
    solver.dfpn(pos);
    let (mut pos, table) = (solver.pos, solver.table);
    let mut solutions = Vec::new();
    search_all_mates(
        &mut pos,
        &table,
        &mut Vec::new(),
        &mut HashSet::new(),
        &mut solutions,
    );
    solutions.sort_by_cached_key(|&(_, score)| score);
    solutions.dedup();
    solutions
        .last()
        .map_or(Vec::new(), |(moves, _)| moves.clone())
}

fn search_all_mates<T>(
    pos: &mut Position,
    table: &T,
    moves: &mut Vec<Move>,
    hashes: &mut HashSet<u64>,
    solutions: &mut Vec<(Vec<Move>, usize)>,
) where
    T: Table,
{
    let node = if moves.len() & 1 == 0 {
        Node::Or
    } else {
        Node::And
    };
    let mate_pd = match node {
        Node::Or => (INF, 0),
        Node::And => (0, INF),
    };
    let mate_moves = generate_legal_moves(pos, node)
        .into_iter()
        .filter(|(_, h)| !hashes.contains(h) && table.look_up_hash(h) == mate_pd)
        .collect::<Vec<_>>();
    if mate_moves.is_empty() {
        solutions.push(calculate_result_and_score(pos, moves));
    } else {
        for &(m, h) in &mate_moves {
            moves.push(m);
            hashes.insert(h);
            pos.do_move(m);
            search_all_mates(pos, table, moves, hashes, solutions);
            pos.undo_move(m);
            moves.pop();
            hashes.remove(&h);
        }
    }
}

fn calculate_result_and_score(pos: &Position, moves: &[Move]) -> (Vec<Move>, usize) {
    let mut moves = moves.to_vec();
    let mut total_hands = PieceType::ALL_HAND
        .map(|pt| pos.hand(!pos.side_to_move()).num(pt))
        .iter()
        .sum::<u8>();
    // 最終2手が「合駒→同」の場合は、合駒無効の詰みなので削除
    while moves.len() > 2 {
        if let (
            MoveType::Drop {
                to: drop_to,
                piece: _,
            },
            MoveType::Normal {
                from: _,
                to: move_to,
                is_promotion: _,
                piece: _,
            },
        ) = (
            moves[moves.len() - 2].move_type(),
            moves[moves.len() - 1].move_type(),
        ) {
            if drop_to == move_to {
                moves.pop();
                moves.pop();
                total_hands -= 1;
                continue;
            }
        }
        break;
    }
    // 1. 玉方が合駒として打った駒が後に取られて
    // 2. 最終的に攻方の持駒に入っている
    // を満たす場合、無駄合駒とみなす
    let mut drops = vec![None; 81];
    for (i, &m) in moves.iter().enumerate() {
        match i & 1 {
            0 => {
                if let MoveType::Normal {
                    from: _,
                    to,
                    is_promotion: _,
                    piece: _,
                } = m.move_type()
                {
                    if let Some(piece_type) = drops[to.index()].take() {
                        if pos.hand(!pos.side_to_move()).num(piece_type) > 0 {
                            // TODO: 候補から除外したいが このパターンだけが候補になる場合もある
                            return (moves, 0);
                        }
                    }
                }
            }
            1 => {
                if let MoveType::Drop { to, piece } = m.move_type() {
                    drops[to.index()] = Some(piece.piece_type());
                }
            }
            _ => {}
        }
    }
    let score = moves.len() * 100 - total_hands as usize;
    (moves, score)
}

#[cfg(test)]
mod tests {
    use super::solve;
    use crate::pos2pos;
    use yasai::{Move, Position};

    fn pos_from_sfen(sfen: &str) -> Position {
        let mut pos = shogi::Position::new();
        pos.set_sfen(sfen).expect("failed to parse SFEN string");
        pos2pos(&pos)
    }

    fn is_valid_moves(sfen: &str, moves: &[Move]) -> bool {
        if moves.len() % 2 == 0 {
            return false;
        }
        let mut pos = pos_from_sfen(sfen);
        for (i, &m) in moves.iter().enumerate() {
            if pos.in_check() == (i % 2 == 0) {
                return false;
            }
            pos.do_move(m);
        }
        is_mated(&mut pos)
    }

    fn is_mated(pos: &mut Position) -> bool {
        if !pos.in_check() {
            return false;
        }
        // TODO: 合駒無効の詰みの場合もある
        for m in pos.legal_moves() {
            if !m.is_drop() {
                return false;
            }
        }
        true
    }

    #[test]
    fn solve_mates() {
        // https://yaneuraou.yaneu.com/2020/12/25/christmas-present/
        let test_cases = vec![
            // head -10 mate3.sfen
            "ln1gkg1nl/6+P2/2sppps1p/2p3p2/p8/P1P1P3P/2NP1PP2/3s1KSR1/L1+b2G1NL w R2Pbgp 42",
            "l3kgsnl/9/p1pS+Bp3/7pp/6PP1/9/PPPPPPn1P/1B1GG2+r1/LNS1K3L w RG3Psnp 54",
            "l3k2nl/4g1gb1/1+S1pspp+P1/p1p6/3n4p/2PPR1P2/P2bPP2P/5GS2/LN1K4L w R2Pgsn2p 50",
            "lns+R4l/1p1p5/p1pkppB1p/6p2/1R7/6P1P/P1PPnPS2/2+b1G1g2/L3K1sNL b 2GS3Pnp 51",
            "1+P1gkg2l/2s3s+P1/3ppp2p/P1p2npp1/l2N1+b3/3KP1P2/N2P1PS1P/2+p1G2R1/L1+r3sNL w Pbgp 58",
            "lnsG5/4g4/prpp1p1pp/1p4p2/4+B3k/2P1P4/P+b1PSP1LP/4K2SL/2G2G1r1 b SP3nl3p 71",
            "l5+R1l/4kS3/p4pnpp/2Pppb3/6p1P/P2s5/NP2+nPPR1/2+bS2GK1/L6NL b 3GSP4p 93",
            "lR5nl/5k1b1/2gp3p1/2s1p1P2/p4N2p/P3PpR2/1PPP1P2P/2G1K2s1/LN6L b GSN2Pbgs2p 83",
            "l1+R5l/2pS5/p2pp+P1pp/2k3p2/2N4P1/PP2R1P1P/2+pPP1N2/2GSG1bs1/LN1K4L b 2GSNPbp 73",
            "lnsg4l/1r1b5/p1pp1+N1+R1/4p3p/9/P3SSk2/NpPPPPg1P/2GK5/L1S4NL b 2Pbg4p 91",
            // head -10 mate5.sfen
            "l2gkg2l/2s3s2/p1nppp1pp/2p3p2/P4P1P1/4n3P/1PPPG1N2/1BKS2+s2/LN3+r3 w RBgl3p 72",
            "lnsgs2+Pl/3kg4/p1pppN2p/6pp1/9/7R1/P1PP1Sg1P/1S3+b3/LN5KL w Nrbg6p 58",
            "lnG4nl/5k3/p1p+R1g1p1/1p1p3sp/5N3/2P1p1p2/PP1GP3P/1SG2+p1+b1/LN1K4L w Srbs4p 60",
            "ln4knl/4+N2b1/4ppsG1/p1P5p/2G3pp1/3P1P2P/P2+pP1P2/2+srSK3/L+r3G1NL w G4Pbs 78",
            "ln5+Pl/3s1kg+R1/p2ppl2p/2ps1Bp2/P8/2P3P1P/N2gP4/5KS2/L+r3G1N+b b GS3Pn3p 57",
            "l3k3l/1r1sg1B2/3p2+R1p/2p1pN2P/9/pPP1PP3/3PK1P2/2G1sg3/LNS2+n1NL b 5Pbgsp 69",
            "ln1g4l/2s2+R3/2kp2G2/p1r2pp1p/4S4/Pp1Gp1P2/BP1P4P/2GKs1+bP1/LNN5L w SN4Pp 88",
            "ln1+P1GBnl/s8/p1p1p1kpp/3P2p2/5p3/Pp3PPP1/1P1SP3P/2R6/1N1GKG1NL b BGLr2sp 67",
            "ln1s3nl/1+S4+B2/p1p1k2pp/3p2P2/7P1/P1Pnpp1R1/1P1P+lP2P/2G1G4/L1SKP2N+b b R2GPsp 73",
            "lnB2k1+Pl/4g1g2/p1pp2ppp/5r3/3s2P1P/NpPnPP3/P5Sb1/3S5/LNG1KG2L w rs4p 74",
            // head -10 mate7.sfen
            "ln1g3+Rl/2sk1s+P2/2ppppb1p/p1b3p2/8P/P4P3/2PPP1P2/1+r2GS3/LN+p2KGNL w GN2Ps 36",
            "ln1g2B+Rl/2s6/pPppppk2/6p1p/9/4P1P1P/P1PPSP3/3+psK3/L+r3G1NL b SNb2gn2p 39",
            "ln+P3s+Pl/2+R1Gsk2/p3pp1g1/4r1ppp/1NS6/6P2/PP1+bPPS1P/3+p1K3/LG3G1NL w Nb3p 72",
            "lnsgk2+Pl/6+N2/p1pp2p1p/4p2R1/9/2P3P2/P2PPPN1P/4s1g1K/L4+r2L w 2B2SN4P2g 56",
            "l+P1g2+S1l/2sk5/p1ppppngp/6p2/9/6P2/P1+bPPP2P/2+r2S3/+rN2GK1NL w SNbgl4p 56",
            "l2R2snl/4gkg2/p+P1ppp2p/2p3pp1/9/1nPPP4/P1G1GPP1P/3K1Ss2/+r3Bb1NL w N2Psl 68",
            "l6nl/3k2+B2/p1n1g2pp/2G1ppp2/2P2N1P1/3P2P1P/Ps1GP4/1+rSK2R2/LN6L b G3Pb2s2p 77",
            "+N5snl/4+N1gp1/1b1p1pkP1/1s1l2pLp/4p+b3/P1P6/1P1PPPP1P/2+rSK2L1/2+r1S1GN1 w 2P2gp 84",
            // TODO: "ln3kgRl/2s1g2p1/2ppppn1p/p5p2/6b2/P3P4/1+rPP1PP1P/1P4S2/LNSK1G1NL w GPbsp 50",
            "3g4l/+R1sg2S2/p1npk1s+Rp/2pb2p2/4g2N1/1p7/P1PP1PP1P/1P1S5/LNK2G1+lL b N3Pb2p 71",
        ];
        for (i, &sfen) in test_cases.iter().enumerate() {
            let ret = solve(pos_from_sfen(sfen));
            assert!(is_valid_moves(sfen, &ret), "failed to solve #{}", i);
        }
    }

    #[test]
    fn ghi_problems() {
        let test_cases = vec![
            "ln1gkg1nl/6+P2/2sppps1p/2p3p2/p8/P1P1P3P/2NP1PP2/3s1KSR1/L1+b2G1NL w R2Pbgp 42", // https://yaneuraou.yaneu.com/2020/12/25/christmas-present/ mate3.sfen:1
            "3Bp2n1/5+R2+B/p2p1GSp1/8p/Pn5l1/1n2SNP2/2pPPS1Pk/1P1SK1G2/L1G1G4 b RL3Pl3p 131", // https://yaneuraou.yaneu.com/2020/12/25/christmas-present/ mate7.sfen:71
            "7+P1/5R1s1/6ks1/9/5L1p1/9/9/9/9 b R2b4g2s4n3l16p 1", // https://www.shogi.or.jp/tsume_shogi/everyday/20211183_1.html
        ];
        for (i, &sfen) in test_cases.iter().enumerate() {
            let ret = solve(pos_from_sfen(sfen));
            assert!(is_valid_moves(sfen, &ret), "failed to solve #{}", i);
        }
    }

    #[test]
    fn other_problems() {
        let test_cases = vec![
            "ln1g3k1/5G2l/1+LspSp2p/2p1S2p1/2r3p2/p3P4/1P+BP1P+b1P/2GS5/L2K1G3 b NPr2n5p 79", // https://yaneuraou.yaneu.com/2020/12/25/christmas-present/ mate3.sfen:569
        ];
        for (i, &sfen) in test_cases.iter().enumerate() {
            let pos = pos_from_sfen(sfen);
            let ret = solve(pos);
            assert!(is_valid_moves(sfen, &ret), "failed to solve #{}", i);
        }
    }

    #[test]
    fn 無駄合駒() {
        let test_cases = vec![
            "7nl/5B1k1/6Ppp/5+R3/9/9/9/9/9 b Srb4g3s3n3l15p 1", // issues/5,
        ];
        for (i, &sfen) in test_cases.iter().enumerate() {
            let ret = solve(pos_from_sfen(sfen));
            assert!(is_valid_moves(sfen, &ret), "failed to solve #{}", i);
        }
    }
}
