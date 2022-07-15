use clap::{ArgEnum, Parser};
use csa::{parse_csa, CsaError};
use shogi_converter::kif_converter::{parse_kif, KifError};
use shogi_converter::Record;
use shogi_core::{Color, Hand, Move, PartialPosition, PieceKind, Square, ToUsi};
use shogi_official_kifu::display_single_move_kansuji;
use shogi_usi_parser::FromUsi;
use solver::implementations::{HashMapTable, YasaiPosition};
use solver::solve;
use std::fs::File;
use std::io::{BufRead, Read};
use std::time::{Duration, Instant};
use thiserror::Error;

#[derive(Error, Debug)]
enum ParseError {
    #[error("failed to parse csa: {0}")]
    Csa(CsaError),
    #[error("failed to parse kif: {0}")]
    Kif(KifError),
    #[error(transparent)]
    Usi(#[from] shogi_usi_parser::Error),
    #[error(transparent)]
    Utf8(#[from] std::str::Utf8Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

trait Parse {
    fn parse(&self, input: &[u8]) -> Result<String, ParseError>;
}

struct CsaParser;

impl Parse for CsaParser {
    fn parse(&self, input: &[u8]) -> Result<String, ParseError> {
        match parse_csa(std::str::from_utf8(input).map_err(ParseError::Utf8)?) {
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
            let sfen = line?;
            run(&sfen, &sfen, args)?;
        }
    } else {
        for input in &args.inputs {
            run(input, input.trim(), args)?;
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

fn run(sfen: &str, input: &str, args: &Args) -> Result<(), ParseError> {
    print!("{}: ", input);
    let pos = PartialPosition::from_usi(&format!("sfen {sfen}"))?;
    if args.verbose {
        println!();
        println!("{}", pos2csa(&pos));
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

fn output(pos: PartialPosition, v: Vec<Move>, format: OutputFormat) -> Vec<String> {
    match format {
        OutputFormat::Usi => v.iter().map(|m| m.to_usi_owned()).collect(),
        OutputFormat::Csa => v
            .iter()
            .scan(pos, |pos, &m| {
                let ret = move2action(pos, m).to_string();
                pos.make_move(m);
                Some(ret)
            })
            .collect(),
        OutputFormat::Kifu => v
            .iter()
            .scan(pos, |pos, &m| {
                let ret = display_single_move_kansuji(pos, m);
                pos.make_move(m);
                ret
            })
            .collect(),
    }
}

fn pos2csa(pos: &PartialPosition) -> String {
    let mut remains = Hand::default();
    for (pk, num) in [
        (PieceKind::Pawn, 18),
        (PieceKind::Lance, 4),
        (PieceKind::Knight, 4),
        (PieceKind::Silver, 4),
        (PieceKind::Gold, 4),
        (PieceKind::Bishop, 2),
        (PieceKind::Rook, 2),
    ] {
        for _ in 0..num {
            remains = remains.added(pk).unwrap();
        }
    }
    let mut board = [[None; 9]; 9];
    for sq in Square::all() {
        if let Some(p) = pos.piece_at(sq) {
            board[sq.rank() as usize - 1][9 - sq.file() as usize] =
                Some((c2c(p.color()), pk2pt(p.piece_kind())));
            if p.piece_kind() != PieceKind::King {
                remains = remains
                    .removed(p.unpromote().unwrap_or(p).piece_kind())
                    .unwrap();
            }
        }
    }
    let mut add_pieces = Vec::new();
    for &pk in Hand::all_hand_pieces().collect::<Vec<_>>().iter().rev() {
        for _ in 0..pos
            .hand_of_a_player(pos.side_to_move())
            .count(pk)
            .unwrap_or_default()
        {
            add_pieces.push((c2c(pos.side_to_move()), csa::Square::new(0, 0), pk2pt(pk)));
            remains = remains.removed(pk).unwrap();
        }
    }
    if remains == pos.hand_of_a_player(pos.side_to_move().flip()) {
        add_pieces.push((
            c2c(pos.side_to_move().flip()),
            csa::Square::new(0, 0),
            csa::PieceType::All,
        ));
    } else {
        for &pk in Hand::all_hand_pieces().collect::<Vec<_>>().iter().rev() {
            for _ in 0..pos
                .hand_of_a_player(pos.side_to_move().flip())
                .count(pk)
                .unwrap_or_default()
            {
                add_pieces.push((
                    c2c(pos.side_to_move().flip()),
                    csa::Square::new(0, 0),
                    pk2pt(pk),
                ));
            }
        }
    }
    csa::Position {
        drop_pieces: Vec::new(),
        bulk: Some(board),
        add_pieces,
        side_to_move: c2c(pos.side_to_move()),
    }
    .to_string()
}

fn move2action(pos: &PartialPosition, m: Move) -> csa::Action {
    match m {
        Move::Normal { from, to, promote } => {
            let p = pos.piece_at(from).expect("piece not found");
            let pk = if promote { p.promote().unwrap_or(p) } else { p }.piece_kind();
            csa::Action::Move(c2c(pos.side_to_move()), sq2sq(from), sq2sq(to), pk2pt(pk))
        }
        Move::Drop { to, piece } => csa::Action::Move(
            c2c(pos.side_to_move()),
            csa::Square::new(0, 0),
            sq2sq(to),
            pk2pt(piece.piece_kind()),
        ),
    }
}

fn pk2pt(pk: PieceKind) -> csa::PieceType {
    match pk {
        PieceKind::Pawn => csa::PieceType::Pawn,
        PieceKind::Lance => csa::PieceType::Lance,
        PieceKind::Knight => csa::PieceType::Knight,
        PieceKind::Silver => csa::PieceType::Silver,
        PieceKind::Gold => csa::PieceType::Gold,
        PieceKind::Bishop => csa::PieceType::Bishop,
        PieceKind::Rook => csa::PieceType::Rook,
        PieceKind::King => csa::PieceType::King,
        PieceKind::ProPawn => csa::PieceType::ProPawn,
        PieceKind::ProLance => csa::PieceType::ProLance,
        PieceKind::ProKnight => csa::PieceType::ProKnight,
        PieceKind::ProSilver => csa::PieceType::ProSilver,
        PieceKind::ProBishop => csa::PieceType::Horse,
        PieceKind::ProRook => csa::PieceType::Dragon,
    }
}

fn c2c(c: Color) -> csa::Color {
    match c {
        Color::Black => csa::Color::Black,
        Color::White => csa::Color::White,
    }
}

fn sq2sq(sq: Square) -> csa::Square {
    csa::Square::new(sq.file(), sq.rank())
}
