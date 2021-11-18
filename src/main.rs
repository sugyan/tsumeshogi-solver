use clap::{App, Arg};
use csa::{parse_csa, CsaError};
use dfpn_solver::impl_default_hash::DefaultHashPosition;
use dfpn_solver::impl_hashmap_table::HashMapTable;
use dfpn_solver::{generate_legal_moves, HashPosition, Solver, Table, INF};
use shogi::{bitboard::Factory, Color, Move, Piece, PieceType, Position, SfenError};
use shogi_converter::kif_converter::{parse_kif, KifError};
use shogi_converter::Record;
use std::io::BufRead;
use std::{cmp::Reverse, fs::File, io::Read, str};
use thiserror::Error;

#[derive(Error, Debug)]
enum ParseError {
    #[error("failed to parse csa: {0}")]
    Csa(CsaError),
    #[error("failed to parse kif: {0}")]
    Kif(KifError),
    #[error("failed to parse sfen: {0}")]
    Sfen(SfenError),
    #[error(transparent)]
    Utf8(#[from] std::str::Utf8Error),
}

trait Parse {
    fn parse(&self, input: &[u8]) -> Result<Position, ParseError>;
}

struct CsaParser;

impl Parse for CsaParser {
    fn parse(&self, input: &[u8]) -> Result<Position, ParseError> {
        match parse_csa(str::from_utf8(input).map_err(ParseError::Utf8)?) {
            Ok(record) => {
                let mut pos = Position::new();
                match pos.set_sfen(&Record::from(record).to_sfen()) {
                    Ok(_) => Ok(pos),
                    Err(e) => Err(ParseError::Sfen(e)),
                }
            }
            Err(e) => Err(ParseError::Csa(e)),
        }
    }
}

struct KifParser;

impl Parse for KifParser {
    fn parse(&self, input: &[u8]) -> Result<Position, ParseError> {
        match parse_kif(input) {
            Ok(record) => {
                let mut pos = Position::new();
                pos.set_sfen(&record.to_sfen())
                    .expect("failed to parse SFEN string");
                Ok(pos)
            }
            Err(e) => Err(ParseError::Kif(e)),
        }
    }
}

fn main() -> Result<(), std::io::Error> {
    let matches = App::new("Tsumeshogi Solver")
        .version("0.1")
        .arg(
            Arg::with_name("v")
                .short("v")
                .long("verbose")
                .help("Verbose mode"),
        )
        .arg(
            Arg::with_name("format")
                .short("f")
                .long("format")
                .help("input format")
                .takes_value(true)
                .possible_values(&["sfen", "csa", "kif"])
                .default_value("sfen"),
        )
        .arg(
            Arg::with_name("INPUT")
                .help("Input files or strings")
                .required(true)
                .multiple(true),
        )
        .get_matches();
    let verbose = matches.is_present("v");
    let inputs = matches.values_of("INPUT").unwrap().collect::<Vec<_>>();

    Factory::init();
    match matches.value_of("format").unwrap() {
        "sfen" => run_sfen(&inputs, verbose),
        "csa" => run_parse(CsaParser, &inputs, verbose),
        "kif" => run_parse(KifParser, &inputs, verbose),
        _ => panic!("unknown format"),
    }
}

fn run_sfen(inputs: &[&str], verbose: bool) -> Result<(), std::io::Error> {
    if inputs == ["-"] {
        let stdin = std::io::stdin();
        for line in stdin.lock().lines() {
            let sfen = line?;
            let mut pos = Position::new();
            match pos.set_sfen(&sfen) {
                Ok(()) => run(pos, &sfen, verbose),
                Err(e) => {
                    eprintln!("failed to parse SFEN string: {}", e);
                    std::process::exit(1);
                }
            }
        }
    } else {
        for &input in inputs {
            let mut pos = Position::new();
            match pos.set_sfen(input) {
                Ok(()) => run(pos, input.trim(), verbose),
                Err(e) => {
                    eprintln!("failed to parse SFEN string: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
    Ok(())
}

fn run_parse<T>(parser: T, inputs: &[&str], verbose: bool) -> Result<(), std::io::Error>
where
    T: Parse,
{
    if inputs == ["-"] {
        let stdin = std::io::stdin();
        let mut buf = Vec::new();
        stdin.lock().read_to_end(&mut buf)?;
        match parser.parse(&buf) {
            Ok(pos) => run(pos, "-", verbose),
            Err(e) => {
                eprintln!("failed to parse input: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        for &input in inputs {
            let mut buf = Vec::new();
            let mut file = match File::open(input) {
                Ok(file) => file,
                Err(e) => {
                    eprintln!("{}: {}", e, input);
                    std::process::exit(1);
                }
            };
            file.read_to_end(&mut buf)?;
            match parser.parse(&buf) {
                Ok(pos) => run(pos, input, verbose),
                Err(e) => {
                    eprintln!("failed to parse input {}: {}", input, e);
                    std::process::exit(1);
                }
            }
        }
    }
    Ok(())
}

fn run(pos: Position, input: &str, verbose: bool) {
    print!("{}: ", input);
    if verbose {
        println!();
        println!("{}", pos);
        println!();
    }
    println!(
        "{:?}",
        solve(pos).iter().map(|m| m.to_string()).collect::<Vec<_>>()
    );
}

fn solve(pos: Position) -> Vec<Move> {
    let mut solver = Solver::new(DefaultHashPosition::new(pos), HashMapTable::new());
    solver.dfpn();

    let mut answers = Vec::new();
    search_all_mates(&mut solver, &mut Vec::new(), &mut answers);

    answers.sort_by_cached_key(|(moves, hands)| (Reverse(moves.len()), *hands));
    answers
        .get(0)
        .map_or(Vec::new(), |(moves, _)| moves.clone())
}

fn search_all_mates<P, T>(
    s: &mut Solver<P, T>,
    moves: &mut Vec<Move>,
    answers: &mut Vec<(Vec<Move>, u8)>,
) where
    P: HashPosition,
    T: Table<T = P::T>,
{
    let mut leaf = true;
    for &(m, h) in &generate_legal_moves(&mut s.pos) {
        let pd = s.table.look_up_hash(&h);
        if (s.pos.side_to_move() == Color::Black && pd == (INF, 0))
            || (s.pos.side_to_move() == Color::White && pd == (0, INF))
        {
            leaf = false;
            moves.push(m);
            s.pos.make_move(m).expect("failed to make move");
            search_all_mates(s, moves, answers);
            s.pos.unmake_move().expect("failed to unmake move");
            moves.pop();
        }
    }
    if leaf {
        // 無駄合駒判定
        // 1. 玉方が合駒として打った駒が後に取られて
        // 2. 最終的に攻方の持駒に入っている
        // を満たす場合に解答候補から外す
        let mut drops = vec![None; 81];
        for (i, &m) in moves.iter().enumerate() {
            match i & 1 {
                0 => {
                    if let Move::Normal {
                        from: _,
                        to,
                        promote: _,
                    } = m
                    {
                        if let Some(piece_type) = drops[to.index()].take() {
                            if s.pos.hand(Piece {
                                piece_type,
                                color: s.pos.side_to_move().flip(),
                            }) > 0
                            {
                                return;
                            }
                        }
                    }
                }
                1 => {
                    if let Move::Drop { to, piece_type } = m {
                        drops[to.index()] = Some(piece_type);
                    }
                }
                _ => {}
            }
        }
        answers.push((
            moves.clone(),
            PieceType::iter()
                .filter_map(|piece_type| {
                    if piece_type.is_hand_piece() {
                        Some(s.pos.hand(Piece {
                            piece_type,
                            color: s.pos.side_to_move().flip(),
                        }))
                    } else {
                        None
                    }
                })
                .sum::<u8>(),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::solve;
    use shogi::bitboard::Factory;
    use shogi::{Move, Piece, PieceType, Position, Square};

    fn is_mated(pos: &mut Position) -> bool {
        let color = pos.side_to_move();
        if !pos.in_check(color) {
            return false;
        }
        // all normal moves
        for from in *pos.player_bb(color) {
            let piece = pos.piece_at(from).expect("no piece at square");
            for to in pos.move_candidates(from, piece) {
                for promote in [true, false] {
                    if pos.make_move(Move::Normal { from, to, promote }).is_ok() {
                        return false;
                    }
                }
            }
        }
        // all drop moves
        for piece_type in PieceType::iter().filter(|pt| pt.is_hand_piece()) {
            if pos.hand(Piece { piece_type, color }) == 0 {
                continue;
            }
            for to in Square::iter() {
                if pos.make_move(Move::Drop { to, piece_type }).is_ok() {
                    return false;
                }
            }
        }
        true
    }

    #[test]
    fn test_solve() {
        Factory::init();

        // https://yaneuraou.yaneu.com/2020/12/25/christmas-present/
        let test_cases = vec![
            // mate3
            "lns+R4l/1p1p5/p1pkppB1p/6p2/1R7/6P1P/P1PPnPS2/2+b1G1g2/L3K1sNL b 2GS3Pnp 51",
            "lnsG5/4g4/prpp1p1pp/1p4p2/4+B3k/2P1P4/P+b1PSP1LP/4K2SL/2G2G1r1 b SP3nl3p 71",
            "l5+R1l/4kS3/p4pnpp/2Pppb3/6p1P/P2s5/NP2+nPPR1/2+bS2GK1/L6NL b 3GSP4p 93",
            "lR5nl/5k1b1/2gp3p1/2s1p1P2/p4N2p/P3PpR2/1PPP1P2P/2G1K2s1/LN6L b GSN2Pbgs2p 83",
            "l1+R5l/2pS5/p2pp+P1pp/2k3p2/2N4P1/PP2R1P1P/2+pPP1N2/2GSG1bs1/LN1K4L b 2GSNPbp 73",
            "lnsg4l/1r1b5/p1pp1+N1+R1/4p3p/9/P3SSk2/NpPPPPg1P/2GK5/L1S4NL b 2Pbg4p 91",
            "l3k2G1/1+B4gPl/n2+Nppsp1/pP2R2bp/9/Pps1P1N1P/2GG1P3/3S5/LNK5L b R6Ps 97",
            "lnkgp1+R1l/1rs4+P1/p1ppG2p1/4N3p/3S5/P7P/2+lPP4/2G1KP3/L1S4+b1 b BN2Pgsn4p 83",
            "l1G1k2nl/2Rs2+R2/pp2bp2p/4p1p2/1n1p1N2P/4P1P2/PPG1SP3/2p1G4/LN1K1s2L b 3Pbgsp 85",
            "lR5nl/4gs3/5p1k1/2PpP3p/p4N1P1/6P1P/PP+bP1P3/4K1G2/L1G3S1+r b GSN5Pbsnl 105",
            // mate5
            "ln5+Pl/3s1kg+R1/p2ppl2p/2ps1Bp2/P8/2P3P1P/N2gP4/5KS2/L+r3G1N+b b GS3Pn3p 57",
            "l3k3l/1r1sg1B2/3p2+R1p/2p1pN2P/9/pPP1PP3/3PK1P2/2G1sg3/LNS2+n1NL b 5Pbgsp 69",
            "ln1+P1GBnl/s8/p1p1p1kpp/3P2p2/5p3/Pp3PPP1/1P1SP3P/2R6/1N1GKG1NL b BGLr2sp 67",
            "ln1s3nl/1+S4+B2/p1p1k2pp/3p2P2/7P1/P1Pnpp1R1/1P1P+lP2P/2G1G4/L1SKP2N+b b R2GPsp 73",
            "ln1k1g3/5s3/p1p1pp1pp/3p5/5+Bp2/2P2PPP1/P3S+n2P/2G3+l2/+p3K3L b RBG2SNL2Prgnp 73",
            "ln1G3nl/1ks1n+PR2/p1pp4p/5ppp1/8b/1P2p4/PGPPP+s2P/1B7/LNSK1G2L b RGSP2p 71",
            "l5g1+R/3g1s3/p2p1p1k1/1r2p1Npp/3P+bPL2/1b4P1P/P+n2PS3/4GGL2/LN3K1P1 b SN3Ps2p 103",
            "1n4R1l/2k3GR1/p1sp1pg2/1p2+B1p1p/2P6/P1pL5/1PNsPPP1P/4g1S2/Lb3GKNL b SN4P 99",
            "lnr2k1nl/6gp1/pl2S3p/2p1p1p2/1P1N1P1N1/P2S1pP1P/3P5/1+p1SG2B1/L1PK5 b BGPrgs2p 105",
            "ln1+R3+Pl/5n3/4pp3/p1k5p/bp7/7p1/PP+pPPP+B1P/5K3/LN1G1G2L b R2G2S3P2snp 81",
            // mate7
            "ln1g2B+Rl/2s6/pPppppk2/6p1p/9/4P1P1P/P1PPSP3/3+psK3/L+r3G1NL b SNb2gn2p 39",
            "l6nl/3k2+B2/p1n1g2pp/2G1ppp2/2P2N1P1/3P2P1P/Ps1GP4/1+rSK2R2/LN6L b G3Pb2s2p 77",
            "3g4l/+R1sg2S2/p1npk1s+Rp/2pb2p2/4g2N1/1p7/P1PP1PP1P/1P1S5/LNK2G1+lL b N3Pb2p 71",
            "l1gks1gnl/3s5/p1S1pp3/2+Bp4p/1p2P4/P1PP1P1P1/1P6P/1BKG5/LN4+rNL b GSN4Pr 101",
            "ln2+Rg1nl/5g1k1/p2+B1p1gp/4p2R1/2pK2p2/P3P1P1P/3+b1PN2/5S3/L7L b 2SN7Pgs 81",
            "lnsg1+R2l/k3+P4/pP1p4p/1G2p1p2/1N+B6/P1P1lg3/2+bP1PP1P/1p3S3/K4G1NL b 3Pr2snp 81",
            "1ng2p1S1/4kg2+R/pBp1psnpp/3P2p2/9/2n1PPPP1/PP2l3P/1+p3LG2/6KNL b RGSPbslp 91",
            "2s1kg2l/1p3g1P1/+R1Npps+B1p/3l1lp2/p8/2P1P1P2/PPNP1SN1P/1+bS1G2R1/1+p1K4L b GN3P 91",
            "ln1B1k3/5r3/p2pp+Pgs1/2ps3pl/P4+bp2/1PP2n3/1G1PP2R1/2SK5/LN7 b GLgsn6p 97",
            "l2+R4l/6ks1/3sg2pp/p1pbp1p2/3pbp1P1/2P2PN2/P3G1P1P/4GKSR1/LN5NL b GSPn3p 93",
        ];
        for &sfen in &test_cases {
            let mut pos = Position::new();
            pos.set_sfen(sfen).expect("failed to parse SFEN string");

            let ret = solve(pos);
            assert!(ret.len() % 2 == 1);
            {
                let mut pos = Position::new();
                pos.set_sfen(sfen).expect("failed to parse SFEN string");
                let color = pos.side_to_move().flip();
                for (i, &m) in ret.iter().enumerate() {
                    if i % 2 == 0 {
                        assert!(!pos.in_check(color));
                    } else {
                        assert!(pos.in_check(color));
                    }
                    pos.make_move(m).expect("failed to make move");
                }
                assert!(is_mated(&mut pos));
            }
        }
    }
}
