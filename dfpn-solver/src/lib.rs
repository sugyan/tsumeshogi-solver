use csa::Position;

pub fn solve(position: &Position) -> String {
    format!("solve {:?}", position)
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
