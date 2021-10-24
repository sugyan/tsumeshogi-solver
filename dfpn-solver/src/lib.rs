use shogi::bitboard::Factory;
use shogi::Position;

pub fn solve(pos: &Position) -> String {
    Factory::init();
    // TODO
    format!("solve {}", pos.to_sfen())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
