use clap::{ArgEnum, Parser};
use encoding_rs::SHIFT_JIS;
use shogi_core::{Color, Move, PartialPosition, PieceKind, Position, Square, ToUsi};
use shogi_kifu_converter::converter::ToCsa;
use shogi_kifu_converter::error::{ConvertError, CoreConvertError, NormalizerError};
use shogi_kifu_converter::jkf::JsonKifuFormat;
use shogi_kifu_converter::parser::{parse_csa_str, parse_kif_str};
use shogi_official_kifu::display_single_move_kansuji;
use shogi_usi_parser::FromUsi;
use std::fmt::Write;
use std::fs::File;
use std::io::{BufRead, Read};
use std::time::{Duration, Instant};
use thiserror::Error;
use tsumeshogi_solver::solver::implementations::{HashMapTable, YasaiPosition};
use tsumeshogi_solver::solver::solve;

#[derive(Error, Debug)]
enum ParseError {
    #[error(transparent)]
    Usi(#[from] shogi_usi_parser::Error),
    #[error(transparent)]
    Utf8(#[from] std::str::Utf8Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Convert(#[from] ConvertError),
    #[error(transparent)]
    Normalize(#[from] NormalizerError),
    #[error(transparent)]
    CoreConvert(#[from] CoreConvertError),
    #[error(transparent)]
    KifError(#[from] KifError),
}

#[derive(Error, Debug)]
enum KifError {
    #[error("Input is not SHIFT-JIS")]
    EncodingNotShiftJISError,
    #[error("Decode error")]
    DecodingError,
}

trait Parse {
    fn parse(&self, input: &[u8]) -> Result<PartialPosition, ParseError>;
}

struct CsaParser;

impl Parse for CsaParser {
    fn parse(&self, input: &[u8]) -> Result<PartialPosition, ParseError> {
        let jkf = parse_csa_str(std::str::from_utf8(input)?)?;
        let pos = Position::try_from(&jkf)?;
        Ok(pos.initial_position().clone())
    }
}

struct KifParser;

impl Parse for KifParser {
    fn parse(&self, input: &[u8]) -> Result<PartialPosition, ParseError> {
        let (cow, encoding_used, had_errors) = SHIFT_JIS.decode(input);
        if encoding_used != SHIFT_JIS {
            return Err(ParseError::from(KifError::EncodingNotShiftJISError));
        }
        if had_errors {
            return Err(ParseError::from(KifError::DecodingError));
        }
        let jkf = parse_kif_str(&cow)?;
        let pos = Position::try_from(&jkf)?;
        Ok(pos.initial_position().clone())
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
    #[clap(short, long, arg_enum, value_name = "FORMAT", default_value_t = InputFormat::Sfen)]
    input_format: InputFormat,
    /// Output format
    #[clap(short, long, arg_enum, value_name = "FORMAT", default_value_t = OutputFormat::Usi)]
    output_format: OutputFormat,
    /// Time limit to solve (seconds)
    #[clap(short, long)]
    timeout: Option<f32>,
    /// Input files or SFEN strings
    #[clap(required(true))]
    inputs: Vec<String>,
}

#[derive(Clone, ArgEnum)]
enum InputFormat {
    Sfen,
    Csa,
    Kif,
}

#[derive(Clone, Copy, ArgEnum)]
enum OutputFormat {
    Usi,
    Csa,
    Kifu,
}

fn main() -> Result<(), ParseError> {
    let args = Args::parse();
    match args.input_format {
        InputFormat::Sfen => run_sfen(&args),
        InputFormat::Csa => run_parse(CsaParser, &args),
        InputFormat::Kif => run_parse(KifParser, &args),
    }
}

fn run_sfen(args: &Args) -> Result<(), ParseError> {
    if args.inputs == ["-"] {
        let stdin = std::io::stdin();
        for line in stdin.lock().lines() {
            let usi = format!("sfen {}", line?);
            let pos = PartialPosition::from_usi(&usi)?;
            run(&pos, &usi, args)?;
        }
    } else {
        for input in &args.inputs {
            let usi = format!("sfen {}", input.trim());
            let pos = PartialPosition::from_usi(&usi)?;
            run(&pos, &usi, args)?;
        }
    }
    Ok(())
}

fn run_parse<T>(parser: T, args: &Args) -> Result<(), ParseError>
where
    T: Parse,
{
    if args.inputs == ["-"] {
        let stdin = std::io::stdin();
        let mut buf = Vec::new();
        stdin.lock().read_to_end(&mut buf)?;
        run(&parser.parse(&buf)?, "-", args)?
    } else {
        for input in &args.inputs {
            let mut buf = Vec::new();
            let mut file = File::open(input)?;
            file.read_to_end(&mut buf)?;
            run(&parser.parse(&buf)?, input, args)?
        }
    }
    Ok(())
}

fn run(pos: &PartialPosition, input: &str, args: &Args) -> Result<(), ParseError> {
    print!("{}: ", input);
    if args.verbose {
        let jkf = JsonKifuFormat::try_from(&Position::arbitrary_position(pos.clone()))?;
        println!();
        println!("{}", jkf.to_csa_owned());
    }
    let now = Instant::now();
    let result = solve::<YasaiPosition, HashMapTable>(
        pos.clone(),
        args.timeout.map(Duration::from_secs_f32),
    )
    .map(|v| output(pos, v, args.output_format).join(" "));
    println!("{:?}", result);
    if args.verbose {
        println!("elapsed: {:?}", now.elapsed());
    }
    Ok(())
}

fn output(pos: &PartialPosition, v: Vec<Move>, format: OutputFormat) -> Vec<String> {
    match format {
        OutputFormat::Usi => v.iter().map(|m| m.to_usi_owned()).collect(),
        OutputFormat::Csa => v
            .iter()
            .scan(pos.clone(), |pos, &m| {
                let ret = move2csa(pos, m);
                pos.make_move(m);
                ret.ok()
            })
            .collect(),
        OutputFormat::Kifu => v
            .iter()
            .scan(pos.clone(), |pos, &m| {
                let ret = display_single_move_kansuji(pos, m);
                pos.make_move(m);
                ret
            })
            .collect(),
    }
}

fn move2csa(pos: &PartialPosition, m: Move) -> Result<String, std::fmt::Error> {
    let mut ret = String::new();
    write_c(pos.side_to_move(), &mut ret)?;
    match m {
        Move::Normal { from, to, promote } => {
            write_sq(from, &mut ret)?;
            write_sq(to, &mut ret)?;
            let pk = pos.piece_at(from).expect("no piece at `from`").piece_kind();
            write_pk(
                if promote {
                    pk.promote().expect("piece kind is not promoted")
                } else {
                    pk
                },
                &mut ret,
            )?;
        }
        Move::Drop { piece, to } => {
            ret.write_str("00")?;
            write_sq(to, &mut ret)?;
            write_pk(piece.piece_kind(), &mut ret)?;
        }
    }
    Ok(ret)
}

fn write_c<W: Write>(c: Color, sink: &mut W) -> Result<(), std::fmt::Error> {
    match c {
        Color::Black => sink.write_char('+'),
        Color::White => sink.write_char('-'),
    }
}

fn write_sq<W: Write>(sq: Square, sink: &mut W) -> Result<(), std::fmt::Error> {
    sink.write_fmt(format_args!("{}{}", sq.file(), sq.rank()))
}

fn write_pk<W: Write>(pk: PieceKind, sink: &mut W) -> Result<(), std::fmt::Error> {
    match pk {
        PieceKind::Pawn => sink.write_str("FU")?,
        PieceKind::Lance => sink.write_str("KY")?,
        PieceKind::Knight => sink.write_str("KE")?,
        PieceKind::Silver => sink.write_str("GI")?,
        PieceKind::Gold => sink.write_str("KI")?,
        PieceKind::Bishop => sink.write_str("KA")?,
        PieceKind::Rook => sink.write_str("HI")?,
        PieceKind::King => sink.write_str("OU")?,
        PieceKind::ProPawn => sink.write_str("TO")?,
        PieceKind::ProLance => sink.write_str("NY")?,
        PieceKind::ProKnight => sink.write_str("NK")?,
        PieceKind::ProSilver => sink.write_str("NG")?,
        PieceKind::ProBishop => sink.write_str("UM")?,
        PieceKind::ProRook => sink.write_str("RY")?,
    }
    Ok(())
}
