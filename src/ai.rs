use crate::game;
use rand::{self, Rng, thread_rng};

pub fn get_random_move(board: &mut game::Board) -> game::Move {
    let moves = game::get_legal_moves(board);
    let mut rng = thread_rng();
    let index = rng.gen_range(0..moves.len());
    moves[index]
}
