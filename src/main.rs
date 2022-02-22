use clap::{App, Arg};
use csa::{parse_csa, CsaError};
use shogi::{bitboard::Factory, Position, SfenError};
use shogi_converter::kif_converter::{parse_kif, KifError};
use shogi_converter::Record;
use std::io::BufRead;
use std::{fs::File, io::Read, str};
use thiserror::Error;
use tsumeshogi_solver::solve;

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
                match pos.set_sfen(&Record::from(record).pos.to_sfen()) {
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
                pos.set_sfen(&record.pos.to_sfen())
                    .expect("failed to parse SFEN string");
                Ok(pos)
            }
            Err(e) => Err(ParseError::Kif(e)),
        }
    }
}

fn main() -> Result<(), std::io::Error> {
    let matches = App::new("Tsumeshogi Solver")
        .version("0.2")
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
