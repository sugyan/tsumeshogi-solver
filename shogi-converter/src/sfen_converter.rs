use crate::{Color, Hand, PieceType, Position, Record};
use itertools::Itertools;

impl Position {
    pub fn to_sfen(&self) -> String {
        let board = (0..9)
            .map(|rank| {
                let mut s = String::new();
                let mut num_spaces = 0;
                for file in 0..9 {
                    if let Some((c, pt)) = self.board.0[rank][file] {
                        if num_spaces > 0 {
                            s.push_str(&num_spaces.to_string());
                            num_spaces = 0;
                        }
                        s.push_str(&pt.to_sfen(c));
                    } else {
                        num_spaces += 1;
                    }
                }
                if num_spaces > 0 {
                    s.push_str(&num_spaces.to_string());
                }
                s
            })
            .join("/");
        let color = self.side_to_move.to_sfen();
        let hand = self.hand.to_sfen();
        format!("{} {} {} 1", board, color, hand)
    }
}

impl Color {
    pub fn to_sfen(&self) -> String {
        String::from(match self {
            Color::Black => "b",
            Color::White => "w",
        })
    }
}

impl PieceType {
    pub fn to_sfen(&self, color: Color) -> String {
        let p = String::from(match self {
            PieceType::Pawn => "p",
            PieceType::Lance => "l",
            PieceType::Knight => "n",
            PieceType::Silver => "s",
            PieceType::Gold => "g",
            PieceType::Bishop => "b",
            PieceType::Rook => "r",
            PieceType::King => "k",
            PieceType::ProPawn => "+p",
            PieceType::ProLance => "+l",
            PieceType::ProKnight => "+n",
            PieceType::ProSilver => "+s",
            PieceType::Horse => "+b",
            PieceType::Dragon => "+r",
        });
        match color {
            Color::Black => p.to_uppercase(),
            Color::White => p,
        }
    }
}

impl Hand {
    pub fn to_sfen(&self) -> String {
        let ret = [Color::Black, Color::White]
            .iter()
            .map(|&color| {
                [
                    PieceType::Rook,
                    PieceType::Bishop,
                    PieceType::Gold,
                    PieceType::Silver,
                    PieceType::Knight,
                    PieceType::Lance,
                    PieceType::Pawn,
                ]
                .iter()
                .map(|pt| {
                    let num = self.0[color.index()][pt.index()];
                    match num {
                        0 => String::new(),
                        1 => pt.to_sfen(color),
                        n => format!("{}{}", n, pt.to_sfen(color)),
                    }
                })
                .join("")
            })
            .join("");
        if ret.is_empty() {
            String::from("-")
        } else {
            ret
        }
    }
}

impl Record {
    pub fn to_sfen(&self) -> String {
        // TODO: moves
        self.pos.to_sfen()
    }
}
