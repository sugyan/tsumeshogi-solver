mod csa_converter;
mod sfen_converter;

use std::fmt;
use std::time::Duration;

#[derive(Default, Debug, PartialEq, Eq)]
pub struct Record {
    // pub black_player: Option<String>,
    // pub white_player: Option<String>,
    // pub event: Option<String>,
    // pub site: Option<String>,
    // pub start_time: Option<Time>,
    // pub end_time: Option<Time>,
    // pub time_limit: Option<TimeLimit>,
    // pub opening: Option<String>,
    pub start_pos: Position,
    pub moves: Vec<Move>,
}

// impl fmt::Display for GameRecord {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         writeln!(f, "V2.2")?;

//         // Metadata
//         let metadata = [
//             ("N+", self.black_player.as_ref().map(|x| x.to_string())),
//             ("N-", self.white_player.as_ref().map(|x| x.to_string())),
//             ("$EVENT:", self.event.as_ref().map(|x| x.to_string())),
//             ("$SITE:", self.site.as_ref().map(|x| x.to_string())),
//             (
//                 "$START_TIME:",
//                 self.start_time.as_ref().map(|x| x.to_string()),
//             ),
//             ("$END_TIME:", self.end_time.as_ref().map(|x| x.to_string())),
//             (
//                 "$TIME_LIMIT:",
//                 self.time_limit.as_ref().map(|x| x.to_string()),
//             ),
//             ("$OPENING:", self.opening.as_ref().map(|x| x.to_string())),
//         ];
//         for &(ref key, ref value) in &metadata {
//             if let Some(ref value) = *value {
//                 writeln!(f, "{}{}", key, value)?;
//             }
//         }

//         // Position
//         write!(f, "{}", self.start_pos)?;

//         // Move records
//         for record in &self.moves {
//             write!(f, "{}", record)?;
//         }

//         Ok(())
//     }
// }

////////////////////////////////////////////////////////////////////////////////

// #[derive(Debug, PartialEq, Eq, Clone)]
// pub struct Time {
//     pub date: NaiveDate,
//     pub time: Option<NaiveTime>,
// }

// impl fmt::Display for Time {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "{}", self.date.format("%Y/%m/%d"))?;
//         if let Some(time) = self.time {
//             write!(f, " {}", time.format("%H:%M:%S"))?;
//         }

//         Ok(())
//     }
// }

////////////////////////////////////////////////////////////////////////////////

// #[derive(Debug, PartialEq, Eq, Clone)]
// pub struct TimeLimit {
//     pub main_time: Duration,
//     pub byoyomi: Duration,
// }

// impl fmt::Display for TimeLimit {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         let secs = self.main_time.as_secs();
//         let hours = secs / 3600;
//         let minutes = (secs % 3600) / 60;

//         write!(
//             f,
//             "{:02}:{:02}+{:02}",
//             hours,
//             minutes,
//             self.byoyomi.as_secs()
//         )
//     }
// }

////////////////////////////////////////////////////////////////////////////////

// #[derive(Debug, PartialEq, Eq)]
// pub enum GameAttribute {
//     Time(Time),
//     TimeLimit(TimeLimit),
//     Str(String),
// }

// impl fmt::Display for GameAttribute {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         match *self {
//             GameAttribute::Time(ref time) => write!(f, "{}", time),
//             GameAttribute::TimeLimit(ref time_limit) => write!(f, "{}", time_limit),
//             GameAttribute::Str(ref s) => write!(f, "{}", s),
//         }
//     }
// }

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Color {
    Black,
    White,
}

impl Default for Color {
    fn default() -> Self {
        Color::Black
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Color::Black => write!(f, "+"),
            Color::White => write!(f, "-"),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Square {
    pub file: u8,
    pub rank: u8,
}

impl Square {
    pub fn new(file: u8, rank: u8) -> Square {
        Square { file, rank }
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.file, self.rank)
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum PieceType {
    Pawn,
    Lance,
    Knight,
    Silver,
    Gold,
    Bishop,
    Rook,
    King,
    ProPawn,
    ProLance,
    ProKnight,
    ProSilver,
    Horse,
    Dragon,

    Extra,
}

impl PieceType {
    fn unpromoted(self) -> Self {
        use self::PieceType::*;
        match self {
            Pawn | ProPawn => Pawn,
            Lance | ProLance => Lance,
            Knight | ProKnight => Knight,
            Silver | ProSilver => Silver,
            Gold => Gold,
            Bishop | Horse => Bishop,
            Rook | Dragon => Rook,
            King => King,
            _ => Extra,
        }
    }
}

// impl fmt::Display for PieceType {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         let pt = match *self {
//             PieceType::Pawn => "FU",
//             PieceType::Lance => "KY",
//             PieceType::Knight => "KE",
//             PieceType::Silver => "GI",
//             PieceType::Gold => "KI",
//             PieceType::Bishop => "KA",
//             PieceType::Rook => "HI",
//             PieceType::King => "OU",
//             PieceType::ProPawn => "TO",
//             PieceType::ProLance => "NY",
//             PieceType::ProKnight => "NK",
//             PieceType::ProSilver => "NG",
//             PieceType::Horse => "UM",
//             PieceType::Dragon => "RY",
//         };
//         write!(f, "{}", pt)
//     }
// }

////////////////////////////////////////////////////////////////////////////////

type Board = [[Option<(Color, PieceType)>; 9]; 9];

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Position {
    pub drop_pieces: Vec<(Square, PieceType)>,
    pub bulk: Option<Board>,
    pub add_pieces: Vec<(Color, Square, PieceType)>,
    pub side_to_move: Color,
}

// impl fmt::Display for Position {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         if let Some(ref bulk) = self.bulk {
//             for (i, row) in bulk.iter().enumerate() {
//                 write!(f, "P{}", i + 1)?;

//                 for pc in row.iter() {
//                     match *pc {
//                         Some((ref color, ref pt)) => write!(f, "{}{}", color, pt)?,
//                         None => write!(f, " * ")?,
//                     }
//                 }

//                 writeln!(f)?;
//             }
//         } else {
//             write!(f, "PI")?;
//             for &(ref sq, ref pt) in &self.drop_pieces {
//                 write!(f, "{}{}", sq, pt)?;
//             }
//             writeln!(f)?;
//         }

//         for &(ref color, ref sq, ref pt) in &self.add_pieces {
//             writeln!(f, "P{}{}{}", color, sq, pt)?;
//         }

//         writeln!(f, "{}", self.side_to_move)?;

//         Ok(())
//     }
// }

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Action {
    Move(Color, Square, Square, PieceType),
    Toryo,
    Chudan,
    Sennichite,
    TimeUp,
    IllegalMove,
    IllegalAction(Color),
    Jishogi,
    Kachi,
    Hikiwake,
    Matta,
    Tsumi,
    Fuzumi,
    Error,
}

// impl fmt::Display for Action {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         match *self {
//             Action::Move(ref color, ref from, ref to, ref pt) => {
//                 write!(f, "{}{}{}{}", color, from, to, pt)
//             }
//             Action::Toryo => write!(f, "%TORYO"),
//             Action::Chudan => write!(f, "%CHUDAN"),
//             Action::Sennichite => write!(f, "%SENNICHITE"),
//             Action::TimeUp => write!(f, "%TIME_UP"),
//             Action::IllegalMove => write!(f, "%ILLEGAL_MOVE"),
//             Action::IllegalAction(ref color) => write!(f, "%{}ILLEGAL_ACTION", color),
//             Action::Jishogi => write!(f, "%JISHOGI"),
//             Action::Kachi => write!(f, "%KACHI"),
//             Action::Hikiwake => write!(f, "%HIKIWAKE"),
//             Action::Matta => write!(f, "%MATTA"),
//             Action::Tsumi => write!(f, "%TSUMI"),
//             Action::Fuzumi => write!(f, "%FUZUMI"),
//             Action::Error => write!(f, "%ERROR"),
//         }
//     }
// }

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, PartialEq, Eq)]
pub struct Move {
    pub action: Action,
    pub time: Option<Duration>,
}

// impl fmt::Display for Move {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         writeln!(f, "{}", self.action)?;

//         if let Some(ref time) = self.time {
//             writeln!(f, "T{}", time.as_secs())?;
//         }

//         Ok(())
//     }
// }

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
