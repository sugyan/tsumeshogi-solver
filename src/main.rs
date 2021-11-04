use clap::{App, Arg};
use csa::parse_csa;
use dfpn_solver::solve;
use shogi::Position;
use shogi_converter::Record;
use std::{fs::File, io::Read};

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
        let mut pos = Position::new();
        pos.set_sfen(&sfen).expect("failed to parse SFEN string");
        println!(
            "{:?}",
            solve(&pos)
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>()
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use dfpn_solver::{HashMapTable, NaiveHashPosition, Solver};
    use shogi::bitboard::Factory;
    use shogi::Position;

    #[test]
    fn test_impl() {
        Factory::init();

        let mut pos = Position::new();
        pos.set_sfen("3sks3/9/4S4/9/1+B7/9/9/9/9 b S2rb4g4n4l18p 1")
            .expect("failed to parse SFEN string");

        let mut solver = Solver::new(NaiveHashPosition::from(&pos), HashMapTable::<u64>::new());
        solver.dfpn();

        let mut moves = Vec::new();
        solver.search_mate(&mut moves);
        assert_eq!(
            vec!["8e5b", "4a5b", "S*4b"],
            moves.into_iter().map(|m| m.to_string()).collect::<Vec<_>>()
        );
    }
}
