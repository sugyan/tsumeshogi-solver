#![feature(test)]
extern crate test;
use dfpn_solver::impl_default_hash::DefaultHashPosition;
use dfpn_solver::impl_hashmap_table::HashMapTable;
use dfpn_solver::impl_vec_table::VecTable;
use dfpn_solver::impl_zobrist_hash::ZobristHashPosition;
use dfpn_solver::Solver;
use shogi::bitboard::Factory;
use shogi::Position;

fn test_cases() -> Vec<String> {
    vec![
        "lns+R4l/1p1p5/p1pkppB1p/6p2/1R7/6P1P/P1PPnPS2/2+b1G1g2/L3K1sNL b 2GS3Pnp 51",
        "lnsG5/4g4/prpp1p1pp/1p4p2/4+B3k/2P1P4/P+b1PSP1LP/4K2SL/2G2G1r1 b SP3nl3p 71",
        "l5+R1l/4kS3/p4pnpp/2Pppb3/6p1P/P2s5/NP2+nPPR1/2+bS2GK1/L6NL b 3GSP4p 93",
        // "lR5nl/5k1b1/2gp3p1/2s1p1P2/p4N2p/P3PpR2/1PPP1P2P/2G1K2s1/LN6L b GSN2Pbgs2p 83",
        // "l1+R5l/2pS5/p2pp+P1pp/2k3p2/2N4P1/PP2R1P1P/2+pPP1N2/2GSG1bs1/LN1K4L b 2GSNPbp 73",
        // "lnsg4l/1r1b5/p1pp1+N1+R1/4p3p/9/P3SSk2/NpPPPPg1P/2GK5/L1S4NL b 2Pbg4p 91",
        // "l3k2G1/1+B4gPl/n2+Nppsp1/pP2R2bp/9/Pps1P1N1P/2GG1P3/3S5/LNK5L b R6Ps 97",
        // "lnkgp1+R1l/1rs4+P1/p1ppG2p1/4N3p/3S5/P7P/2+lPP4/2G1KP3/L1S4+b1 b BN2Pgsn4p 83",
        // "3g2S1l/3s2k2/3ppplpp/2p3R2/rP7/1LP1P2P1/N2P1P2P/2GSG4/3KN2NL b BG4Pbsnp 89",
        // "l1G1k2nl/2Rs2+R2/pp2bp2p/4p1p2/1n1p1N2P/4P1P2/PPG1SP3/2p1G4/LN1K1s2L b 3Pbgsp 85",
    ]
    .into_iter()
    .map(String::from)
    .collect()
}

#[bench]
fn bench_default_hashmap(b: &mut test::Bencher) {
    Factory::init();

    b.iter(|| {
        for sfen in test_cases() {
            let mut pos = Position::new();
            pos.set_sfen(&sfen).expect("failed to parse SFEN string");
            Solver::new(DefaultHashPosition::new(pos), HashMapTable::new()).dfpn();
        }
    });
}

#[bench]
fn bench_zobrist_hashmap(b: &mut test::Bencher) {
    Factory::init();

    b.iter(|| {
        for sfen in test_cases() {
            let mut pos = Position::new();
            pos.set_sfen(&sfen).expect("failed to parse SFEN string");
            Solver::new(ZobristHashPosition::new(pos), HashMapTable::<u64>::new()).dfpn();
        }
    });
}

#[bench]
fn bench_zobrist_vec(b: &mut test::Bencher) {
    Factory::init();

    b.iter(|| {
        for sfen in test_cases() {
            let mut pos = Position::new();
            pos.set_sfen(&sfen).expect("failed to parse SFEN string");
            Solver::new(ZobristHashPosition::new(pos), VecTable::new(16)).dfpn();
        }
    });
}