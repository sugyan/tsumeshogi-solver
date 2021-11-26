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
        "ln1gkg1nl/6+P2/2sppps1p/2p3p2/p8/P1P1P3P/2NP1PP2/3s1KSR1/L1+b2G1NL w R2Pbgp 42",
        "l3kgsnl/9/p1pS+Bp3/7pp/6PP1/9/PPPPPPn1P/1B1GG2+r1/LNS1K3L w RG3Psnp 54",
        "l3k2nl/4g1gb1/1+S1pspp+P1/p1p6/3n4p/2PPR1P2/P2bPP2P/5GS2/LN1K4L w R2Pgsn2p 50",
        "lns+R4l/1p1p5/p1pkppB1p/6p2/1R7/6P1P/P1PPnPS2/2+b1G1g2/L3K1sNL b 2GS3Pnp 51",
        "1+P1gkg2l/2s3s+P1/3ppp2p/P1p2npp1/l2N1+b3/3KP1P2/N2P1PS1P/2+p1G2R1/L1+r3sNL w Pbgp 58",
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
            Solver::new(DefaultHashPosition::default(), HashMapTable::default()).dfpn(pos);
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
            Solver::new(
                ZobristHashPosition::default(),
                HashMapTable::<u64>::default(),
            )
            .dfpn(pos);
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
            Solver::new(ZobristHashPosition::default(), VecTable::new(16)).dfpn(pos);
        }
    });
}
