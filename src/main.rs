use clap::{App, Arg};
use csa::parse_csa;
use dfpn_solver::solve;
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

        match parse_csa(&content) {
            Ok(record) => {
                println!("{:?}", solve(&record.start_pos));
            }
            Err(err) => panic!("{}", err),
        }
    }
    Ok(())
}
