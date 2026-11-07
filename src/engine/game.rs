use crate::engine::MoveList;
use crate::engine::board::Undo;
use crate::engine::{Board, Move};
use rand::RngExt;
use rand::rngs::SmallRng;

const MAX_GAME_LENGTH: usize = 1024;

pub struct GameStruct {
    pub b: Board,
    undo_stack: [Undo; MAX_GAME_LENGTH],
    ply: usize,
}

impl GameStruct {
    pub fn new() -> Self {
        Self {
            b: Board::new(),
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
        let mut moves = MoveList::new();
        self.b.get_legal_moves_list(&mut moves);
        if moves.count == 0 || self.is_draw() {
            return None;
        }
        let index = rng.random_range(0..moves.count);
        moves.get(index)
    }

    pub fn perft(&mut self, depth: u8, move_lists: &mut [MoveList]) -> u64 {
        if depth == 0 {
            return 1;
        }

        if depth == 1 {
            let mut count = 0;
            self.b.feed_legal_moves(|_| {
                count += 1;
            });
            return count;
        }

        let moves = &mut move_lists[depth as usize];
        self.b.get_legal_moves_list(moves);
        let count = moves.count;
        let mut nodes = 0;
        for i in 0..count {
            let mv = move_lists[depth as usize].get(i).unwrap();
            self.make_move(mv);
            nodes += self.perft(depth - 1, move_lists);
            self.unmake_move();
        }
        nodes
    }
}

impl Default for GameStruct {
    fn default() -> Self {
        Self::new()
    }
}
