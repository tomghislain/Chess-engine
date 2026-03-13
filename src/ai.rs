use crate::game;
use rand::{self, Rng, thread_rng};
use std::collections::HashMap;

pub fn get_random_move(board: &mut game::Board) -> Option<game::Move> {
    let moves = board.get_legal_moves();
    if moves.is_empty() || board.is_draw() {
        return None;
    };
    let mut rng = thread_rng();
    let index = rng.gen_range(0..moves.len());
    Some(moves[index])
}

enum TTFlag {
    Exact,
    UpperBound,
    LowerBound,
}

struct TTEntry {
    hash: u64,
    depth: u8,
    score: i8,
    flag: TTFlag,
}

type TranspositionTable = HashMap<u64, TTEntry>;

fn evaluate(board: &game::Board) -> i8 {
    match board.game_result() {
        None => 0,
        Some(win) => {
            if win {
                -1
            } else {
                0
            }
        }
    }
}

fn negamax(
    board: &mut game::Board,
    depth: u8,
    mut alpha: i8,
    mut beta: i8,
    tt: &mut TranspositionTable,
) -> i8 {
    let original_alpha = alpha;
    let hash = board.zobrist;

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

    if depth == 0 {
        return evaluate(board);
    }

    let moves = board.get_legal_moves();
    if moves.is_empty() {
        return evaluate(board);
    }

    let mut best_score = i8::MIN;

    for mv in moves {
        board.make_move(mv);
        let score = -negamax(board, depth - 1, -beta, -alpha, tt);
        board.unmake_move(mv);

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

pub fn search_root(board: &mut game::Board, depth: u8) -> Option<game::Move> {
    if board.game_result().is_some() {
        return None;
    }
    let mut tt = TranspositionTable::new();
    let alpha = i8::MIN;
    let beta = i8::MAX;
    let mut best_score = i8::MIN;
    let mut best_move = None;

    let moves = board.get_legal_moves();
    for mv in moves {
        board.make_move(mv);
        let score = -negamax(board, depth, alpha, beta, &mut tt);
        board.unmake_move(mv);

        if score > best_score {
            best_score = score;
            best_move = Some(mv);
        }
    }

    best_move
}
