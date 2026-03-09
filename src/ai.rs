use crate::game;
use rand::{self, Rng, thread_rng};

pub fn get_random_move(board: &mut game::Board) -> Option<game::Move> {
    let moves = game::get_legal_moves(board);
    if moves.is_empty() || board.is_50_move_draw() || board.is_threefold_repetition() {
        return None;
    };
    let mut rng = thread_rng();
    let index = rng.gen_range(0..moves.len());
    Some(moves[index])
}
