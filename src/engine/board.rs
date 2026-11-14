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

#[derive(Clone, Copy, PartialEq, Eq)]
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

    pub fn from_fen(fen: &str) -> Self {
        let mut f = fen.split_whitespace();

        let placement_str = f.next().expect("No placement");
        let side_str = f.next().expect("No side to move");
        let castling_str = f.next().expect("No castling");
        let en_passant_str = f.next().expect("No en passant");
        let halfmove_str = f.next().unwrap_or("0");
        // Fullmove is ignored as Board does not store it

        let mut pieces = [Bitboard(0); 12];
        let mut mailbox = [Piece::None; 64];
        let mut ranks = 0;
        for (i, rank_str) in placement_str.split('/').enumerate() {
            assert!(i < 8, "Too many ranks");
            let rank = 7 - i;
            let mut file = 0;
            for c in rank_str.chars() {
                if c.is_ascii_digit() {
                    let d = c.to_digit(10).unwrap();
                    assert!((1..=8).contains(&d), "Invalid digit in placement");
                    file += d;
                } else {
                    let p = match c {
                        'P' => Piece::WP,
                        'N' => Piece::WN,
                        'B' => Piece::WB,
                        'R' => Piece::WR,
                        'Q' => Piece::WQ,
                        'K' => Piece::WK,
                        'p' => Piece::BP,
                        'n' => Piece::BN,
                        'b' => Piece::BB,
                        'r' => Piece::BR,
                        'q' => Piece::BQ,
                        'k' => Piece::BK,
                        _ => panic!("Invalid piece char"),
                    };
                    assert!(file < 8, "Invalid rank length");
                    let sq = (rank * 8 + file as usize) as u8;
                    pieces[p as usize] |= Bitboard(1u64 << sq);
                    mailbox[sq as usize] = p;
                    file += 1;
                }
            }
            assert!(file == 8, "Invalid file width");
            ranks += 1;
        }
        assert!(ranks == 8, "Invalid rank amount");

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
        let occ = color_occ[0] | color_occ[1];

        let side_to_move = match side_str {
            "w" => Color::White,
            "b" => Color::Black,
            _ => panic!("Invalid side to move"),
        };

        let mut castling = 0u8;
        if castling_str != "-" {
            for c in castling_str.chars() {
                castling |= match c {
                    'K' => WK_CASTLE,
                    'Q' => WQ_CASTLE,
                    'k' => BK_CASTLE,
                    'q' => BQ_CASTLE,
                    _ => panic!("Invalid castling str"),
                }
            }
        }

        let en_passant = if en_passant_str == "-" {
            Square::NO_SQUARE
        } else {
            let b = en_passant_str.as_bytes();
            let f = b[0].wrapping_sub(b'a');
            let r = b[1].wrapping_sub(b'1');
            assert!(f < 8 && r < 8, "Invalid en passant str");
            Square(r * 8 + f)
        };

        let halfmove_clock = halfmove_str.parse().expect("Invalid halfmove str");

        let mut b = Self {
            pieces,
            color_occ,
            mailbox,
            occ,
            side_to_move,
            castling,
            en_passant,
            halfmove_clock,
            hash: 0,
        };
        b.hash = b.compute_zobrist();
        b
    }

    #[inline(always)]
    pub fn piece_at(&self, sq: Square) -> Piece {
        debug_assert!(sq.as_u8() < 64);
        unsafe { *self.mailbox.get_unchecked(sq.as_usize()) }
    }

    // Sets the bit to 0 at index sq of the bitmap at index p of self.pieces
    fn remove_piece(&mut self, p: Piece, sq: Square) {
        debug_assert!(p != Piece::None);
        debug_assert!(sq.as_u8() < 64);
        let mask = Bitboard::from(sq);
        unsafe {
            *self.pieces.get_unchecked_mut(p as usize) &= !mask;
            *self.mailbox.get_unchecked_mut(sq.as_usize()) = Piece::None;
        }
    }

    // Sets the bit to 1 at index sq of the bitmap at index p of self.pieces
    fn add_piece(&mut self, p: Piece, sq: Square) {
        debug_assert!(p != Piece::None);
        debug_assert!(sq.as_u8() < 64);
        let mask = Bitboard::from(sq);
        unsafe {
            *self.pieces.get_unchecked_mut(p as usize) |= mask;
            *self.mailbox.get_unchecked_mut(sq.as_usize()) = p;
        }
    }

    // Flips the bits at the from and to index in the bitmap at index p of self.pieces
    fn move_piece(&mut self, p: Piece, from: Square, to: Square) {
        debug_assert!(p != Piece::None);
        let mask = Bitboard::from(from) | Bitboard::from(to);
        unsafe {
            *self.pieces.get_unchecked_mut(p as usize) ^= mask;
            *self.mailbox.get_unchecked_mut(from.as_usize()) = Piece::None;
            *self.mailbox.get_unchecked_mut(to.as_usize()) = p;
        }
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
            self.add_piece(promotion_piece, mv.to());
            self.hash ^= z.piece_square[promotion_piece as usize][mv.to().as_usize()];
        } else {
            self.move_piece(moved_piece, mv.from(), mv.to());
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
        let is_pawn_move = matches!(moved_piece, Piece::WP | Piece::BP);
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
        debug_assert!(undo != Undo::default());
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

            match self.side_to_move {
                Color::White => {
                    self.add_piece(Piece::WP, mv.from());
                }
                Color::Black => {
                    self.add_piece(Piece::BP, mv.from());
                }
            }
        } else {
            let moved_piece = self.piece_at(mv.to());
            self.move_piece(moved_piece, mv.to(), mv.from());

            if is_castling {
                let (rook, rook_from, rook_to) = match mv.to().as_u8() {
                    62 => (Piece::BR, Square(61), Square(63)), // black king side
                    58 => (Piece::BR, Square(59), Square(56)), // black queen side
                    6 => (Piece::WR, Square(5), Square(7)),    // white king side
                    2 => (Piece::WR, Square(3), Square(0)),    // white queen side
                    _ => unreachable!(),
                };

                self.move_piece(rook, rook_from, rook_to);

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

    #[inline(always)]
    pub fn count_legal_moves(&self) -> u64 {
        let mut count = 0;
        self.feed_legal_moves(|_| count += 1);
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
            return self.count_legal_moves();
        }

        let mut nodes = 0;
        self.feed_legal_moves(|mv| {
            let mut child = *self;
            child.make_move(mv);
            nodes += child.perft(depth - 1);
        });
        nodes
    }
}

impl Default for Board {
    // fn default() -> Self {
    //     Self::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
    // }

    fn default() -> Self {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_position() {
        let b = Board::default();
        assert!(b.perft(0) == 1);
        assert!(b.perft(1) == 20);
        assert!(b.perft(2) == 400);
        assert!(b.perft(3) == 8_902);
        assert!(b.perft(4) == 197_281);
        assert!(b.perft(5) == 4_865_609);
        assert!(b.perft(6) == 119_060_324);
        assert!(b.perft(7) == 3_195_901_860);
        // assert!(b.perft(8) == 84_998_976_956);
        // assert!(b.perft(9) == 2_439_530_234_167);
    }

    #[test]
    fn kiwipete() {
        let b =
            Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - ");
        assert!(b.perft(1) == 48);
        assert!(b.perft(2) == 2_039);
        assert!(b.perft(3) == 97_862);
        assert!(b.perft(4) == 4_085_603);
        assert!(b.perft(5) == 193_690_690);
        assert!(b.perft(6) == 8_031_647_685);
    }

    #[test]
    fn postition3() {
        let b = Board::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1 ");
        assert!(b.perft(1) == 14);
        assert!(b.perft(2) == 191);
        assert!(b.perft(3) == 2_812);
        assert!(b.perft(4) == 43_238);
        assert!(b.perft(5) == 674_624);
        assert!(b.perft(6) == 11_030_083);
        assert!(b.perft(7) == 178_633_661);
        assert!(b.perft(8) == 3_009_794_393);
    }

    #[test]
    fn position4() {
        let b = Board::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
        assert!(b.perft(1) == 6);
        assert!(b.perft(2) == 264);
        assert!(b.perft(3) == 9_467);
        assert!(b.perft(4) == 422_333);
        assert!(b.perft(5) == 15_833_292);
        assert!(b.perft(6) == 706_045_033);
    }

    #[test]
    fn position5() {
        let b = Board::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8  ");
        assert!(b.perft(1) == 44);
        assert!(b.perft(2) == 1_486);
        assert!(b.perft(3) == 62_379);
        assert!(b.perft(4) == 2_103_487);
        assert!(b.perft(5) == 89_941_194);
    }
}
