use std::ops;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Node {
    Or,
    And,
}

impl ops::Not for Node {
    type Output = Node;

    fn not(self) -> Self::Output {
        match self {
            Node::Or => Node::And,
            Node::And => Node::Or,
        }
    }
}
