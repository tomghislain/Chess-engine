use crate::engine::types::{
    CAPTURE_FLAG, CASTLING_FLAG, Color, DOUBLE_PAWN_PUSH_FLAG, EN_PASSANT_FLAG, Move,
    PROMOTION_FLAG, Piece,
};
use crate::engine::{Square, zobrist};
pub const WK_CASTLE: u8 = 1 << 0;
pub const WQ_CASTLE: u8 = 1 << 1;
pub const BK_CASTLE: u8 = 1 << 2;
pub const BQ_CASTLE: u8 = 1 << 3;

#[derive(Clone)]
pub struct Undo {
    pub captured_piece: Piece,
    pub castling: u8,
    pub en_passant: Square,
    pub halfmove_clock: u8,
    pub zobrist: u64,
}

#[derive(Clone)]
pub struct Board {
    pub pieces: [u64; 12],
    pub color_occ: [u64; 2],
    pub mailbox: [Piece; 64],
    pub occ: u64,

    pub side_to_move: Color,
    pub castling: u8,
    pub en_passant: Square,
    pub halfmove_clock: u8,
    pub zobrist: u64,

    pub history: Vec<u64>,
    pub undo_stack: Vec<Undo>,
}

impl Undo {
    pub fn new(board: &Board, captured_piece: Piece) -> Self {
        Undo {
            captured_piece,
            castling: board.castling,
            en_passant: board.en_passant,
            halfmove_clock: board.halfmove_clock,
            zobrist: board.zobrist,
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
            while pieces != 0 {
                let sq = pieces.trailing_zeros();
                pieces &= pieces - 1;
                hash ^= z.piece_square[i][sq as usize];
            }
        }

        // Side to move
        if self.side_to_move == Color::Black {
            hash ^= z.side;
        }

        // Castling rights
        if self.castling & WK_CASTLE != 0 {
            hash ^= z.castling[0];
        }
        if self.castling & WQ_CASTLE != 0 {
            hash ^= z.castling[1];
        }
        if self.castling & BK_CASTLE != 0 {
            hash ^= z.castling[2];
        }
        if self.castling & BQ_CASTLE != 0 {
            hash ^= z.castling[3];
        }

        // En passant
        if self.en_passant.as_u8() < 64 {
            let file = self.en_passant.as_u8() % 8;
            hash ^= z.en_passant[file as usize];
        }

        hash
    }
    // Function to create a board at the default position
    pub fn new() -> Self {
        let mut pieces = [0u64; 12];

        pieces[Piece::WP as usize] = (1 << 8)
            | (1 << 9)
            | (1 << 10)
            | (1 << 11)
            | (1 << 12)
            | (1 << 13)
            | (1 << 14)
            | (1 << 15);
        pieces[Piece::WR as usize] = (1 << 0) | (1 << 7);
        pieces[Piece::WN as usize] = (1 << 1) | (1 << 6);
        pieces[Piece::WB as usize] = (1 << 2) | (1 << 5);
        pieces[Piece::WQ as usize] = 1 << 3;
        pieces[Piece::WK as usize] = 1 << 4;

        pieces[Piece::BP as usize] = (1 << 48)
            | (1 << 49)
            | (1 << 50)
            | (1 << 51)
            | (1 << 52)
            | (1 << 53)
            | (1 << 54)
            | (1 << 55);
        pieces[Piece::BR as usize] = (1 << 56) | (1 << 63);
        pieces[Piece::BN as usize] = (1 << 57) | (1 << 62);
        pieces[Piece::BB as usize] = (1 << 58) | (1 << 61);
        pieces[Piece::BQ as usize] = 1 << 59;
        pieces[Piece::BK as usize] = 1 << 60;

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

        let occ = color_occ[0] | color_occ[1];

        let side_to_move = Color::White;

        let castling = WK_CASTLE | WQ_CASTLE | BK_CASTLE | BQ_CASTLE;

        let en_passant = Square::from(64);
        let halfmove_clock = 0;
        let zobrist = 0;
        let history = Vec::with_capacity(256);
        let undo_stack = Vec::with_capacity(256);

        let mut board = Board {
            pieces,
            mailbox,
            color_occ,
            occ,
            side_to_move,
            castling,
            en_passant,
            halfmove_clock,
            zobrist,
            history,
            undo_stack,
        };

        board.zobrist = board.compute_zobrist();
        board.history.push(board.zobrist);
        board
    }
    pub fn piece_at(&self, sq: Square) -> Piece {
        self.mailbox[sq.as_usize()]
    }

    // Sets the bit to 0 at index sq of the bitmap at index p of self.pieces
    fn remove_piece(&mut self, p: Piece, sq: Square) {
        let mask = 1 << sq.as_u8();
        self.pieces[p as usize] &= !mask;
    }

    // Sets the bit to 1 at index sq of the bitmap at index p of self.pieces
    fn add_piece(&mut self, p: Piece, sq: Square) {
        let mask = 1 << sq.as_u8();
        self.pieces[p as usize] |= mask;
    }

    // Flips the bits at the from and to index in the bitmap at index p of self.pieces
    fn move_piece(&mut self, p: Piece, from: Square, to: Square) {
        let mask = 1 << from.as_u8() | 1 << to.as_u8();
        self.pieces[p as usize] ^= mask;
    }

    pub fn make_move(&mut self, mv: Move) {
        let z = zobrist::zobrist();
        let old_hash = self.zobrist;
        let mut hash = old_hash;

        let flags = mv.flags();
        let is_capture = flags & CAPTURE_FLAG != 0;
        let is_promotion = flags & PROMOTION_FLAG != 0;
        let is_en_passant = flags & EN_PASSANT_FLAG != 0;
        let is_castling = flags & CASTLING_FLAG != 0;
        let is_double_pawn_push = flags & DOUBLE_PAWN_PUSH_FLAG != 0;
        let mut moving_side_mask = 0u64;
        let mut opponent_side_mask = 0u64;
        let us = self.side_to_move;

        // Remove piece if capture
        let captured_piece = if is_capture {
            let c_sq = if is_en_passant {
                if us == Color::White {
                    Square::from(mv.to().as_u8() - 8)
                } else {
                    Square::from(mv.to().as_u8() + 8)
                }
            } else {
                mv.to()
            };
            let c_p = self.piece_at(c_sq);
            self.remove_piece(c_p, c_sq);
            self.mailbox[c_sq.as_usize()] = Piece::None;
            let mask = 1u64 << c_sq.as_u8();
            opponent_side_mask |= mask;
            hash ^= z.piece_square[c_p as usize][c_sq.as_usize()];
            c_p
        } else {
            Piece::None
        };

        // Save undo information
        let undo = Undo::new(self, captured_piece);
        self.undo_stack.push(undo);

        // Move the played piece (plus rook for castling)
        let moved_piece = self.piece_at(mv.from());
        let moved_mask = (1u64 << mv.from().as_u8()) | (1u64 << mv.to().as_u8());
        moving_side_mask |= moved_mask;
        hash ^= z.piece_square[moved_piece as usize][mv.from().as_usize()];

        if is_promotion {
            let promotion_piece = mv.promotion();
            self.remove_piece(moved_piece, mv.from());
            self.mailbox[mv.from().as_usize()] = Piece::None;
            self.add_piece(promotion_piece, mv.to());
            self.mailbox[mv.to().as_usize()] = promotion_piece;
            hash ^= z.piece_square[promotion_piece as usize][mv.to().as_usize()];
        } else {
            self.move_piece(moved_piece, mv.from(), mv.to());
            self.mailbox[mv.from().as_usize()] = Piece::None;
            self.mailbox[mv.to().as_usize()] = moved_piece;
            hash ^= z.piece_square[moved_piece as usize][mv.to().as_usize()];

            if is_castling {
                let (rook, rook_from, rook_to) = match mv.to().as_u8() {
                    62 => (Piece::BR, Square::from(63), Square::from(61)), // black king side
                    58 => (Piece::BR, Square::from(56), Square::from(59)), // black queen side
                    6 => (Piece::WR, Square::from(7), Square::from(5)),    // white king side
                    2 => (Piece::WR, Square::from(0), Square::from(3)),    // white queen side
                    _ => unreachable!(),
                };

                self.move_piece(rook, rook_from, rook_to);
                self.mailbox[rook_from.as_usize()] = Piece::None;
                self.mailbox[rook_to.as_usize()] = rook;

                let rook_mask = (1u64 << rook_from.as_u8()) | (1u64 << rook_to.as_u8());
                moving_side_mask |= rook_mask;
                hash ^= z.piece_square[rook as usize][rook_from.as_usize()];
                hash ^= z.piece_square[rook as usize][rook_to.as_usize()];
            }
        }

        // Update occupancy
        self.occ ^= moving_side_mask ^ opponent_side_mask;
        self.color_occ[self.side_to_move as usize] ^= moving_side_mask;
        self.color_occ[self.side_to_move.flip() as usize] ^= opponent_side_mask;

        // Update en passant square
        if self.en_passant.as_u8() < 64 {
            let file = self.en_passant.as_u8() % 8;
            hash ^= z.en_passant[file as usize];
        }
        self.en_passant = Square::from(if is_double_pawn_push {
            (mv.from().as_u8() + mv.to().as_u8()) / 2
        } else {
            64
        });
        if self.en_passant.as_u8() < 64 {
            let file = self.en_passant.as_u8() % 8;
            hash ^= z.en_passant[file as usize];
        }

        // Update castling rights
        let old_castling = self.castling;
        if old_castling & WK_CASTLE != 0 {
            hash ^= z.castling[0];
        }
        if old_castling & WQ_CASTLE != 0 {
            hash ^= z.castling[1];
        }
        if old_castling & BK_CASTLE != 0 {
            hash ^= z.castling[2];
        }
        if old_castling & BQ_CASTLE != 0 {
            hash ^= z.castling[3];
        }

        self.castling &= match moved_piece {
            Piece::WK => !(WK_CASTLE | WQ_CASTLE),
            Piece::BK => !(BK_CASTLE | BQ_CASTLE),
            Piece::WR => {
                if mv.from().as_u8() == 7 {
                    !WK_CASTLE
                } else if mv.from().as_u8() == 0 {
                    !WQ_CASTLE
                } else {
                    !0
                }
            }
            Piece::BR => {
                if mv.from().as_u8() == 63 {
                    !BK_CASTLE
                } else if mv.from().as_u8() == 56 {
                    !BQ_CASTLE
                } else {
                    !0
                }
            }
            _ => !0,
        };

        if is_capture {
            self.castling &= match captured_piece {
                Piece::WR => {
                    if mv.to().as_u8() == 7 {
                        !WK_CASTLE
                    } else if mv.to().as_u8() == 0 {
                        !WQ_CASTLE
                    } else {
                        !0
                    }
                }
                Piece::BR => {
                    if mv.to().as_u8() == 63 {
                        !BK_CASTLE
                    } else if mv.to().as_u8() == 56 {
                        !BQ_CASTLE
                    } else {
                        !0
                    }
                }
                _ => !0,
            };
        }

        if self.castling & WK_CASTLE != 0 {
            hash ^= z.castling[0];
        }
        if self.castling & WQ_CASTLE != 0 {
            hash ^= z.castling[1];
        }
        if self.castling & BK_CASTLE != 0 {
            hash ^= z.castling[2];
        }
        if self.castling & BQ_CASTLE != 0 {
            hash ^= z.castling[3];
        }

        // Update halfmove clock
        let is_pawn_move = is_double_pawn_push || matches!(moved_piece, Piece::WP | Piece::BP);
        self.halfmove_clock = if is_capture || is_pawn_move {
            0
        } else {
            self.halfmove_clock + 1
        };

        // Update side_to_move
        self.side_to_move = self.side_to_move.flip();
        hash ^= z.side;

        // Update hash
        self.zobrist = hash;
        self.history.push(self.zobrist);
    }

    pub fn unmake_move(&mut self, mv: Move) {
        // Pop undo stack and restore previous state
        let undo = self.undo_stack.pop().unwrap();
        self.castling = undo.castling;
        self.en_passant = undo.en_passant;
        self.halfmove_clock = undo.halfmove_clock;
        self.zobrist = undo.zobrist;
        self.history.pop();
        self.side_to_move = self.side_to_move.flip();

        let flags = mv.flags();
        let is_capture = flags & CAPTURE_FLAG != 0;
        let is_promotion = flags & PROMOTION_FLAG != 0;
        let is_en_passant = flags & EN_PASSANT_FLAG != 0;
        let is_castling = flags & CASTLING_FLAG != 0;
        let mut moving_side_mask: u64 = 0;
        let mut opponent_side_mask: u64 = 0;

        // Move back the piece (checking for promotion and castling)
        moving_side_mask |= (1u64 << mv.to().as_u8()) | (1u64 << mv.from().as_u8());
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
                    62 => (Piece::BR, Square::from(61), Square::from(63)), // black king side
                    58 => (Piece::BR, Square::from(59), Square::from(56)), // black queen side
                    6 => (Piece::WR, Square::from(5), Square::from(7)),    // white king side
                    2 => (Piece::WR, Square::from(3), Square::from(0)),    // white queen side
                    _ => unreachable!(),
                };

                self.move_piece(rook, rook_from, rook_to);
                self.mailbox[rook_from.as_usize()] = Piece::None;
                self.mailbox[rook_to.as_usize()] = rook;

                let rook_mask = (1u64 << rook_from.as_u8()) | (1u64 << rook_to.as_u8());
                moving_side_mask |= rook_mask;
            }
        }

        // In case of capture (and maybe en passant), restore the taken piece
        if is_capture {
            let c_p = undo.captured_piece;
            let c_sq = if is_en_passant {
                match self.side_to_move {
                    Color::White => Square::from(mv.to().as_u8() - 8),
                    Color::Black => Square::from(mv.to().as_u8() + 8),
                }
            } else {
                mv.to()
            };
            let sq_mask = 1u64 << c_sq.as_u8();
            opponent_side_mask |= sq_mask;
            self.add_piece(c_p, c_sq);
            self.mailbox[c_sq.as_usize()] = c_p;
        }

        // Recompute occupancy
        self.occ ^= moving_side_mask ^ opponent_side_mask;
        self.color_occ[self.side_to_move as usize] ^= moving_side_mask;
        self.color_occ[self.side_to_move.flip() as usize] ^= opponent_side_mask;
    }

    fn is_50_move_draw(&self) -> bool {
        self.halfmove_clock >= 100
    }

    fn is_threefold_repetition(&self) -> bool {
        let current = self.zobrist;
        let limit = self.halfmove_clock as usize / 2;
        self.history
            .iter()
            .rev()
            .step_by(2)
            .take(limit)
            .filter(|&&h| h == current)
            .nth(2)
            .is_some()
    }

    fn bishops_same_color(&self, bishops: u64) -> bool {
        const LIGHT_SQUARES: u64 = 0x55AA55AA55AA55AA;
        (bishops & LIGHT_SQUARES) == bishops || (bishops & LIGHT_SQUARES) == 0
    }

    fn insufficient_minors(&self, knights: u64, bishops: u64) -> bool {
        if knights != 0 && bishops == 0 {
            true
        } else if bishops != 0 && knights == 0 {
            self.bishops_same_color(bishops)
        } else {
            false
        }
    }

    fn is_insufficient_material(&self) -> bool {
        if (self.pieces[Piece::WP as usize]
            | self.pieces[Piece::BP as usize]
            | self.pieces[Piece::WR as usize]
            | self.pieces[Piece::BR as usize]
            | self.pieces[Piece::WQ as usize]
            | self.pieces[Piece::BQ as usize])
            != 0
        {
            return false;
        }
        let w_knights = self.pieces[Piece::WN as usize];
        let b_knights = self.pieces[Piece::BN as usize];
        let w_bishops = self.pieces[Piece::WB as usize];
        let b_bishops = self.pieces[Piece::BB as usize];

        let white_has_minors = (w_knights | w_bishops) != 0;
        let black_has_minors = (b_knights | b_bishops) != 0;

        match (white_has_minors, black_has_minors) {
            (false, false) => true,
            (false, true) => self.insufficient_minors(b_knights, b_bishops),
            (true, false) => self.insufficient_minors(w_knights, w_bishops),
            (true, true) => {
                if (w_knights | b_knights) != 0 {
                    false
                } else {
                    let bishops = w_bishops | b_bishops;
                    self.bishops_same_color(bishops)
                }
            }
        }
    }

    pub fn is_draw(&self) -> bool {
        self.is_50_move_draw() || self.is_threefold_repetition() || self.is_insufficient_material()
    }
    pub fn perft(&mut self, depth: u8) -> u64 {
        let mut moves = Vec::with_capacity(218);
        self.get_legal_moves(&mut moves);
        if depth == 0 {
            return 1;
        } else if depth == 1 {
            return moves.len() as u64;
        }

        let mut nodes = 0;

        for mv in moves {
            // println!("Making move at depth {}: {:?}", depth, m);
            self.make_move(mv);
            nodes += self.perft(depth - 1);
            self.unmake_move(mv);
            // println!("Unmaking move at depth {}: {:?}", depth, m);
        }
        nodes
    }

    pub fn is_king_checked(&self) -> bool {
        let attacker_color = self.side_to_move.flip();
        let king_sq = match attacker_color {
            Color::Black => Square::from(self.pieces[Piece::WK as usize].trailing_zeros() as u8),
            Color::White => Square::from(self.pieces[Piece::BK as usize].trailing_zeros() as u8),
        };
        self.compute_checkers(king_sq, self.occ, attacker_color, &self.pieces) != 0
    }
}
