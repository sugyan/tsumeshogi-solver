use clap::{App, Arg};
use csa::parse_csa;
use dfpn_solver::Solver;
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

        let mut solver = Solver::new();
        println!("{}", solver.solve(&mut pos));
    }
    Ok(())
}
