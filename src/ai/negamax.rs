use crate::engine::{Board, Move, MoveList};
use std::collections::HashMap;

enum TTFlag {
    Exact,
    UpperBound,
    LowerBound,
}

struct TTEntry {
    hash: u64,
    depth: u8,
    score: isize,
    flag: TTFlag,
}

type TranspositionTable = HashMap<u64, TTEntry>;

fn evaluate(board: &Board) -> isize {
    board.material_evaluation()
}

fn negamax(
    board: &mut Board,
    depth: u8,
    mut alpha: isize,
    mut beta: isize,
    tt: &mut TranspositionTable,
) -> isize {
    let original_alpha = alpha;
    let hash = board.zobrist();

    // Check transposition table
    if let Some(entry) = tt.get(&hash)
        && entry.hash == hash
        && entry.depth >= depth
    {
        match entry.flag {
            TTFlag::Exact => return entry.score,
            TTFlag::LowerBound => alpha = alpha.max(entry.score),
            TTFlag::UpperBound => beta = beta.min(entry.score),
        }
        if alpha >= beta {
            return entry.score;
        }
    }

    if board.is_draw() {
        return 0;
    }

    if depth == 0 {
        return evaluate(board);
    }

    let mut moves = MoveList::new();
    board.get_legal_moves_list(&mut moves);
    if moves.count == 0 {
        if board.is_king_checked() {
            return -10000 + depth as isize;
        } else {
            return 0;
        }
    }

    let mut best_score = isize::MIN;

    for mv in moves.iter() {
        board.make_move(*mv);
        let score = -negamax(board, depth - 1, -beta, -alpha, tt);
        board.unmake_move(*mv);

        if score > best_score {
            best_score = score;
        }
        alpha = alpha.max(score);
        if alpha >= beta {
            break;
        }
    }

    let flag = if best_score <= original_alpha {
        TTFlag::UpperBound
    } else if best_score >= beta {
        TTFlag::LowerBound
    } else {
        TTFlag::Exact
    };
    tt.insert(
        hash,
        TTEntry {
            hash,
            depth,
            score: best_score,
            flag,
        },
    );

    best_score
}

pub fn search_root(board: &mut Board, depth: u8) -> Option<Move> {
    if board.is_draw() || !board.has_legal_moves() {
        return None;
    }
    let mut tt = TranspositionTable::new();
    let alpha = isize::MIN;
    let beta = isize::MAX;
    let mut best_score = isize::MIN;
    let mut best_move = None;

    let mut moves = MoveList::new();
    board.get_legal_moves_list(&mut moves);
    for mv in moves.iter() {
        board.make_move(*mv);
        let score = -negamax(board, depth, alpha, beta, &mut tt);
        board.unmake_move(*mv);

        if score > best_score {
            best_score = score;
            best_move = Some(mv);
        }
    }

    best_move.copied()
}
