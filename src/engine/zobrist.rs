use rand::{self, Rng, SeedableRng, rngs::StdRng};
use std::sync::OnceLock;

pub struct Zobrist {
    pub piece_square: [[u64; 64]; 12],
    pub side: u64,
    pub castling: [u64; 16],
    pub en_passant: [u64; 8],
}

static ZOBRIST: OnceLock<Zobrist> = OnceLock::new();

fn init_zobrist() -> Zobrist {
    let mut rng = StdRng::seed_from_u64(314159265);
    let mut piece_square = [[0; 64]; 12];
    for i in 0..12 {
        for j in 0..64 {
            piece_square[i][j] = rng.next_u64();
        }
    }
    let side = rng.next_u64();
    let mut castling = [0; 16];
    for i in 0..16 {
        castling[i] = rng.next_u64();
    }
    let mut en_passant = [0; 8];
    for i in 0..8 {
        en_passant[i] = rng.next_u64();
    }
    Zobrist {
        piece_square,
        side,
        castling,
        en_passant,
    }
}

pub fn zobrist() -> &'static Zobrist {
    ZOBRIST.get_or_init(init_zobrist)
}
