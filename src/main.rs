use clap::{ArgEnum, Parser};
use csa::{parse_csa, CsaError};
use shogi::Position;
use shogi_converter::kif_converter::{parse_kif, KifError};
use shogi_converter::Record;
use std::io::{BufRead, Read};
use std::time::{Duration, Instant};
use std::{fs::File, str};
use thiserror::Error;
use tsumeshogi_solver::solve;

#[derive(Error, Debug)]
enum ParseError {
    #[error("failed to parse csa: {0}")]
    Csa(CsaError),
    #[error("failed to parse kif: {0}")]
    Kif(KifError),
    #[error(transparent)]
    Utf8(#[from] str::Utf8Error),
}

trait Parse {
    fn parse(&self, input: &[u8]) -> Result<String, ParseError>;
}

struct CsaParser;

impl Parse for CsaParser {
    fn parse(&self, input: &[u8]) -> Result<String, ParseError> {
        match parse_csa(str::from_utf8(input).map_err(ParseError::Utf8)?) {
            Ok(record) => Ok(Record::from(record).pos.to_sfen()),
            Err(e) => Err(ParseError::Csa(e)),
        }
    }
}

struct KifParser;

impl Parse for KifParser {
    fn parse(&self, input: &[u8]) -> Result<String, ParseError> {
        match parse_kif(input) {
            Ok(record) => Ok(record.pos.to_sfen()),
            Err(e) => Err(ParseError::Kif(e)),
        }
    }
}

#[derive(Parser)]
#[clap(name = "Tsumeshogi Solver")]
#[clap(version)]
struct Args {
    /// Verbose mode
    #[clap(short, long)]
    verbose: bool,
    /// Input format
    #[clap(short, long, arg_enum, value_name = "FORMAT", default_value_t = Format::Sfen)]
    format: Format,
    /// Time limit to solve (seconds)
    #[clap(short, long)]
    timeout: Option<f32>,
    /// Input files or SFEN strings
    #[clap(required(true))]
    inputs: Vec<String>,
}

#[derive(Clone, ArgEnum)]
enum Format {
    Sfen,
    Csa,
    Kif,
}

fn main() -> Result<(), std::io::Error> {
    let args = Args::parse();
    match args.format {
        Format::Sfen => run_sfen(&args),
        Format::Csa => run_parse(CsaParser, &args),
        Format::Kif => run_parse(KifParser, &args),
    }
}

fn run_sfen(args: &Args) -> Result<(), std::io::Error> {
    if args.inputs == ["-"] {
        let stdin = std::io::stdin();
        for line in stdin.lock().lines() {
            let sfen = line?;
            let mut pos = Position::new();
            match pos.set_sfen(&sfen) {
                Ok(()) => run(&sfen, &sfen, args),
                Err(e) => {
                    eprintln!("failed to parse SFEN string: {}", e);
                    std::process::exit(1);
                }
            }
        }
    } else {
        for input in &args.inputs {
            let mut pos = Position::new();
            match pos.set_sfen(input) {
                Ok(()) => run(input, input.trim(), args),
                Err(e) => {
                    eprintln!("failed to parse SFEN string: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
    Ok(())
}

fn run_parse<T>(parser: T, args: &Args) -> Result<(), std::io::Error>
where
    T: Parse,
{
    if args.inputs == ["-"] {
        let stdin = std::io::stdin();
        let mut buf = Vec::new();
        stdin.lock().read_to_end(&mut buf)?;
        match parser.parse(&buf) {
            Ok(sfen) => run(&sfen, "-", args),
            Err(e) => {
                eprintln!("failed to parse input: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        for input in &args.inputs {
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
                Ok(sfen) => run(&sfen, input, args),
                Err(e) => {
                    eprintln!("failed to parse input {}: {}", input, e);
                    std::process::exit(1);
                }
            }
        }
    }
    Ok(())
}

fn run(sfen: &str, input: &str, args: &Args) {
    print!("{}: ", input);
    if args.verbose {
        let mut pos = Position::new();
        pos.set_sfen(sfen).expect("failed to parse SFEN string");
        println!();
        println!("{pos}");
        println!();
    }
    let now = Instant::now();
    println!(
        "{:?}",
        solve(sfen, args.timeout.map(Duration::from_secs_f32),)
    );
    if args.verbose {
        println!("elapsed: {:?}", now.elapsed());
    }
}
