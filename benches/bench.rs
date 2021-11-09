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
