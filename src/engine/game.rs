use crate::engine::MoveList;
use crate::engine::board::Undo;
use crate::engine::{Board, Move};
use rand::RngExt;
use rand::rngs::SmallRng;

const MAX_GAME_LENGTH: usize = 2048;

#[derive(Clone)]
pub struct GameStruct {
    pub b: Board,
    undo_stack: [Undo; MAX_GAME_LENGTH],
    ply: usize,
}

impl GameStruct {
    pub fn from_fen(fen: &str) -> Self {
        Self {
            b: Board::from_fen(fen),
            undo_stack: [Undo::default(); MAX_GAME_LENGTH],
            ply: 0,
        }
    }

    pub fn make_move(&mut self, mv: Move) {
        self.undo_stack[self.ply] = self.b.make_move(mv);
        self.ply += 1;
    }

    pub fn unmake_move(&mut self) {
        self.ply -= 1;
        self.b.unmake_move(self.undo_stack[self.ply])
    }

    fn is_threefold_repetition(&self) -> bool {
        let current = self.b.hash;
        self.undo_stack
            .iter()
            .rev()
            .skip(1)
            .step_by(2)
            .filter(|u| u.hash == current)
            .take(2)
            .count()
            >= 2
    }

    pub fn is_draw(&self) -> bool {
        self.b.is_draw() || self.is_threefold_repetition()
    }

    pub fn get_random_move(&self, rng: &mut SmallRng) -> Option<Move> {
        let mut moves = MoveList::default();
        self.b.get_legal_moves_list(&mut moves);
        if moves.count == 0 || self.is_draw() {
            return None;
        }
        let index = rng.random_range(0..moves.count);
        moves.get(index)
    }
}

impl Default for GameStruct {
    fn default() -> Self {
        Self {
            b: Board::default(),
            undo_stack: [Undo::default(); MAX_GAME_LENGTH],
            ply: 0,
        }
    }
}
