use crate::engine::types::{
    Bitboard, CAPTURE_FLAG, CASTLING_FLAG, Color, DOUBLE_PAWN_PUSH_FLAG, EN_PASSANT_FLAG, Move,
    PROMOTION_FLAG, Piece,
};
use crate::engine::{Square, zobrist};
pub const WK_CASTLE: u8 = 1 << 0;
pub const WQ_CASTLE: u8 = 1 << 1;
pub const BK_CASTLE: u8 = 1 << 2;
pub const BQ_CASTLE: u8 = 1 << 3;

const CASTLING_UPDATE: [u8; 64] = [
    !WQ_CASTLE,
    !0,
    !0,
    !0,
    !(WQ_CASTLE | WK_CASTLE),
    !0,
    !0,
    !WK_CASTLE,
    !0,
    !0,
    !0,
    !0,
    !0,
    !0,
    !0,
    !0,
    !0,
    !0,
    !0,
    !0,
    !0,
    !0,
    !0,
    !0,
    !0,
    !0,
    !0,
    !0,
    !0,
    !0,
    !0,
    !0,
    !0,
    !0,
    !0,
    !0,
    !0,
    !0,
    !0,
    !0,
    !0,
    !0,
    !0,
    !0,
    !0,
    !0,
    !0,
    !0,
    !0,
    !0,
    !0,
    !0,
    !0,
    !0,
    !0,
    !0,
    !BQ_CASTLE,
    !0,
    !0,
    !0,
    !(BQ_CASTLE | BK_CASTLE),
    !0,
    !0,
    !BK_CASTLE,
];

#[derive(Clone, Copy)]
pub struct Board {
    pub pieces: [Bitboard; 12],
    pub color_occ: [Bitboard; 2],
    pub mailbox: [Piece; 64],
    pub occ: Bitboard,

    pub side_to_move: Color,

    pub castling: u8,
    pub en_passant: Square,
    pub halfmove_clock: u8,
    pub hash: u64,
}

#[derive(Clone, Copy)]
pub struct Undo {
    mv: Move,
    captured_piece: Piece,
    castling: u8,
    en_passant: Square,
    halfmove_clock: u8,
    pub hash: u64,
}

impl Default for Undo {
    fn default() -> Self {
        Self {
            mv: Move::default(),
            captured_piece: Piece::None,
            castling: 0,
            en_passant: Square::NO_SQUARE,
            halfmove_clock: 0,
            hash: 0,
        }
    }
}

impl Board {
    fn compute_zobrist(&self) -> u64 {
        let z = zobrist::zobrist();
        let mut hash = 0;

        // Pieces
        for (i, &bb) in self.pieces.iter().enumerate() {
            let mut pieces = bb;
            while pieces.0 != 0 {
                let sq = pieces.pop_lsb();
                hash ^= z.piece_square[i][sq as usize];
            }
        }

        // Side to move
        if self.side_to_move == Color::Black {
            hash ^= z.side;
        }

        // Castling rights
        hash ^= z.castling[self.castling as usize];

        // En passant
        if self.en_passant != Square::NO_SQUARE {
            let file = self.en_passant.as_u8() % 8;
            hash ^= z.en_passant[file as usize];
        }

        hash
    }

    pub fn new() -> Self {
        let mut pieces = [Bitboard(0); 12];

        pieces[Piece::WP as usize] = Bitboard(
            (1 << 8)
                | (1 << 9)
                | (1 << 10)
                | (1 << 11)
                | (1 << 12)
                | (1 << 13)
                | (1 << 14)
                | (1 << 15),
        );
        pieces[Piece::WR as usize] = Bitboard((1 << 0) | (1 << 7));
        pieces[Piece::WN as usize] = Bitboard((1 << 1) | (1 << 6));
        pieces[Piece::WB as usize] = Bitboard((1 << 2) | (1 << 5));
        pieces[Piece::WQ as usize] = Bitboard(1 << 3);
        pieces[Piece::WK as usize] = Bitboard(1 << 4);

        pieces[Piece::BP as usize] = Bitboard(
            (1 << 48)
                | (1 << 49)
                | (1 << 50)
                | (1 << 51)
                | (1 << 52)
                | (1 << 53)
                | (1 << 54)
                | (1 << 55),
        );
        pieces[Piece::BR as usize] = Bitboard((1 << 56) | (1 << 63));
        pieces[Piece::BN as usize] = Bitboard((1 << 57) | (1 << 62));
        pieces[Piece::BB as usize] = Bitboard((1 << 58) | (1 << 61));
        pieces[Piece::BQ as usize] = Bitboard(1 << 59);
        pieces[Piece::BK as usize] = Bitboard(1 << 60);

        let mailbox = [
            Piece::WR,
            Piece::WN,
            Piece::WB,
            Piece::WQ,
            Piece::WK,
            Piece::WB,
            Piece::WN,
            Piece::WR,
            Piece::WP,
            Piece::WP,
            Piece::WP,
            Piece::WP,
            Piece::WP,
            Piece::WP,
            Piece::WP,
            Piece::WP,
            Piece::None,
            Piece::None,
            Piece::None,
            Piece::None,
            Piece::None,
            Piece::None,
            Piece::None,
            Piece::None,
            Piece::None,
            Piece::None,
            Piece::None,
            Piece::None,
            Piece::None,
            Piece::None,
            Piece::None,
            Piece::None,
            Piece::None,
            Piece::None,
            Piece::None,
            Piece::None,
            Piece::None,
            Piece::None,
            Piece::None,
            Piece::None,
            Piece::None,
            Piece::None,
            Piece::None,
            Piece::None,
            Piece::None,
            Piece::None,
            Piece::None,
            Piece::None,
            Piece::BP,
            Piece::BP,
            Piece::BP,
            Piece::BP,
            Piece::BP,
            Piece::BP,
            Piece::BP,
            Piece::BP,
            Piece::BR,
            Piece::BN,
            Piece::BB,
            Piece::BQ,
            Piece::BK,
            Piece::BB,
            Piece::BN,
            Piece::BR,
        ];

        let color_occ = [
            pieces[Piece::WP as usize]
                | pieces[Piece::WN as usize]
                | pieces[Piece::WB as usize]
                | pieces[Piece::WR as usize]
                | pieces[Piece::WQ as usize]
                | pieces[Piece::WK as usize],
            pieces[Piece::BP as usize]
                | pieces[Piece::BN as usize]
                | pieces[Piece::BB as usize]
                | pieces[Piece::BR as usize]
                | pieces[Piece::BQ as usize]
                | pieces[Piece::BK as usize],
        ];

        let mut board = Self {
            pieces,
            color_occ,
            mailbox,
            occ: color_occ[0] | color_occ[1],
            side_to_move: Color::White,
            castling: WK_CASTLE | WQ_CASTLE | BK_CASTLE | BQ_CASTLE,
            en_passant: Square::NO_SQUARE,
            halfmove_clock: 0,
            hash: 0,
        };

        board.hash = board.compute_zobrist();
        board
    }

    pub const fn piece_at(&self, sq: Square) -> Piece {
        self.mailbox[sq.as_usize()]
    }

    // Sets the bit to 0 at index sq of the bitmap at index p of self.pieces
    fn remove_piece(&mut self, p: Piece, sq: Square) {
        let mask = Bitboard::from(sq);
        self.pieces[p as usize] &= !mask;
    }

    // Sets the bit to 1 at index sq of the bitmap at index p of self.pieces
    fn add_piece(&mut self, p: Piece, sq: Square) {
        let mask = Bitboard::from(sq);
        self.pieces[p as usize] |= mask;
    }

    // Flips the bits at the from and to index in the bitmap at index p of self.pieces
    fn move_piece(&mut self, p: Piece, from: Square, to: Square) {
        let mask = Bitboard::from(from) | Bitboard::from(to);
        self.pieces[p as usize] ^= mask;
    }

    pub fn make_move(&mut self, mv: Move) -> Undo {
        let z = zobrist::zobrist();
        let mut undo = Undo {
            mv,
            captured_piece: Piece::None,
            castling: self.castling,
            en_passant: self.en_passant,
            halfmove_clock: self.halfmove_clock,
            hash: self.hash,
        };

        let flags = mv.flags();
        let is_capture = flags & CAPTURE_FLAG != 0;
        let is_promotion = flags & PROMOTION_FLAG != 0;
        let is_en_passant = flags & EN_PASSANT_FLAG != 0;
        let is_castling = flags & CASTLING_FLAG != 0;
        let is_double_pawn_push = flags & DOUBLE_PAWN_PUSH_FLAG != 0;
        let mut moving_side_mask = Bitboard(0);
        let mut opponent_side_mask = Bitboard(0);
        let side_to_move = self.side_to_move;
        let opponent = side_to_move.flip();

        // Remove piece if capture
        undo.captured_piece = if is_capture {
            let c_sq = if is_en_passant {
                if side_to_move == Color::White {
                    Square(mv.to().as_u8() - 8)
                } else {
                    Square(mv.to().as_u8() + 8)
                }
            } else {
                mv.to()
            };
            let c_p = self.piece_at(c_sq);
            self.remove_piece(c_p, c_sq);
            self.mailbox[c_sq.as_usize()] = Piece::None;
            let mask = Bitboard::from(c_sq);
            opponent_side_mask |= mask;
            self.hash ^= z.piece_square[c_p as usize][c_sq.as_usize()];
            c_p
        } else {
            Piece::None
        };

        // Move the played piece (plus rook for castling)
        let moved_piece = self.piece_at(mv.from());
        let moved_mask = Bitboard::from(mv.from()) | Bitboard::from(mv.to());
        moving_side_mask |= moved_mask;
        self.hash ^= z.piece_square[moved_piece as usize][mv.from().as_usize()];

        if is_promotion {
            let promotion_piece = mv.promotion();
            self.remove_piece(moved_piece, mv.from());
            self.mailbox[mv.from().as_usize()] = Piece::None;
            self.add_piece(promotion_piece, mv.to());
            self.mailbox[mv.to().as_usize()] = promotion_piece;
            self.hash ^= z.piece_square[promotion_piece as usize][mv.to().as_usize()];
        } else {
            self.move_piece(moved_piece, mv.from(), mv.to());
            self.mailbox[mv.from().as_usize()] = Piece::None;
            self.mailbox[mv.to().as_usize()] = moved_piece;
            self.hash ^= z.piece_square[moved_piece as usize][mv.to().as_usize()];

            if is_castling {
                let (rook, rook_from, rook_to) = match mv.to().as_u8() {
                    62 => (Piece::BR, Square(63), Square(61)), // black king side
                    58 => (Piece::BR, Square(56), Square(59)), // black queen side
                    6 => (Piece::WR, Square(7), Square(5)),    // white king side
                    2 => (Piece::WR, Square(0), Square(3)),    // white queen side
                    _ => unreachable!(),
                };

                self.move_piece(rook, rook_from, rook_to);
                self.mailbox[rook_from.as_usize()] = Piece::None;
                self.mailbox[rook_to.as_usize()] = rook;

                let rook_mask = Bitboard::from(rook_from) | Bitboard::from(rook_to);
                moving_side_mask |= rook_mask;
                self.hash ^= z.piece_square[rook as usize][rook_from.as_usize()];
                self.hash ^= z.piece_square[rook as usize][rook_to.as_usize()];
            }
        }

        // Update occupancy
        self.occ ^= moving_side_mask ^ opponent_side_mask;
        self.color_occ[side_to_move as usize] ^= moving_side_mask;
        self.color_occ[opponent as usize] ^= opponent_side_mask;

        // Update en passant square
        if self.en_passant != Square::NO_SQUARE {
            let file = self.en_passant.as_u8() % 8;
            self.hash ^= z.en_passant[file as usize];
        }
        self.en_passant = Square(if is_double_pawn_push {
            u8::midpoint(mv.from().as_u8(), mv.to().as_u8())
        } else {
            64
        });
        if self.en_passant != Square::NO_SQUARE {
            let file = self.en_passant.as_u8() % 8;
            self.hash ^= z.en_passant[file as usize];
        }

        // Update castling rights
        self.hash ^= z.castling[self.castling as usize];

        self.castling &= CASTLING_UPDATE[mv.from().as_usize()];
        self.castling &= CASTLING_UPDATE[mv.to().as_usize()];

        self.hash ^= z.castling[self.castling as usize];

        // Update halfmove clock
        let is_pawn_move = is_double_pawn_push || matches!(moved_piece, Piece::WP | Piece::BP);
        self.halfmove_clock = if is_capture || is_pawn_move {
            0
        } else {
            self.halfmove_clock + 1
        };

        // Update side_to_move
        self.side_to_move = opponent;
        self.hash ^= z.side;

        undo
    }

    pub fn unmake_move(&mut self, undo: Undo) {
        self.side_to_move = self.side_to_move.flip();

        let mv = undo.mv;
        let flags = mv.flags();
        let is_capture = flags & CAPTURE_FLAG != 0;
        let is_promotion = flags & PROMOTION_FLAG != 0;
        let is_en_passant = flags & EN_PASSANT_FLAG != 0;
        let is_castling = flags & CASTLING_FLAG != 0;
        let mut moving_side_mask = Bitboard(0);
        let mut opponent_side_mask = Bitboard(0);

        // Move back the piece (checking for promotion and castling)
        moving_side_mask |= Bitboard::from(mv.to()) | Bitboard::from(mv.from());
        if is_promotion {
            let promotion_piece = mv.promotion();
            self.remove_piece(promotion_piece, mv.to());
            self.mailbox[mv.to().as_usize()] = Piece::None;

            match self.side_to_move {
                Color::White => {
                    self.add_piece(Piece::WP, mv.from());
                    self.mailbox[mv.from().as_usize()] = Piece::WP;
                }
                Color::Black => {
                    self.add_piece(Piece::BP, mv.from());
                    self.mailbox[mv.from().as_usize()] = Piece::BP;
                }
            }
        } else {
            let moved_piece = self.piece_at(mv.to());
            self.move_piece(moved_piece, mv.to(), mv.from());
            self.mailbox[mv.to().as_usize()] = Piece::None;
            self.mailbox[mv.from().as_usize()] = moved_piece;

            if is_castling {
                let (rook, rook_from, rook_to) = match mv.to().as_u8() {
                    62 => (Piece::BR, Square(61), Square(63)), // black king side
                    58 => (Piece::BR, Square(59), Square(56)), // black queen side
                    6 => (Piece::WR, Square(5), Square(7)),    // white king side
                    2 => (Piece::WR, Square(3), Square(0)),    // white queen side
                    _ => unreachable!(),
                };

                self.move_piece(rook, rook_from, rook_to);
                self.mailbox[rook_from.as_usize()] = Piece::None;
                self.mailbox[rook_to.as_usize()] = rook;

                let rook_mask = Bitboard::from(rook_from) | Bitboard::from(rook_to);
                moving_side_mask |= rook_mask;
            }
        }

        // In case of capture (and maybe en passant), restore the taken piece
        if is_capture {
            let c_p = undo.captured_piece;
            let c_sq = if is_en_passant {
                match self.side_to_move {
                    Color::White => Square(mv.to().as_u8() - 8),
                    Color::Black => Square(mv.to().as_u8() + 8),
                }
            } else {
                mv.to()
            };
            let sq_mask = Bitboard::from(c_sq);
            opponent_side_mask |= sq_mask;
            self.add_piece(c_p, c_sq);
            self.mailbox[c_sq.as_usize()] = c_p;
        }

        // Recompute occupancy
        self.occ ^= moving_side_mask ^ opponent_side_mask;
        self.color_occ[self.side_to_move as usize] ^= moving_side_mask;
        self.color_occ[self.side_to_move.flip() as usize] ^= opponent_side_mask;

        // Restore irreversible variables
        self.castling = undo.castling;
        self.en_passant = undo.en_passant;
        self.halfmove_clock = undo.halfmove_clock;
        self.hash = undo.hash;
    }

    pub fn is_king_checked(&self) -> bool {
        let attacker = self.side_to_move.flip();
        let king_sq = match attacker {
            Color::Black => Square(self.pieces[Piece::WK as usize].trailing_zeros()),
            Color::White => Square(self.pieces[Piece::BK as usize].trailing_zeros()),
        };
        self.is_square_attacked(king_sq, attacker)
    }

    const fn is_50_move_draw(&self) -> bool {
        self.halfmove_clock >= 100
    }

    fn bishops_same_color(bishops: Bitboard) -> bool {
        const LIGHT_SQUARES: Bitboard = Bitboard(0x55_AA_55_AA_55_AA_55_AA);
        (bishops & LIGHT_SQUARES) == bishops || (bishops & LIGHT_SQUARES).is_empty()
    }

    fn insufficient_minors(knights: Bitboard, bishops: Bitboard) -> bool {
        if !knights.is_empty() && !bishops.is_empty() {
            false
        } else if !bishops.is_empty() && knights.is_empty() {
            Self::bishops_same_color(bishops)
        } else {
            knights.count_ones() == 1
        }
    }

    fn is_insufficient_material(&self) -> bool {
        if !(self.pieces[Piece::WP as usize]
            | self.pieces[Piece::BP as usize]
            | self.pieces[Piece::WR as usize]
            | self.pieces[Piece::BR as usize]
            | self.pieces[Piece::WQ as usize]
            | self.pieces[Piece::BQ as usize])
            .is_empty()
        {
            return false;
        }
        let w_knights = self.pieces[Piece::WN as usize];
        let b_knights = self.pieces[Piece::BN as usize];
        let w_bishops = self.pieces[Piece::WB as usize];
        let b_bishops = self.pieces[Piece::BB as usize];

        let white_has_minors = !(w_knights | w_bishops).is_empty();
        let black_has_minors = !(b_knights | b_bishops).is_empty();

        match (white_has_minors, black_has_minors) {
            (false, false) => true,
            (false, true) => Self::insufficient_minors(b_knights, b_bishops),
            (true, false) => Self::insufficient_minors(w_knights, w_bishops),
            (true, true) => {
                if (w_knights | b_knights).is_empty() {
                    let bishops = w_bishops | b_bishops;
                    Self::bishops_same_color(bishops)
                } else {
                    false
                }
            }
        }
    }

    pub fn is_draw(&self) -> bool {
        self.is_50_move_draw() || self.is_insufficient_material()
    }

    pub fn material_evaluation(&self) -> isize {
        let mut score = 0;
        score += 100 * self.pieces[Piece::WP as usize].count_ones() as isize;
        score += 350 * self.pieces[Piece::WN as usize].count_ones() as isize;
        score += 350 * self.pieces[Piece::WB as usize].count_ones() as isize;
        score += 525 * self.pieces[Piece::WR as usize].count_ones() as isize;
        score += 1000 * self.pieces[Piece::WQ as usize].count_ones() as isize;

        score -= 100 * self.pieces[Piece::BP as usize].count_ones() as isize;
        score -= 350 * self.pieces[Piece::BN as usize].count_ones() as isize;
        score -= 350 * self.pieces[Piece::BB as usize].count_ones() as isize;
        score -= 525 * self.pieces[Piece::BR as usize].count_ones() as isize;
        score -= 1000 * self.pieces[Piece::BQ as usize].count_ones() as isize;

        if self.side_to_move == Color::White {
            score
        } else {
            -score
        }
    }

    pub fn count_legal_moves(&self) -> usize {
        let mut count = 0;
        self.feed_legal_moves(|_| {
            count += 1;
        });
        count
    }

    pub fn has_legal_moves(&self) -> bool {
        self.count_legal_moves() != 0
    }

    pub fn perft(&self, depth: u8) -> u64 {
        if depth == 0 {
            return 1;
        }

        if depth == 1 {
            let mut count = 0;
            self.feed_legal_moves(|_| count += 1);
            return count;
        }

        let mut nodes = 0;
        self.feed_legal_moves(|mv| {
            let mut child = self.clone();
            child.make_move(mv);
            nodes += child.perft(depth - 1);
        });
        nodes
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}
