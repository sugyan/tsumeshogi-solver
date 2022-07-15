use dfpn::search::Search;
use dfpn::{Node, Position, Table, INF};
use dfpn_extended::{CancelableSearcher, CanceledError};
use shogi_core::{Move, PartialPosition};
use std::collections::HashSet;
use std::time::Duration;

pub trait CalculateResult {
    fn calculate_result_and_score(&self, moves: &[Move]) -> (Vec<Move>, usize);
}

pub fn solve<P, T>(
    position: PartialPosition,
    timeout: Option<Duration>,
) -> Result<Vec<Move>, CanceledError>
where
    P: Position + From<PartialPosition> + CalculateResult,
    T: Table,
{
    let pos = P::from(position);
    let mut searcher: CancelableSearcher<P, T> = CancelableSearcher::new(pos, timeout);
    searcher.dfpn_search().map(|_| {
        let mut solutions = Vec::new();
        search_all_mates(
            &mut searcher,
            &mut Vec::new(),
            &mut HashSet::new(),
            &mut solutions,
        );
        solutions.sort_by_cached_key(|&(_, score)| score);
        solutions.dedup();
        solutions
            .last()
            .map_or(Vec::new(), |(moves, _)| moves.clone())
    })
}

fn search_all_mates<P, T>(
    searcher: &mut CancelableSearcher<P, T>,
    moves: &mut Vec<Move>,
    hashes: &mut HashSet<u64>,
    solutions: &mut Vec<(Vec<Move>, usize)>,
) where
    P: Position + CalculateResult,
    T: Table,
{
    let (node, mate_pd) = if moves.len() & 1 == 0 {
        (Node::Or, (INF, 0))
    } else {
        (Node::And, (0, INF))
    };
    let mate_moves = searcher
        .generate_legal_moves(node)
        .into_iter()
        .filter(|(_, h)| !hashes.contains(h) && searcher.look_up_hash(h) == mate_pd)
        .collect::<Vec<_>>();
    if node == Node::And && mate_moves.is_empty() {
        solutions.push(searcher.pos.calculate_result_and_score(moves));
    } else {
        for &(m, h) in &mate_moves {
            moves.push(m.into());
            hashes.insert(h);
            searcher.do_move(m);
            search_all_mates(searcher, moves, hashes, solutions);
            searcher.undo_move(m);
            moves.pop();
            hashes.remove(&h);
        }
    }
}
