use crate::{Color, PieceType, Record};
use itertools::Itertools;

impl Record {
    pub fn to_sfen(&self) -> String {
        let board = (0..9)
            .map(|rank| {
                let mut s = String::new();
                let mut num_spaces = 0;
                for file in 0..9 {
                    if let Some((c, pt)) = self.pos.board.0[rank][file] {
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
            .join("/");
        let color = match self.pos.side_to_move {
            Color::Black => "b",
            Color::White => "w",
        };
        let mut hand = [Color::Black, Color::White]
            .iter()
            .map(|&color| {
                ['r', 'b', 'g', 's', 'n', 'l', 'p']
                    .iter()
                    .zip(self.pos.hand.0[color.index()].iter().rev())
                    .map(|(&c, &num)| {
                        let s = match num {
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
        });
        match c {
            Color::Black => p.to_uppercase(),
            Color::White => p,
        }
    }
}
