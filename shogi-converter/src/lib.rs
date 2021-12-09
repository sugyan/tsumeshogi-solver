pub mod csa_converter;
pub mod kif_converter;
pub mod sfen_converter;
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
    pub pos: Position,
    pub moves: Vec<Move>,
}

// #[derive(Debug, PartialEq, Eq, Clone)]
// pub struct Time {
//     pub date: NaiveDate,
//     pub time: Option<NaiveTime>,
// }

////////////////////////////////////////////////////////////////////////////////

// #[derive(Debug, PartialEq, Eq, Clone)]
// pub struct TimeLimit {
//     pub main_time: Duration,
//     pub byoyomi: Duration,
// }

////////////////////////////////////////////////////////////////////////////////

// #[derive(Debug, PartialEq, Eq)]
// pub enum GameAttribute {
//     Time(Time),
//     TimeLimit(TimeLimit),
//     Str(String),
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

impl Color {
    /// Converts the instance into the unique number for array indexing purpose.
    #[inline(always)]
    pub fn index(self) -> usize {
        self as usize
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
}

impl PieceType {
    /// Returns an iterator over all variants.
    pub fn iter() -> PieceTypeIter {
        PieceTypeIter::new()
    }

    fn promote(self) -> Option<Self> {
        use self::PieceType::*;

        Some(match self {
            Pawn => ProPawn,
            Lance => ProLance,
            Knight => ProKnight,
            Silver => ProSilver,
            Bishop => Horse,
            Rook => Dragon,
            _ => return None,
        })
    }

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
        }
    }

    /// Checks if this piece type can be a part of hand pieces.
    pub fn is_hand_piece(self) -> bool {
        matches!(
            self,
            PieceType::Rook
                | PieceType::Bishop
                | PieceType::Gold
                | PieceType::Silver
                | PieceType::Knight
                | PieceType::Lance
                | PieceType::Pawn
        )
    }

    #[inline(always)]
    pub fn index(self) -> usize {
        self as usize
    }
}

pub struct PieceTypeIter {
    current: Option<PieceType>,
}

impl PieceTypeIter {
    fn new() -> PieceTypeIter {
        PieceTypeIter {
            current: Some(PieceType::Pawn),
        }
    }
}

impl Iterator for PieceTypeIter {
    type Item = PieceType;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current;

        if let Some(current) = self.current {
            self.current = match current {
                PieceType::Pawn => Some(PieceType::Lance),
                PieceType::Lance => Some(PieceType::Knight),
                PieceType::Knight => Some(PieceType::Silver),
                PieceType::Silver => Some(PieceType::Gold),
                PieceType::Gold => Some(PieceType::Bishop),
                PieceType::Bishop => Some(PieceType::Rook),
                PieceType::Rook => Some(PieceType::King),
                PieceType::King => Some(PieceType::ProPawn),
                PieceType::ProPawn => Some(PieceType::ProLance),
                PieceType::ProLance => Some(PieceType::ProKnight),
                PieceType::ProKnight => Some(PieceType::ProSilver),
                PieceType::ProSilver => Some(PieceType::Horse),
                PieceType::Horse => Some(PieceType::Dragon),
                PieceType::Dragon => None,
            };
        }
        current
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, PartialEq, Eq)]
pub struct Board([[Option<(Color, PieceType)>; 9]; 9]);

impl Default for Board {
    fn default() -> Self {
        use self::Color::*;
        use self::PieceType::*;
        Self([
            [
                Some((White, Lance)),
                Some((White, Knight)),
                Some((White, Silver)),
                Some((White, Gold)),
                Some((White, King)),
                Some((White, Gold)),
                Some((White, Silver)),
                Some((White, Knight)),
                Some((White, Lance)),
            ],
            [
                None,
                Some((White, Rook)),
                None,
                None,
                None,
                None,
                None,
                Some((White, Bishop)),
                None,
            ],
            [
                Some((White, Pawn)),
                Some((White, Pawn)),
                Some((White, Pawn)),
                Some((White, Pawn)),
                Some((White, Pawn)),
                Some((White, Pawn)),
                Some((White, Pawn)),
                Some((White, Pawn)),
                Some((White, Pawn)),
            ],
            [None, None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None, None],
            [
                Some((Black, Pawn)),
                Some((Black, Pawn)),
                Some((Black, Pawn)),
                Some((Black, Pawn)),
                Some((Black, Pawn)),
                Some((Black, Pawn)),
                Some((Black, Pawn)),
                Some((Black, Pawn)),
                Some((Black, Pawn)),
            ],
            [
                None,
                Some((Black, Bishop)),
                None,
                None,
                None,
                None,
                None,
                Some((Black, Rook)),
                None,
            ],
            [
                Some((Black, Lance)),
                Some((Black, Knight)),
                Some((Black, Silver)),
                Some((Black, Gold)),
                Some((Black, King)),
                Some((Black, Gold)),
                Some((Black, Silver)),
                Some((Black, Knight)),
                Some((Black, Lance)),
            ],
        ])
    }
}

#[derive(Default, Debug, PartialEq, Eq)]
pub struct Position {
    pub drop_pieces: Vec<(Square, PieceType)>,
    pub board: Board,
    pub hand: Hand,
    pub side_to_move: Color,
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Default, Debug, PartialEq, Eq)]
pub struct Hand([[u8; 7]; 2]);

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

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, PartialEq, Eq)]
pub struct Move {
    pub action: Action,
    pub time: Option<Duration>,
    pub comments: Vec<String>,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
