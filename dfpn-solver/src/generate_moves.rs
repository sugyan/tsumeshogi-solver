use crate::types::Node;
use yasai::{Move, Position};

pub fn generate_legal_moves(pos: &mut Position, node: Node) -> Vec<(Move, u64)> {
    let mut children = Vec::new();
    for m in pos.legal_moves() {
        if node == Node::And || pos.is_check_move(m) {
            pos.do_move(m);
            children.push((m, pos.key()));
            pos.undo_move(m);
        }
    }
    children
}
