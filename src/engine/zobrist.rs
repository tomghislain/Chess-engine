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
    let mut rng = StdRng::seed_from_u64(314_159_265);
    let piece_square: [[u64; 64]; 12] =
        std::array::from_fn(|_| std::array::from_fn(|_| rng.next_u64()));
    let side = rng.next_u64();
    let castling: [u64; 16] = std::array::from_fn(|_| rng.next_u64());
    let en_passant: [u64; 8] = std::array::from_fn(|_| rng.next_u64());
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
