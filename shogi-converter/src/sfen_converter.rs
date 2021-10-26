use crate::{Color, PieceType, Record};
use itertools::Itertools;

impl Record {
    pub fn to_sfen(&self) -> String {
        let board = if let Some(b) = self.start_pos.bulk {
            (0..9)
                .map(|rank| {
                    let mut s = String::new();
                    let mut num_spaces = 0;
                    for file in 0..9 {
                        if let Some((c, pt)) = b[rank][file] {
                            if num_spaces > 0 {
                                s.push_str(&num_spaces.to_string());
                                num_spaces = 0;
                            }
                            s.push_str(&Self::convert_piece(c, pt));
                        } else {
                            num_spaces += 1;
                        }
                    }
                    if num_spaces > 0 {
                        s.push_str(&num_spaces.to_string());
                    }
                    s
                })
                .join("/")
        } else {
            String::from("lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL")
        };
        let color = match self.start_pos.side_to_move {
            Color::Black => "b",
            Color::White => "w",
        };
        let mut hand = [Color::Black, Color::White]
            .iter()
            .map(|&color| {
                let pieces = self
                    .start_pos
                    .add_pieces
                    .iter()
                    .filter_map(|&(c, _, pt)| if c == color { Some(pt) } else { None })
                    .collect::<Vec<_>>();
                let counts = pieces.iter().fold([0; 7], |mut c, &pt| {
                    c[match pt {
                        PieceType::Rook => 0,
                        PieceType::Bishop => 1,
                        PieceType::Gold => 2,
                        PieceType::Silver => 3,
                        PieceType::Knight => 4,
                        PieceType::Lance => 5,
                        PieceType::Pawn => 6,
                        _ => unreachable!(),
                    }] += 1;
                    c
                });
                ['r', 'b', 'g', 's', 'n', 'l', 'p']
                    .iter()
                    .enumerate()
                    .map(|(i, &c)| {
                        let s = match counts[i] {
                            0 => String::new(),
                            1 => c.to_string(),
                            n => format!("{}{}", n, c),
                        };
                        match color {
                            Color::Black => s.to_uppercase(),
                            Color::White => s,
                        }
                    })
                    .join("")
            })
            .join("");
        if hand.is_empty() {
            hand = String::from("-");
        }
        format!("{} {} {} 1", board, color, hand)
    }

    fn convert_piece(c: Color, pt: PieceType) -> String {
        let p = String::from(match pt {
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
            _ => "",
        });
        match c {
            Color::Black => p.to_uppercase(),
            Color::White => p,
        }
    }
}
