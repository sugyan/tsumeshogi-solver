#![feature(test)]
extern crate test;
use dfpn::search::Search;
use dfpn::DefaultSearcher;
use dfpn::{impl_hashmap_table::HashMapTable, impl_vec_table::VecTable};
use tsumeshogi_solver::backend::YasaiPosition;

fn test_cases() -> Vec<String> {
    vec![
        // mate3.sfen
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
        // mate5.sfen
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
    ]
    .into_iter()
    .map(String::from)
    .collect()
}

#[bench]
fn bench_yasai_hashmap(b: &mut test::Bencher) {
    b.iter(|| {
        for sfen in test_cases() {
            let mut searcher =
                DefaultSearcher::<_, HashMapTable>::new(YasaiPosition::from(sfen.as_str()));
            searcher.dfpn_search();
        }
    })
}

#[bench]
fn bench_yasai_vec(b: &mut test::Bencher) {
    b.iter(|| {
        for sfen in test_cases() {
            let mut searcher =
                DefaultSearcher::<_, VecTable>::new(YasaiPosition::from(sfen.as_str()));
            searcher.dfpn_search();
        }
    })
}
