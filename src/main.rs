use clap::{App, Arg};
use csa::parse_csa;
use dfpn_solver::impl_default_hash::DefaultHashPosition;
use dfpn_solver::impl_hashmap_table::HashMapTable;
use dfpn_solver::{generate_legal_moves, HashPosition, Solver, Table, INF};
use shogi::{bitboard::Factory, Color, Move, Position};
use shogi_converter::Record;
use std::{cmp::Reverse, fs::File, io::Read};

fn main() -> Result<(), std::io::Error> {
    let matches = App::new("Tsumeshogi Solver")
        .version("0.1")
        .arg(
            Arg::with_name("INPUT")
                .help("Input filepath")
                .required(true),
        )
        .get_matches();
    if let Some(input) = matches.value_of("INPUT") {
        let mut file = File::open(input)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        let csa = parse_csa(&content).expect("failed to parse CSA string");
        let sfen = Record::from(csa).to_sfen();

        Factory::init();

        let mut pos = Position::new();
        pos.set_sfen(&sfen).expect("failed to parse SFEN string");
        println!(
            "{:?}",
            solve(pos).iter().map(|p| p.to_string()).collect::<Vec<_>>()
        );
    }
    Ok(())
}

fn solve(pos: Position) -> Vec<Move> {
    let mut solver = Solver::new(DefaultHashPosition::new(pos), HashMapTable::new());
    solver.dfpn();

    let mut moves = Vec::new();
    search_mate(&mut solver, &mut moves);
    moves
}

fn search_mate<HP, T>(s: &mut Solver<HP, T>, moves: &mut Vec<Move>)
where
    HP: HashPosition,
    T: Table<T = HP::T>,
{
    let mut v = generate_legal_moves(&mut s.hp)
        .iter()
        .map(|&(m, h)| (m, s.t.look_up_hash(&h)))
        .collect::<Vec<_>>();
    v.sort_by_cached_key(|&(_, (p, d))| match s.hp.side_to_move() {
        Color::Black => (Reverse(p), d),
        Color::White => (Reverse(d), p),
    });
    for &(m, (p, d)) in &v {
        if (s.hp.side_to_move() == Color::Black && p == INF)
            || (s.hp.side_to_move() == Color::White && d == INF)
        {
            moves.push(m);
            s.hp.make_move(m).expect("failed to make move");
            search_mate(s, moves);
            s.hp.unmake_move().expect("failed to unmake move");
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::solve;
    use shogi::bitboard::Factory;
    use shogi::Position;

    #[test]
    fn test_solve() {
        Factory::init();

        let test_cases = vec![
            "lns+R4l/1p1p5/p1pkppB1p/6p2/1R7/6P1P/P1PPnPS2/2+b1G1g2/L3K1sNL b 2GS3Pnp 51",
            "lnsG5/4g4/prpp1p1pp/1p4p2/4+B3k/2P1P4/P+b1PSP1LP/4K2SL/2G2G1r1 b SP3nl3p 71",
            "l5+R1l/4kS3/p4pnpp/2Pppb3/6p1P/P2s5/NP2+nPPR1/2+bS2GK1/L6NL b 3GSP4p 93",
            "lR5nl/5k1b1/2gp3p1/2s1p1P2/p4N2p/P3PpR2/1PPP1P2P/2G1K2s1/LN6L b GSN2Pbgs2p 83",
            "l1+R5l/2pS5/p2pp+P1pp/2k3p2/2N4P1/PP2R1P1P/2+pPP1N2/2GSG1bs1/LN1K4L b 2GSNPbp 73",
            "lnsg4l/1r1b5/p1pp1+N1+R1/4p3p/9/P3SSk2/NpPPPPg1P/2GK5/L1S4NL b 2Pbg4p 91",
            "l3k2G1/1+B4gPl/n2+Nppsp1/pP2R2bp/9/Pps1P1N1P/2GG1P3/3S5/LNK5L b R6Ps 97",
            "lnkgp1+R1l/1rs4+P1/p1ppG2p1/4N3p/3S5/P7P/2+lPP4/2G1KP3/L1S4+b1 b BN2Pgsn4p 83",
            "3g2S1l/3s2k2/3ppplpp/2p3R2/rP7/1LP1P2P1/N2P1P2P/2GSG4/3KN2NL b BG4Pbsnp 89",
            "l1G1k2nl/2Rs2+R2/pp2bp2p/4p1p2/1n1p1N2P/4P1P2/PPG1SP3/2p1G4/LN1K1s2L b 3Pbgsp 85",
        ];
        for &sfen in &test_cases {
            let mut pos = Position::new();
            pos.set_sfen(sfen).expect("failed to parse SFEN string");

            let ret = solve(pos);
            assert!(ret.len() % 2 == 1);
        }
    }
}
