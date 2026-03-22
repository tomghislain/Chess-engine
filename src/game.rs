use rand::{self, Rng, SeedableRng, rngs::StdRng};
use std::fmt;
use std::sync::OnceLock;

pub const FILE_A: u64 = 0x0101010101010101;
pub const FILE_B: u64 = 0x0202020202020202;
pub const FILE_G: u64 = 0x4040404040404040;
pub const FILE_H: u64 = 0x8080808080808080;
pub const RANK_2: u64 = 0x000000000000FF00;
pub const RANK_7: u64 = 0x00FF000000000000;

#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(u8)]
pub enum Piece {
    WP,
    WN,
    WB,
    WR,
    WQ,
    WK,
    BP,
    BN,
    BB,
    BR,
    BQ,
    BK,
    None,
}

#[derive(Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum Color {
    White = 0,
    Black = 1,
}

pub const WK_CASTLE: u8 = 1 << 0;
pub const WQ_CASTLE: u8 = 1 << 1;
pub const BK_CASTLE: u8 = 1 << 2;
pub const BQ_CASTLE: u8 = 1 << 3;

#[derive(Clone)]
pub struct Undo {
    pub castling: u8,
    pub en_passant: u8,
    pub halfmove_clock: u8,
    pub zobrist: u64,
}

#[derive(Copy, PartialEq, Clone, Debug)]
pub struct Move(u32);

struct Zobrist {
    piece_square: [[u64; 64]; 12],
    side: u64,
    castling: [u64; 4],
    en_passant: [u64; 8],
}

#[derive(Clone)]
pub struct Board {
    pub pieces: [u64; 12],
    pub color_occ: [u64; 2],
    pub mailbox: [Piece; 64],
    pub occ: u64,

    pub side_to_move: Color,
    pub castling: u8,
    pub en_passant: u8,
    pub halfmove_clock: u8,
    pub zobrist: u64,

    pub history: Vec<u64>,
    pub undo_stack: Vec<Undo>,
}

pub const CAPTURE_FLAG: u8 = 1 << 0;
pub const PROMOTION_FLAG: u8 = 1 << 1;
pub const EN_PASSANT_FLAG: u8 = 1 << 2;
pub const CASTLING_FLAG: u8 = 1 << 3;
pub const DOUBLE_PAWN_PUSH_FLAG: u8 = 1 << 4;

impl Piece {
    pub const PIECE_LIST: [Piece; 12] = [
        Piece::WP,
        Piece::WN,
        Piece::WB,
        Piece::WR,
        Piece::WQ,
        Piece::WK,
        Piece::BP,
        Piece::BN,
        Piece::BB,
        Piece::BR,
        Piece::BQ,
        Piece::BK,
    ];
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Color::White => write!(f, "White"),
            Color::Black => write!(f, "Black"),
        }
    }
}

impl Color {
    pub fn flip(&self) -> Self {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

impl Undo {
    pub fn new(board: &Board) -> Self {
        Undo {
            castling: board.castling,
            en_passant: board.en_passant,
            halfmove_clock: board.halfmove_clock,
            zobrist: board.zobrist,
        }
    }
}

impl Move {
    pub fn new(
        from: u8,
        to: u8,
        moved_piece: Piece,
        captured_piece: Piece,
        promotion_piece: Piece,
        flags: u8,
    ) -> Self {
        Move(
            from as u32 // 6 bits
             | ((to as u32) << 6) // 6 bits
             | ((moved_piece as u32) << 12) // 4 bits
             | ((captured_piece as u32)<< 16) // 4 bits
             | ((promotion_piece as u32)<< 20) // 4 bits
             | ((flags as u32) << 24), // 5 bits
        )
    }

    pub fn from(self) -> u8 {
        (self.0 & 0b111111) as u8
    }

    pub fn to(self) -> u8 {
        ((self.0 >> 6) & 0b111111) as u8
    }

    pub fn moved_piece(self) -> Piece {
        let index = ((self.0 >> 12) & 0b1111) as usize;
        Piece::PIECE_LIST[index]
    }

    pub fn captured_piece(self) -> Piece {
        let index = ((self.0 >> 16) & 0b1111) as usize;
        if index < 12 {
            Piece::PIECE_LIST[index]
        } else {
            Piece::None
        }
    }

    pub fn promotion(self) -> Piece {
        let index = ((self.0 >> 20) & 0b1111) as usize;
        if index < 12 {
            Piece::PIECE_LIST[index]
        } else {
            Piece::None
        }
    }

    pub fn flags(self) -> u8 {
        ((self.0 >> 24) & 0b11111) as u8
    }
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
    let mut castling = [0; 4];
    for i in 0..4 {
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

fn zobrist() -> &'static Zobrist {
    ZOBRIST.get_or_init(init_zobrist)
}

impl Board {
    fn compute_zobrist(&self) -> u64 {
        let z = zobrist();
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
        if self.en_passant < 64 {
            let file = self.en_passant % 8;
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

        let en_passant = 64;
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
    pub fn piece_at(&self, sq: u8) -> Piece {
        self.mailbox[sq as usize]
    }

    // Sets the bit to 0 at index sq of the bitmap at index p of self.pieces
    fn remove_piece(&mut self, p: Piece, sq: u8) {
        let mask = 1 << sq;
        self.pieces[p as usize] &= !mask;
    }

    // Sets the bit to 1 at index sq of the bitmap at index p of self.pieces
    fn add_piece(&mut self, p: Piece, sq: u8) {
        let mask = 1 << sq;
        self.pieces[p as usize] |= mask;
    }

    // Flips the bits at the from and to index in the bitmap at index p of self.pieces
    fn move_piece(&mut self, p: Piece, from: u8, to: u8) {
        let mask = 1 << from | 1 << to;
        self.pieces[p as usize] ^= mask;
    }

    pub fn make_move(&mut self, mv: Move) {
        let z = zobrist();
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
        if is_capture {
            let c_p = mv.captured_piece();
            let c_sq = if is_en_passant {
                if us == Color::White {
                    mv.to() - 8
                } else {
                    mv.to() + 8
                }
            } else {
                mv.to()
            };
            self.remove_piece(c_p, c_sq);
            self.mailbox[c_sq as usize] = Piece::None;
            let mask = 1u64 << c_sq;
            opponent_side_mask |= mask;
            hash ^= z.piece_square[c_p as usize][c_sq as usize];
        }

        // Save undo information
        let undo = Undo::new(self);
        self.undo_stack.push(undo);

        // Move the played piece (plus rook for castling)
        let moved_piece = mv.moved_piece();
        let moved_mask = (1u64 << mv.from()) | (1u64 << mv.to());
        moving_side_mask |= moved_mask;
        hash ^= z.piece_square[moved_piece as usize][mv.from() as usize];

        if is_promotion {
            let promotion_piece = mv.promotion();
            self.remove_piece(moved_piece, mv.from());
            self.mailbox[mv.from() as usize] = Piece::None;
            self.add_piece(promotion_piece, mv.to());
            self.mailbox[mv.to() as usize] = promotion_piece;
            hash ^= z.piece_square[promotion_piece as usize][mv.to() as usize];
        } else {
            self.move_piece(moved_piece, mv.from(), mv.to());
            self.mailbox[mv.from() as usize] = Piece::None;
            self.mailbox[mv.to() as usize] = moved_piece;
            hash ^= z.piece_square[moved_piece as usize][mv.to() as usize];

            if is_castling {
                let (rook, rook_from, rook_to) = match mv.to() {
                    62 => (Piece::BR, 63, 61), // black king side
                    58 => (Piece::BR, 56, 59), // black queen side
                    6 => (Piece::WR, 7, 5),    // white king side
                    2 => (Piece::WR, 0, 3),    // white queen side
                    _ => unreachable!(),
                };

                self.move_piece(rook, rook_from, rook_to);
                self.mailbox[rook_from as usize] = Piece::None;
                self.mailbox[rook_to as usize] = rook;

                let rook_mask = (1u64 << rook_from) | (1u64 << rook_to);
                moving_side_mask |= rook_mask;
                hash ^= z.piece_square[rook as usize][rook_from as usize];
                hash ^= z.piece_square[rook as usize][rook_to as usize];
            }
        }

        // Update occupancy
        self.occ ^= moving_side_mask ^ opponent_side_mask;
        self.color_occ[self.side_to_move as usize] ^= moving_side_mask;
        self.color_occ[self.side_to_move.flip() as usize] ^= opponent_side_mask;

        // Update en passant square
        if self.en_passant < 64 {
            let file = self.en_passant % 8;
            hash ^= z.en_passant[file as usize];
        }
        self.en_passant = if is_double_pawn_push {
            (mv.from() + mv.to()) / 2
        } else {
            64
        };
        if self.en_passant < 64 {
            let file = self.en_passant % 8;
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
                if mv.from() == 7 {
                    !WK_CASTLE
                } else if mv.from() == 0 {
                    !WQ_CASTLE
                } else {
                    !0
                }
            }
            Piece::BR => {
                if mv.from() == 63 {
                    !BK_CASTLE
                } else if mv.from() == 56 {
                    !BQ_CASTLE
                } else {
                    !0
                }
            }
            _ => !0,
        };

        if is_capture {
            self.castling &= match mv.captured_piece() {
                Piece::WR => {
                    if mv.to() == 7 {
                        !WK_CASTLE
                    } else if mv.to() == 0 {
                        !WQ_CASTLE
                    } else {
                        !0
                    }
                }
                Piece::BR => {
                    if mv.to() == 63 {
                        !BK_CASTLE
                    } else if mv.to() == 56 {
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
        moving_side_mask |= (1u64 << mv.to()) | (1u64 << mv.from());
        if is_promotion {
            let promotion_piece = mv.promotion();
            self.remove_piece(promotion_piece, mv.to());
            self.mailbox[mv.to() as usize] = Piece::None;

            match self.side_to_move {
                Color::White => {
                    self.add_piece(Piece::WP, mv.from());
                    self.mailbox[mv.from() as usize] = Piece::WP;
                }
                Color::Black => {
                    self.add_piece(Piece::BP, mv.from());
                    self.mailbox[mv.from() as usize] = Piece::BP;
                }
            }
        } else {
            let moved_piece = mv.moved_piece();
            self.move_piece(moved_piece, mv.to(), mv.from());
            self.mailbox[mv.to() as usize] = Piece::None;
            self.mailbox[mv.from() as usize] = moved_piece;

            if is_castling {
                let (rook, rook_from, rook_to) = match mv.to() {
                    62 => (Piece::BR, 61, 63), // black king side
                    58 => (Piece::BR, 59, 56), // black queen side
                    6 => (Piece::WR, 5, 7),    // white king side
                    2 => (Piece::WR, 3, 0),    // white queen side
                    _ => unreachable!(),
                };

                self.move_piece(rook, rook_from, rook_to);
                self.mailbox[rook_from as usize] = Piece::None;
                self.mailbox[rook_to as usize] = rook;

                let rook_mask = (1u64 << rook_from) | (1u64 << rook_to);
                moving_side_mask |= rook_mask;
            }
        }

        // In case of capture (and maybe en passant), restore the taken piece
        if is_capture {
            let c_p = mv.captured_piece();
            let c_sq = if is_en_passant {
                match self.side_to_move {
                    Color::White => mv.to() - 8,
                    Color::Black => mv.to() + 8,
                }
            } else {
                mv.to()
            };
            let sq_mask = 1u64 << c_sq;
            opponent_side_mask |= sq_mask;
            self.add_piece(c_p, c_sq);
            self.mailbox[c_sq as usize] = c_p;
        }

        // Recompute occupancy
        self.occ ^= moving_side_mask ^ opponent_side_mask;
        self.color_occ[self.side_to_move as usize] ^= moving_side_mask;
        self.color_occ[self.side_to_move.flip() as usize] ^= opponent_side_mask;
    }

    const fn gen_white_pawn_attacks() -> [u64; 64] {
        let mut table = [0; 64];
        let mut sq = 0;
        while sq < 64 {
            let sq_mask = 1u64 << sq;
            let mut mask = 0u64;
            if sq < 56 {
                mask |= (sq_mask << 7) & !FILE_H;
                mask |= (sq_mask << 9) & !FILE_A;
            }
            table[sq] = mask;
            sq += 1;
        }
        table
    }

    const WHITE_PAWN_ATTACKS: [u64; 64] = Self::gen_white_pawn_attacks();

    fn get_white_pawn_moves(
        &self,
        moves: &mut Vec<Move>,
        block_mask: u64,
        capture_mask: u64,
        pinned: u64,
        pin_rays: &[u64; 64],
    ) {
        let pawns = self.pieces[Piece::WP as usize];
        let empty = !self.occ;

        // Single pushes
        let mut single_push = (pawns << 8) & empty;
        while single_push != 0 {
            let to = single_push.trailing_zeros() as u8;
            single_push &= single_push - 1;
            let from = to - 8;
            let is_promotion = to > 55;
            let is_pinned = (pinned >> from) & 1 != 0;
            let is_legal = if is_pinned {
                (1u64 << to) & pin_rays[from as usize] & block_mask != 0
            } else {
                (block_mask >> to) & 1 != 0
            };
            if is_legal {
                if is_promotion {
                    moves.push(Move::new(
                        from,
                        to,
                        Piece::WP,
                        Piece::None,
                        Piece::WN,
                        PROMOTION_FLAG,
                    ));
                    moves.push(Move::new(
                        from,
                        to,
                        Piece::WP,
                        Piece::None,
                        Piece::WB,
                        PROMOTION_FLAG,
                    ));
                    moves.push(Move::new(
                        from,
                        to,
                        Piece::WP,
                        Piece::None,
                        Piece::WR,
                        PROMOTION_FLAG,
                    ));
                    moves.push(Move::new(
                        from,
                        to,
                        Piece::WP,
                        Piece::None,
                        Piece::WQ,
                        PROMOTION_FLAG,
                    ));
                } else {
                    moves.push(Move::new(from, to, Piece::WP, Piece::None, Piece::None, 0));
                }
            }
        }

        // Double pushes
        let first_push = ((pawns & RANK_2) << 8) & empty;
        let mut double_push = (first_push << 8) & empty;
        while double_push != 0 {
            let to = double_push.trailing_zeros() as u8;
            double_push &= double_push - 1;
            let from = to - 16;
            let is_pinned = (pinned >> from) & 1 != 0;
            let is_legal = if is_pinned {
                (1u64 << to) & pin_rays[from as usize] & block_mask != 0
            } else {
                (block_mask >> to) & 1 != 0
            };
            if is_legal {
                moves.push(Move::new(
                    from,
                    to,
                    Piece::WP,
                    Piece::None,
                    Piece::None,
                    DOUBLE_PAWN_PUSH_FLAG,
                ))
            };
        }

        // Captures
        let mut captures = pawns;
        while captures != 0 {
            let from = captures.trailing_zeros() as u8;
            captures &= captures - 1;

            let mut attacks = Self::WHITE_PAWN_ATTACKS[from as usize] & self.color_occ[1];

            while attacks != 0 {
                let to = attacks.trailing_zeros() as u8;
                attacks &= attacks - 1;

                let is_promotion = to > 55;
                let captured_piece = self.piece_at(to);
                let is_pinned = (pinned >> from) & 1 != 0;
                let is_legal = if is_pinned {
                    (1u64 << to) & pin_rays[from as usize] & capture_mask != 0
                } else {
                    (capture_mask >> to) & 1 != 0
                };
                if is_legal {
                    if is_promotion {
                        moves.push(Move::new(
                            from,
                            to,
                            Piece::WP,
                            captured_piece,
                            Piece::WN,
                            CAPTURE_FLAG | PROMOTION_FLAG,
                        ));
                        moves.push(Move::new(
                            from,
                            to,
                            Piece::WP,
                            captured_piece,
                            Piece::WB,
                            CAPTURE_FLAG | PROMOTION_FLAG,
                        ));
                        moves.push(Move::new(
                            from,
                            to,
                            Piece::WP,
                            captured_piece,
                            Piece::WR,
                            CAPTURE_FLAG | PROMOTION_FLAG,
                        ));
                        moves.push(Move::new(
                            from,
                            to,
                            Piece::WP,
                            captured_piece,
                            Piece::WQ,
                            CAPTURE_FLAG | PROMOTION_FLAG,
                        ));
                    } else {
                        moves.push(Move::new(
                            from,
                            to,
                            Piece::WP,
                            captured_piece,
                            Piece::None,
                            CAPTURE_FLAG,
                        ));
                    }
                }
            }
        }

        // En passant
        if self.en_passant < 64 {
            let en_passant_mask = 1 << self.en_passant;
            let mut left_en_passant = ((pawns << 7) & !FILE_H) & en_passant_mask;
            while left_en_passant != 0 {
                let to = left_en_passant.trailing_zeros() as u8;
                left_en_passant &= left_en_passant - 1;
                let from = to - 7;
                let captured_sq = to - 8;
                let changed_occ = self.occ ^ (1u64 << from) ^ (1u64 << captured_sq) ^ (1u64 << to);
                let mut changed_pieces = self.pieces;
                changed_pieces[Piece::BP as usize] ^= 1u64 << captured_sq;
                changed_pieces[Piece::WP as usize] ^= (1u64 << to) | (1u64 << from);
                let checkers = self.compute_checkers(
                    self.pieces[Piece::WK as usize].trailing_zeros() as u8,
                    changed_occ,
                    Color::Black,
                    &changed_pieces,
                );
                if checkers == 0 {
                    moves.push(Move::new(
                        from,
                        to,
                        Piece::WP,
                        Piece::BP,
                        Piece::None,
                        CAPTURE_FLAG | EN_PASSANT_FLAG,
                    ));
                }
            }
            let mut right_en_passant = ((pawns << 9) & !FILE_A) & en_passant_mask;
            while right_en_passant != 0 {
                let to = right_en_passant.trailing_zeros() as u8;
                right_en_passant &= right_en_passant - 1;
                let from = to - 9;
                let captured_sq = to - 8;
                let changed_occ = self.occ ^ (1u64 << from) ^ (1u64 << captured_sq) ^ (1u64 << to);
                let mut changed_pieces = self.pieces;
                changed_pieces[Piece::BP as usize] ^= 1u64 << captured_sq;
                changed_pieces[Piece::WP as usize] ^= (1u64 << to) | (1u64 << from);
                let checkers = self.compute_checkers(
                    self.pieces[Piece::WK as usize].trailing_zeros() as u8,
                    changed_occ,
                    Color::Black,
                    &changed_pieces,
                );
                if checkers == 0 {
                    moves.push(Move::new(
                        from,
                        to,
                        Piece::WP,
                        Piece::BP,
                        Piece::None,
                        CAPTURE_FLAG | EN_PASSANT_FLAG,
                    ));
                }
            }
        }
    }

    const fn gen_black_pawn_attacks() -> [u64; 64] {
        let mut table = [0; 64];
        let mut sq = 0;
        while sq < 64 {
            let sq_mask = 1u64 << sq;
            let mut mask = 0u64;
            if sq > 7 {
                mask |= (sq_mask >> 7) & !FILE_A;
                mask |= (sq_mask >> 9) & !FILE_H;
            }
            table[sq] = mask;
            sq += 1;
        }
        table
    }

    const BLACK_PAWN_ATTACKS: [u64; 64] = Self::gen_black_pawn_attacks();

    fn get_black_pawn_moves(
        &self,
        moves: &mut Vec<Move>,
        block_mask: u64,
        capture_mask: u64,
        pinned: u64,
        pin_rays: &[u64; 64],
    ) {
        let pawns = self.pieces[Piece::BP as usize];
        let empty = !self.occ;

        // Single pushes
        let mut single_push = (pawns >> 8) & empty;
        while single_push != 0 {
            let to = single_push.trailing_zeros() as u8;
            single_push &= single_push - 1;
            let from = to + 8;
            let is_promotion = to < 8;
            let is_pinned = (pinned >> from) & 1 != 0;
            let is_legal = if is_pinned {
                (1u64 << to) & pin_rays[from as usize] & block_mask != 0
            } else {
                (block_mask >> to) & 1 != 0
            };
            if is_legal {
                if is_promotion {
                    moves.push(Move::new(
                        from,
                        to,
                        Piece::BP,
                        Piece::None,
                        Piece::BN,
                        PROMOTION_FLAG,
                    ));
                    moves.push(Move::new(
                        from,
                        to,
                        Piece::BP,
                        Piece::None,
                        Piece::BB,
                        PROMOTION_FLAG,
                    ));
                    moves.push(Move::new(
                        from,
                        to,
                        Piece::BP,
                        Piece::None,
                        Piece::BR,
                        PROMOTION_FLAG,
                    ));
                    moves.push(Move::new(
                        from,
                        to,
                        Piece::BP,
                        Piece::None,
                        Piece::BQ,
                        PROMOTION_FLAG,
                    ));
                } else {
                    moves.push(Move::new(from, to, Piece::BP, Piece::None, Piece::None, 0));
                }
            }
        }

        // Double pushes
        let first_push = ((pawns & RANK_7) >> 8) & empty;
        let mut double_push = (first_push >> 8) & empty;
        while double_push != 0 {
            let to = double_push.trailing_zeros() as u8;
            double_push &= double_push - 1;
            let from = to + 16;
            let is_pinned = (pinned >> from) & 1 != 0;
            let is_legal = if is_pinned {
                (1u64 << to) & pin_rays[from as usize] & block_mask != 0
            } else {
                (block_mask >> to) & 1 != 0
            };
            if is_legal {
                moves.push(Move::new(
                    from,
                    to,
                    Piece::BP,
                    Piece::None,
                    Piece::None,
                    DOUBLE_PAWN_PUSH_FLAG,
                ));
            }
        }

        // Captures
        let mut captures = pawns;
        while captures != 0 {
            let from = captures.trailing_zeros() as u8;
            captures &= captures - 1;

            let mut attacks = Self::BLACK_PAWN_ATTACKS[from as usize] & self.color_occ[0];

            while attacks != 0 {
                let to = attacks.trailing_zeros() as u8;
                attacks &= attacks - 1;

                let is_promotion = to < 8;
                let captured_piece = self.piece_at(to);
                let is_pinned = (pinned >> from) & 1 != 0;
                let is_legal = if is_pinned {
                    (1u64 << to) & pin_rays[from as usize] & capture_mask != 0
                } else {
                    (capture_mask >> to) & 1 != 0
                };
                if is_legal {
                    if is_promotion {
                        moves.push(Move::new(
                            from,
                            to,
                            Piece::BP,
                            captured_piece,
                            Piece::BN,
                            CAPTURE_FLAG | PROMOTION_FLAG,
                        ));
                        moves.push(Move::new(
                            from,
                            to,
                            Piece::BP,
                            captured_piece,
                            Piece::BB,
                            CAPTURE_FLAG | PROMOTION_FLAG,
                        ));
                        moves.push(Move::new(
                            from,
                            to,
                            Piece::BP,
                            captured_piece,
                            Piece::BR,
                            CAPTURE_FLAG | PROMOTION_FLAG,
                        ));
                        moves.push(Move::new(
                            from,
                            to,
                            Piece::BP,
                            captured_piece,
                            Piece::BQ,
                            CAPTURE_FLAG | PROMOTION_FLAG,
                        ));
                    } else {
                        moves.push(Move::new(
                            from,
                            to,
                            Piece::BP,
                            captured_piece,
                            Piece::None,
                            CAPTURE_FLAG,
                        ));
                    }
                }
            }
        }

        // En passant
        if self.en_passant < 64 {
            let en_passant_mask = 1 << self.en_passant;
            let mut left_en_passant = ((pawns >> 7) & !FILE_A) & en_passant_mask;
            while left_en_passant != 0 {
                let to = left_en_passant.trailing_zeros() as u8;
                left_en_passant &= left_en_passant - 1;
                let from = to + 7;
                let captured_sq = to + 8;
                let changed_occ = self.occ ^ (1u64 << from) ^ (1u64 << captured_sq) ^ (1u64 << to);
                let mut changed_pieces = self.pieces;
                changed_pieces[Piece::WP as usize] ^= 1u64 << captured_sq;
                changed_pieces[Piece::BP as usize] ^= (1u64 << to) | (1u64 << from);
                let checkers = self.compute_checkers(
                    self.pieces[Piece::BK as usize].trailing_zeros() as u8,
                    changed_occ,
                    Color::White,
                    &changed_pieces,
                );
                if checkers == 0 {
                    moves.push(Move::new(
                        from,
                        to,
                        Piece::BP,
                        Piece::WP,
                        Piece::None,
                        CAPTURE_FLAG | EN_PASSANT_FLAG,
                    ));
                }
            }
            let mut right_en_passant = ((pawns >> 9) & !FILE_H) & en_passant_mask;
            while right_en_passant != 0 {
                let to = right_en_passant.trailing_zeros() as u8;
                right_en_passant &= right_en_passant - 1;
                let from = to + 9;
                let captured_sq = to + 8;
                let changed_occ = self.occ ^ (1u64 << from) ^ (1u64 << captured_sq) ^ (1u64 << to);
                let mut changed_pieces = self.pieces;
                changed_pieces[Piece::WP as usize] ^= 1u64 << captured_sq;
                changed_pieces[Piece::BP as usize] ^= (1u64 << to) | (1u64 << from);
                let checkers = self.compute_checkers(
                    self.pieces[Piece::BK as usize].trailing_zeros() as u8,
                    changed_occ,
                    Color::White,
                    &changed_pieces,
                );
                if checkers == 0 {
                    moves.push(Move::new(
                        from,
                        to,
                        Piece::BP,
                        Piece::WP,
                        Piece::None,
                        CAPTURE_FLAG | EN_PASSANT_FLAG,
                    ));
                }
            }
        }
    }

    const fn gen_knight_attacks() -> [u64; 64] {
        let mut table = [0u64; 64];
        let mut sq = 0;

        while sq < 64 {
            let mask = 1u64 << sq;

            let attacks = ((mask << 17) & !FILE_A)
                | ((mask << 15) & !FILE_H)
                | ((mask << 10) & !FILE_A & !FILE_B)
                | ((mask << 6) & !FILE_H & !FILE_G)
                | ((mask >> 17) & !FILE_H)
                | ((mask >> 15) & !FILE_A)
                | ((mask >> 10) & !FILE_H & !FILE_G)
                | ((mask >> 6) & !FILE_A & !FILE_B);

            table[sq as usize] = attacks;
            sq += 1;
        }

        table
    }

    const KNIGHT_ATTACKS: [u64; 64] = Self::gen_knight_attacks();

    fn get_knight_moves(
        &self,
        moves: &mut Vec<Move>,
        knight_piece: Piece,
        block_mask: u64,
        capture_mask: u64,
        pinned: u64,
    ) {
        let us = self.side_to_move;
        let mut knights = self.pieces[knight_piece as usize] & !pinned;
        while knights != 0 {
            let from = knights.trailing_zeros() as u8;
            knights &= knights - 1;

            let mut attacks = Self::KNIGHT_ATTACKS[from as usize]
                & !self.color_occ[us as usize]
                & (block_mask | capture_mask);

            while attacks != 0 {
                let to = attacks.trailing_zeros() as u8;
                attacks &= attacks - 1;

                let captured_piece = self.piece_at(to);
                let flag = if captured_piece == Piece::None {
                    0
                } else {
                    CAPTURE_FLAG
                };

                moves.push(Move::new(
                    from,
                    to,
                    knight_piece,
                    captured_piece,
                    Piece::None,
                    flag,
                ));
            }
        }
    }

    const fn gen_rook_masks() -> [u64; 64] {
        let mut masks = [0; 64];
        let mut sq: usize = 0;
        while sq < 64 {
            let rank = sq / 8;
            let file = sq % 8;
            let mut mask = 0;
            // North
            let mut r = rank + 1;
            while r < 7 {
                mask |= 1u64 << (r * 8 + file);
                r += 1;
            }
            // South
            let mut r = rank as i32 - 1;
            while r > 0 {
                mask |= 1u64 << (r * 8 + file as i32);
                r -= 1;
            }
            // East
            let mut f = file + 1;
            while f < 7 {
                mask |= 1u64 << (rank * 8 + f);
                f += 1;
            }
            // West
            let mut f = file as i32 - 1;
            while f > 0 {
                mask |= 1u64 << (rank as i32 * 8 + f);
                f -= 1;
            }
            masks[sq] = mask;
            sq += 1;
        }
        masks
    }

    const fn gen_bishop_masks() -> [u64; 64] {
        let mut masks = [0; 64];
        let mut sq: usize = 0;
        while sq < 64 {
            let rank = sq / 8;
            let file = sq % 8;
            let mut mask = 0;
            // North-East
            let mut r = rank + 1;
            let mut f = file + 1;
            while r < 7 && f < 7 {
                mask |= 1u64 << (r * 8 + f);
                r += 1;
                f += 1;
            }
            // North-West
            let mut r = rank + 1;
            let mut f = file as i32 - 1;
            while r < 7 && f > 0 {
                mask |= 1u64 << (r as i32 * 8 + f);
                r += 1;
                f -= 1;
            }
            // South-East
            let mut r = rank as i32 - 1;
            let mut f = file + 1;
            while r > 0 && f < 7 {
                mask |= 1u64 << (r * 8 + f as i32);
                r -= 1;
                f += 1;
            }
            // South-West
            let mut r = rank as i32 - 1;
            let mut f = file as i32 - 1;
            while r > 0 && f > 0 {
                mask |= 1u64 << (r as u32 * 8 + f as u32);
                r -= 1;
                f -= 1;
            }
            masks[sq] = mask;
            sq += 1;
        }
        masks
    }

    const ROOK_MASKS: [u64; 64] = Self::gen_rook_masks();
    const BISHOP_MASKS: [u64; 64] = Self::gen_bishop_masks();

    // Magics and shift from https://github.com/SebLague/Chess-Coding-Adventure/
    const ROOK_MAGIC_NUMBERS: [u64; 64] = [
        468374916371625120,
        18428729537625841661,
        2531023729696186408,
        6093370314119450896,
        13830552789156493815,
        16134110446239088507,
        12677615322350354425,
        5404321144167858432,
        2111097758984580,
        18428720740584907710,
        17293734603602787839,
        4938760079889530922,
        7699325603589095390,
        9078693890218258431,
        578149610753690728,
        9496543503900033792,
        1155209038552629657,
        9224076274589515780,
        1835781998207181184,
        509120063316431138,
        16634043024132535807,
        18446673631917146111,
        9623686630121410312,
        4648737361302392899,
        738591182849868645,
        1732936432546219272,
        2400543327507449856,
        5188164365601475096,
        10414575345181196316,
        1162492212166789136,
        9396848738060210946,
        622413200109881612,
        7998357718131801918,
        7719627227008073923,
        16181433497662382080,
        18441958655457754079,
        1267153596645440,
        18446726464209379263,
        1214021438038606600,
        4650128814733526084,
        9656144899867951104,
        18444421868610287615,
        3695311799139303489,
        10597006226145476632,
        18436046904206950398,
        18446726472933277663,
        3458977943764860944,
        39125045590687766,
        9227453435446560384,
        6476955465732358656,
        1270314852531077632,
        2882448553461416064,
        11547238928203796481,
        1856618300822323264,
        2573991788166144,
        4936544992551831040,
        13690941749405253631,
        15852669863439351807,
        18302628748190527413,
        12682135449552027479,
        13830554446930287982,
        18302628782487371519,
        7924083509981736956,
        4734295326018586370,
    ];
    const BISHOP_MAGIC_NUMBERS: [u64; 64] = [
        16509839532542417919,
        14391803910955204223,
        1848771770702627364,
        347925068195328958,
        5189277761285652493,
        3750937732777063343,
        18429848470517967340,
        17870072066711748607,
        16715520087474960373,
        2459353627279607168,
        7061705824611107232,
        8089129053103260512,
        7414579821471224013,
        9520647030890121554,
        17142940634164625405,
        9187037984654475102,
        4933695867036173873,
        3035992416931960321,
        15052160563071165696,
        5876081268917084809,
        1153484746652717320,
        6365855841584713735,
        2463646859659644933,
        1453259901463176960,
        9808859429721908488,
        2829141021535244552,
        576619101540319252,
        5804014844877275314,
        4774660099383771136,
        328785038479458864,
        2360590652863023124,
        569550314443282,
        17563974527758635567,
        11698101887533589556,
        5764964460729992192,
        6953579832080335136,
        1318441160687747328,
        8090717009753444376,
        16751172641200572929,
        5558033503209157252,
        17100156536247493656,
        7899286223048400564,
        4845135427956654145,
        2368485888099072,
        2399033289953272320,
        6976678428284034058,
        3134241565013966284,
        8661609558376259840,
        17275805361393991679,
        15391050065516657151,
        11529206229534274423,
        9876416274250600448,
        16432792402597134585,
        11975705497012863580,
        11457135419348969979,
        9763749252098620046,
        16960553411078512574,
        15563877356819111679,
        14994736884583272463,
        9441297368950544394,
        14537646123432199168,
        9888547162215157388,
        18140215579194907366,
        18374682062228545019,
    ];

    const ROOK_SHIFTS: [u64; 64] = [
        52, 52, 52, 52, 52, 52, 52, 52, 53, 53, 53, 54, 53, 53, 54, 53, 53, 54, 54, 54, 53, 53, 54,
        53, 53, 54, 53, 53, 54, 54, 54, 53, 52, 54, 53, 53, 53, 53, 54, 53, 52, 53, 54, 54, 53, 53,
        54, 53, 53, 54, 54, 54, 53, 53, 54, 53, 52, 53, 53, 53, 53, 53, 53, 52,
    ];
    const BISHOP_SHIFTS: [u64; 64] = [
        58, 60, 59, 59, 59, 59, 60, 58, 60, 59, 59, 59, 59, 59, 59, 60, 59, 59, 57, 57, 57, 57, 59,
        59, 59, 59, 57, 55, 55, 57, 59, 59, 59, 59, 57, 55, 55, 57, 59, 59, 59, 59, 57, 57, 57, 57,
        59, 59, 60, 60, 59, 59, 59, 59, 60, 60, 58, 60, 59, 59, 59, 59, 59, 58,
    ];

    const fn rook_attacks_for_blockers(sq: usize, blockers: u64) -> u64 {
        let mut attacks = 0;
        let rank = sq / 8;
        let file = sq % 8;
        // North
        let mut r = rank + 1;
        while r < 8 {
            let target = r * 8 + file;
            attacks |= 1 << target;
            if blockers & (1 << target) != 0 {
                break;
            }
            r += 1;
        }
        // South
        let mut r = rank as i32 - 1;
        while r >= 0 {
            let target = (r * 8 + file as i32) as usize;
            attacks |= 1 << target;
            if blockers & (1 << target) != 0 {
                break;
            }
            r -= 1;
        }
        // East
        let mut f = file + 1;
        while f < 8 {
            let target = rank * 8 + f;
            attacks |= 1 << target;
            if blockers & (1 << target) != 0 {
                break;
            }
            f += 1;
        }
        // West
        let mut f = file as i32 - 1;
        while f >= 0 {
            let target = rank * 8 + f as usize;
            attacks |= 1 << target;
            if blockers & (1 << target) != 0 {
                break;
            }
            f -= 1;
        }
        attacks
    }

    const fn bishop_attacks_for_blockers(sq: usize, blockers: u64) -> u64 {
        let mut attacks = 0;
        let rank = sq / 8;
        let file = sq % 8;
        // North-East
        let mut r = rank + 1;
        let mut f = file + 1;
        while r < 8 && f < 8 {
            let target = r * 8 + f;
            attacks |= 1 << target;
            if blockers & (1 << target) != 0 {
                break;
            }
            r += 1;
            f += 1;
        }
        // North-West
        let mut r = rank + 1;
        let mut f = file as i32 - 1;
        while r < 8 && f >= 0 {
            let target = r * 8 + f as usize;
            attacks |= 1 << target;
            if blockers & (1 << target) != 0 {
                break;
            }
            r += 1;
            f -= 1;
        }
        // South-East
        let mut r = rank as i32 - 1;
        let mut f = file + 1;
        while r >= 0 && f < 8 {
            let target = (r * 8 + f as i32) as usize;
            attacks |= 1 << target;
            if blockers & (1 << target) != 0 {
                break;
            }
            r -= 1;
            f += 1;
        }
        // South-West
        let mut r = rank as i32 - 1;
        let mut f = file as i32 - 1;
        while r >= 0 && f >= 0 {
            let target = (r * 8 + f) as usize;
            attacks |= 1 << target;
            if blockers & (1 << target) != 0 {
                break;
            }
            r -= 1;
            f -= 1;
        }
        attacks
    }

    const fn gen_rook_attacks() -> [[u64; 4096]; 64] {
        let mut table = [[0; 4096]; 64];
        let mut sq = 0;
        while sq < 64 {
            let mask = Self::ROOK_MASKS[sq];
            let magic = Self::ROOK_MAGIC_NUMBERS[sq];
            let shift = Self::ROOK_SHIFTS[sq];
            let mut blockers = 0;
            loop {
                let attacks = Self::rook_attacks_for_blockers(sq, blockers);
                let index = ((blockers.wrapping_mul(magic)) >> shift) as usize;
                table[sq][index] = attacks;
                blockers = (blockers.wrapping_sub(mask)) & mask;
                if blockers == 0 {
                    break;
                }
            }
            sq += 1;
        }
        table
    }

    const fn gen_bishop_attacks() -> [[u64; 512]; 64] {
        let mut table = [[0; 512]; 64];
        let mut sq = 0;
        while sq < 64 {
            let mask = Self::BISHOP_MASKS[sq];
            let magic = Self::BISHOP_MAGIC_NUMBERS[sq];
            let shift = Self::BISHOP_SHIFTS[sq];
            let mut blockers = 0;
            loop {
                let attacks = Self::bishop_attacks_for_blockers(sq, blockers);
                let index = ((blockers.wrapping_mul(magic)) >> shift) as usize;
                table[sq][index] = attacks;
                blockers = (blockers.wrapping_sub(mask)) & mask;
                if blockers == 0 {
                    break;
                }
            }
            sq += 1;
        }
        table
    }

    const ROOK_ATTACKS: [[u64; 4096]; 64] = Self::gen_rook_attacks();
    const BISHOP_ATTACKS: [[u64; 512]; 64] = Self::gen_bishop_attacks();

    fn get_rook_moves(
        &self,
        moves: &mut Vec<Move>,
        rook_piece: Piece,
        block_mask: u64,
        capture_mask: u64,
        pinned: u64,
        pin_rays: &[u64; 64],
    ) {
        let us = self.side_to_move;
        let mut rooks = self.pieces[rook_piece as usize];
        while rooks != 0 {
            let from = rooks.trailing_zeros() as u8;
            rooks &= rooks - 1;

            let is_pinned = (pinned >> from) & 1 != 0;

            let mask = Self::ROOK_MASKS[from as usize];
            let blockers = self.occ & mask;
            let idx = (blockers.wrapping_mul(Self::ROOK_MAGIC_NUMBERS[from as usize])
                >> Self::ROOK_SHIFTS[from as usize]) as usize;
            let legal_mask = if is_pinned {
                !self.color_occ[us as usize] & (block_mask | capture_mask) & pin_rays[from as usize]
            } else {
                !self.color_occ[us as usize] & (block_mask | capture_mask)
            };
            let attacks = Self::ROOK_ATTACKS[from as usize][idx] & legal_mask;

            let mut remaining = attacks;
            while remaining != 0 {
                let to = remaining.trailing_zeros() as u8;
                remaining &= remaining - 1;

                let captured_piece = self.piece_at(to);
                let flag = if captured_piece == Piece::None {
                    0
                } else {
                    CAPTURE_FLAG
                };
                moves.push(Move::new(
                    from,
                    to,
                    rook_piece,
                    captured_piece,
                    Piece::None,
                    flag,
                ));
            }
        }
    }

    fn get_bishop_moves(
        &self,
        moves: &mut Vec<Move>,
        bishop_piece: Piece,
        block_mask: u64,
        capture_mask: u64,
        pinned: u64,
        pin_rays: &[u64; 64],
    ) {
        let us = self.side_to_move;
        let mut bishops = self.pieces[bishop_piece as usize];
        while bishops != 0 {
            let from = bishops.trailing_zeros() as u8;
            bishops &= bishops - 1;

            let is_pinned = (pinned >> from) & 1 != 0;

            let mask = Self::BISHOP_MASKS[from as usize];
            let blockers = self.occ & mask;
            let idx = (blockers.wrapping_mul(Self::BISHOP_MAGIC_NUMBERS[from as usize])
                >> Self::BISHOP_SHIFTS[from as usize]) as usize;
            let legal_mask = if is_pinned {
                !self.color_occ[us as usize] & (block_mask | capture_mask) & pin_rays[from as usize]
            } else {
                !self.color_occ[us as usize] & (block_mask | capture_mask)
            };
            let attacks = Self::BISHOP_ATTACKS[from as usize][idx] & legal_mask;

            let mut remaining = attacks;
            while remaining != 0 {
                let to = remaining.trailing_zeros() as u8;
                remaining &= remaining - 1;

                let captured_piece = self.piece_at(to);
                let flag = if captured_piece == Piece::None {
                    0
                } else {
                    CAPTURE_FLAG
                };
                moves.push(Move::new(
                    from,
                    to,
                    bishop_piece,
                    captured_piece,
                    Piece::None,
                    flag,
                ));
            }
        }
    }

    fn get_queen_moves(
        &self,
        moves: &mut Vec<Move>,
        queen_piece: Piece,
        block_mask: u64,
        capture_mask: u64,
        pinned: u64,
        pin_rays: &[u64; 64],
    ) {
        let us = self.side_to_move;
        let mut queens = self.pieces[queen_piece as usize];
        while queens != 0 {
            let from = queens.trailing_zeros() as u8;
            queens &= queens - 1;

            let is_pinned = (pinned >> from) & 1 != 0;

            let orth_mask = Self::ROOK_MASKS[from as usize];
            let orth_blockers = self.occ & orth_mask;
            let orth_idx = (orth_blockers.wrapping_mul(Self::ROOK_MAGIC_NUMBERS[from as usize])
                >> Self::ROOK_SHIFTS[from as usize]) as usize;
            let orth_attacks = Self::ROOK_ATTACKS[from as usize][orth_idx];

            let diag_mask = Self::BISHOP_MASKS[from as usize];
            let diag_blockers = self.occ & diag_mask;
            let diag_idx = (diag_blockers.wrapping_mul(Self::BISHOP_MAGIC_NUMBERS[from as usize])
                >> Self::BISHOP_SHIFTS[from as usize]) as usize;
            let diag_attacks = Self::BISHOP_ATTACKS[from as usize][diag_idx];

            let legal_mask = if is_pinned {
                !self.color_occ[us as usize] & (block_mask | capture_mask) & pin_rays[from as usize]
            } else {
                !self.color_occ[us as usize] & (block_mask | capture_mask)
            };

            let attacks = (orth_attacks | diag_attacks) & legal_mask;

            let mut remaining = attacks;
            while remaining != 0 {
                let to = remaining.trailing_zeros() as u8;
                remaining &= remaining - 1;

                let captured_piece = self.piece_at(to);
                let flag = if captured_piece == Piece::None {
                    0
                } else {
                    CAPTURE_FLAG
                };
                moves.push(Move::new(
                    from,
                    to,
                    queen_piece,
                    captured_piece,
                    Piece::None,
                    flag,
                ));
            }
        }
    }

    fn compute_danger(&self, king_sq: u8) -> u64 {
        let mut danger = 0;
        let (
            mut pawns,
            pawn_attack_table,
            mut knights,
            mut orth_pieces,
            mut diag_pieces,
            enemy_king,
        ) = match self.side_to_move {
            Color::White => (
                self.pieces[Piece::BP as usize],
                Self::BLACK_PAWN_ATTACKS,
                self.pieces[Piece::BN as usize],
                self.pieces[Piece::BR as usize] | self.pieces[Piece::BQ as usize],
                self.pieces[Piece::BB as usize] | self.pieces[Piece::BQ as usize],
                self.pieces[Piece::BK as usize],
            ),
            Color::Black => (
                self.pieces[Piece::WP as usize],
                Self::WHITE_PAWN_ATTACKS,
                self.pieces[Piece::WN as usize],
                self.pieces[Piece::WR as usize] | self.pieces[Piece::WQ as usize],
                self.pieces[Piece::WB as usize] | self.pieces[Piece::WQ as usize],
                self.pieces[Piece::WK as usize],
            ),
        };
        let kingless_occ = self.occ & !(1u64 << king_sq);
        while pawns != 0 {
            let sq = pawns.trailing_zeros() as u8;
            pawns &= pawns - 1;
            danger |= pawn_attack_table[sq as usize];
        }
        while knights != 0 {
            let sq = knights.trailing_zeros() as u8;
            knights &= knights - 1;
            danger |= Self::KNIGHT_ATTACKS[sq as usize];
        }
        while diag_pieces != 0 {
            let sq = diag_pieces.trailing_zeros();
            diag_pieces &= diag_pieces - 1;
            let mask = Self::BISHOP_MASKS[sq as usize];
            let blockers = kingless_occ & mask;
            let idx = (blockers.wrapping_mul(Self::BISHOP_MAGIC_NUMBERS[sq as usize])
                >> Self::BISHOP_SHIFTS[sq as usize]) as usize;
            danger |= Self::BISHOP_ATTACKS[sq as usize][idx];
        }
        while orth_pieces != 0 {
            let sq = orth_pieces.trailing_zeros();
            orth_pieces &= orth_pieces - 1;
            let mask = Self::ROOK_MASKS[sq as usize];
            let blockers = kingless_occ & mask;
            let idx = (blockers.wrapping_mul(Self::ROOK_MAGIC_NUMBERS[sq as usize])
                >> Self::ROOK_SHIFTS[sq as usize]) as usize;
            danger |= Self::ROOK_ATTACKS[sq as usize][idx];
        }
        danger |= Self::KING_ATTACKS[enemy_king.trailing_zeros() as usize];

        danger
    }

    const fn gen_king_attacks() -> [u64; 64] {
        let mut table = [0u64; 64];

        let mut sq = 0;
        while sq < 64 {
            let rank = sq / 8;
            let file = sq % 8;

            let mut attacks = 0u64;

            let mut dr = -1;
            while dr <= 1 {
                let mut df = -1;
                while df <= 1 {
                    if dr == 0 && df == 0 {
                        df += 1;
                        continue;
                    }

                    let r = rank as i16 + dr;
                    let f = file as i16 + df;

                    if r >= 0 && r < 8 && f >= 0 && f < 8 {
                        let target = (r * 8 + f) as u64;
                        attacks |= 1u64 << target;
                    }

                    df += 1;
                }
                dr += 1;
            }
            table[sq] = attacks;
            sq += 1;
        }
        table
    }

    const KING_ATTACKS: [u64; 64] = Self::gen_king_attacks();

    fn get_king_moves(&self, moves: &mut Vec<Move>, king_sq: u8, danger_mask: u64) {
        let us = self.side_to_move;
        let mut attacks =
            Self::KING_ATTACKS[king_sq as usize] & !self.color_occ[us as usize] & !danger_mask;
        while attacks != 0 {
            let to = attacks.trailing_zeros() as u8;
            attacks &= attacks - 1;

            let captured_piece = self.piece_at(to);
            let flag = if captured_piece == Piece::None {
                0
            } else {
                CAPTURE_FLAG
            };

            moves.push(Move::new(
                king_sq,
                to,
                if us == Color::White {
                    Piece::WK
                } else {
                    Piece::BK
                },
                captured_piece,
                Piece::None,
                flag,
            ));
        }
    }

    fn compute_checkers(
        &self,
        king_sq: u8,
        occ: u64,
        attacker_color: Color,
        pieces: &[u64; 12],
    ) -> u64 {
        let mut checkers = 0;
        let (pawns, pawn_attack_table, knights, orth_pieces, diag_pieces) = match attacker_color {
            Color::Black => (
                pieces[Piece::BP as usize],
                Self::WHITE_PAWN_ATTACKS,
                pieces[Piece::BN as usize],
                pieces[Piece::BR as usize] | pieces[Piece::BQ as usize],
                pieces[Piece::BB as usize] | pieces[Piece::BQ as usize],
            ),
            Color::White => (
                pieces[Piece::WP as usize],
                Self::BLACK_PAWN_ATTACKS,
                pieces[Piece::WN as usize],
                pieces[Piece::WR as usize] | pieces[Piece::WQ as usize],
                pieces[Piece::WB as usize] | pieces[Piece::WQ as usize],
            ),
        };
        let pawn_attacks = pawn_attack_table[king_sq as usize];
        checkers |= pawn_attacks & pawns;

        let knight_attacks = Self::KNIGHT_ATTACKS[king_sq as usize];
        checkers |= knight_attacks & knights;

        let diag_mask = Self::BISHOP_MASKS[king_sq as usize];
        let diag_blockers = diag_mask & occ;
        let diag_idx = (diag_blockers.wrapping_mul(Self::BISHOP_MAGIC_NUMBERS[king_sq as usize])
            >> Self::BISHOP_SHIFTS[king_sq as usize]) as usize;
        let diag_attacks = Self::BISHOP_ATTACKS[king_sq as usize][diag_idx];
        checkers |= diag_attacks & diag_pieces;

        let orth_mask = Self::ROOK_MASKS[king_sq as usize];
        let orth_blockers = orth_mask & occ;
        let orth_idx = (orth_blockers.wrapping_mul(Self::ROOK_MAGIC_NUMBERS[king_sq as usize])
            >> Self::ROOK_SHIFTS[king_sq as usize]) as usize;
        let orth_attacks = Self::ROOK_ATTACKS[king_sq as usize][orth_idx];
        checkers |= orth_attacks & orth_pieces;

        checkers
    }

    const fn direction(from_i: i32, to_i: i32) -> i32 {
        let (from_rank, from_file) = (from_i / 8, from_i % 8);
        let (to_rank, to_file) = (to_i / 8, to_i % 8);

        let (dr, df) = (to_rank - from_rank, to_file - from_file);

        let (step_r, step_f) = (
            if dr == 0 {
                0
            } else if dr > 0 {
                1
            } else {
                -1
            },
            if df == 0 {
                0
            } else if df > 0 {
                1
            } else {
                -1
            },
        );

        if dr == 0 || df == 0 || dr.abs() == df.abs() {
            step_r * 8 + step_f
        } else {
            0
        }
    }

    const fn between_mask(from: usize, to: usize) -> u64 {
        let from_i = from as i32;
        let to_i = to as i32;

        let step = Self::direction(from_i, to_i);

        if step == 0 {
            return 0;
        }

        let mut sq = from_i + step;
        let mut mask = 0u64;

        while sq != to_i {
            mask |= 1u64 << sq;
            sq += step;
        }

        mask
    }

    const fn gen_between() -> [[u64; 64]; 64] {
        let mut table = [[0u64; 64]; 64];

        let mut from = 0;
        while from < 64 {
            let mut to = 0;
            while to < 64 {
                table[from][to] = Self::between_mask(from, to);
                to += 1;
            }
            from += 1;
        }

        table
    }

    const BETWEEN: [[u64; 64]; 64] = Self::gen_between();

    fn compute_pins(&self, king_sq: u8) -> (u64, [u64; 64]) {
        let mut pinned = 0;
        let mut pin_rays = [0; 64];

        let (orth_pieces, diag_pieces) = match self.side_to_move {
            Color::White => (
                self.pieces[Piece::BR as usize] | self.pieces[Piece::BQ as usize],
                self.pieces[Piece::BB as usize] | self.pieces[Piece::BQ as usize],
            ),
            Color::Black => (
                self.pieces[Piece::WR as usize] | self.pieces[Piece::WQ as usize],
                self.pieces[Piece::WB as usize] | self.pieces[Piece::WQ as usize],
            ),
        };

        let potential_diag_attackers = Self::BISHOP_ATTACKS[king_sq as usize][0] & diag_pieces;
        let potential_orth_attackers = Self::ROOK_ATTACKS[king_sq as usize][0] & orth_pieces;

        let mut attackers = potential_diag_attackers | potential_orth_attackers;

        while attackers != 0 {
            let attacker_sq = attackers.trailing_zeros() as usize;
            attackers &= attackers - 1;

            let between = Self::BETWEEN[king_sq as usize][attacker_sq];
            let blockers = between & self.occ;
            let our_blockers = between & self.color_occ[self.side_to_move as usize];

            if blockers.count_ones() == 1 && our_blockers != 0 {
                let pinned_sq = blockers.trailing_zeros() as usize;
                pinned |= blockers;
                pin_rays[pinned_sq] = between | (1u64 << attacker_sq);
            }
        }

        (pinned, pin_rays)
    }

    fn compute_block_mask(&self, king_sq: u8, checkers: u64) -> u64 {
        let checker_sq = checkers.trailing_zeros();
        let checker_piece = self.piece_at(checker_sq as u8);

        match checker_piece {
            Piece::BN | Piece::WN | Piece::BP | Piece::WP => 0,
            Piece::BR | Piece::WR | Piece::BB | Piece::WB | Piece::BQ | Piece::WQ => {
                Self::BETWEEN[king_sq as usize][checker_sq as usize]
            }
            _ => 0,
        }
    }

    fn gen_castling_moves(&self, moves: &mut Vec<Move>, king_danger_squares: u64) {
        let castling_rights = self.castling;
        match self.side_to_move {
            Color::White => {
                if castling_rights & WK_CASTLE != 0
                    && self.occ & ((1u64 << 5) | (1u64 << 6)) == 0
                    && king_danger_squares & ((1u64 << 4) | (1u64 << 5) | (1u64 << 6)) == 0
                {
                    moves.push(Move::new(
                        4,
                        6,
                        Piece::WK,
                        Piece::None,
                        Piece::None,
                        CASTLING_FLAG,
                    ));
                }
                if castling_rights & WQ_CASTLE != 0
                    && self.occ & ((1u64 << 1) | (1u64 << 2) | (1u64 << 3)) == 0
                    && king_danger_squares & ((1u64 << 4) | (1u64 << 3) | (1u64 << 2)) == 0
                {
                    moves.push(Move::new(
                        4,
                        2,
                        Piece::WK,
                        Piece::None,
                        Piece::None,
                        CASTLING_FLAG,
                    ));
                }
            }
            Color::Black => {
                if castling_rights & BK_CASTLE != 0
                    && self.occ & ((1u64 << 61) | (1u64 << 62)) == 0
                    && king_danger_squares & ((1u64 << 60) | (1u64 << 61) | (1u64 << 62)) == 0
                {
                    moves.push(Move::new(
                        60,
                        62,
                        Piece::BK,
                        Piece::None,
                        Piece::None,
                        CASTLING_FLAG,
                    ));
                }
                if castling_rights & BQ_CASTLE != 0
                    && self.occ & ((1u64 << 57) | (1u64 << 58) | (1u64 << 59)) == 0
                    && king_danger_squares & ((1u64 << 60) | (1u64 << 59) | (1u64 << 58)) == 0
                {
                    moves.push(Move::new(
                        60,
                        58,
                        Piece::BK,
                        Piece::None,
                        Piece::None,
                        CASTLING_FLAG,
                    ));
                }
            }
        };
    }

    fn get_non_king_moves(
        &self,
        moves: &mut Vec<Move>,
        block_mask: u64,
        capture_mask: u64,
        pinned: u64,
        pin_rays: [u64; 64],
    ) {
        match self.side_to_move {
            Color::White => {
                self.get_white_pawn_moves(moves, block_mask, capture_mask, pinned, &pin_rays);
                self.get_knight_moves(moves, Piece::WN, block_mask, capture_mask, pinned);
                self.get_rook_moves(
                    moves,
                    Piece::WR,
                    block_mask,
                    capture_mask,
                    pinned,
                    &pin_rays,
                );
                self.get_bishop_moves(
                    moves,
                    Piece::WB,
                    block_mask,
                    capture_mask,
                    pinned,
                    &pin_rays,
                );
                self.get_queen_moves(
                    moves,
                    Piece::WQ,
                    block_mask,
                    capture_mask,
                    pinned,
                    &pin_rays,
                );
            }
            Color::Black => {
                self.get_black_pawn_moves(moves, block_mask, capture_mask, pinned, &pin_rays);
                self.get_knight_moves(moves, Piece::BN, block_mask, capture_mask, pinned);
                self.get_rook_moves(
                    moves,
                    Piece::BR,
                    block_mask,
                    capture_mask,
                    pinned,
                    &pin_rays,
                );
                self.get_bishop_moves(
                    moves,
                    Piece::BB,
                    block_mask,
                    capture_mask,
                    pinned,
                    &pin_rays,
                );
                self.get_queen_moves(
                    moves,
                    Piece::BQ,
                    block_mask,
                    capture_mask,
                    pinned,
                    &pin_rays,
                );
            }
        }
    }

    pub fn get_legal_moves(&self, moves: &mut Vec<Move>) {
        moves.clear();
        let us = self.side_to_move;

        let king_sq = self.pieces[if us == Color::White {
            Piece::WK
        } else {
            Piece::BK
        } as usize]
            .trailing_zeros() as u8;
        debug_assert!(king_sq <= 64);

        let king_danger_mask = self.compute_danger(king_sq); // Squares the king cannot move to, i.e attacked squares if the king did not exist

        self.get_king_moves(moves, king_sq, king_danger_mask);
        let checkers: u64 =
            self.compute_checkers(king_sq, self.occ, self.side_to_move.flip(), &self.pieces); // Bitmap of pieces that put the king in check

        if checkers.count_ones() > 1 {
            return; // If it is a double check, only king moves can be legal
        }

        // pinned: bitmap of pieces that cannot move because they would put the king in check
        // pin_rays: list of bitmaps for each square a piece could be on, that show where that piece can move without puting the king in check
        let (pinned, pin_rays) = self.compute_pins(king_sq);

        let in_check = checkers != 0;
        // block_mask: bitmap of squares a piece can go to in order to stop the check (if the checker is a sliding piece, otherwise no squares can block it without capturing)
        let (block_mask, capture_mask) = if in_check {
            (self.compute_block_mask(king_sq, checkers), checkers)
        } else {
            self.gen_castling_moves(moves, king_danger_mask); // Castling moves can only happen when not in check
            (!0u64, !0u64) // All squares are valid to move to and capture on if not check are in place (except for pins)
        };

        self.get_non_king_moves(moves, block_mask, capture_mask, pinned, pin_rays); // Careful for ep
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

    pub fn is_king_checked(&self) -> bool {
        let attacker_color = self.side_to_move.flip();
        let king_sq = match attacker_color {
            Color::Black => self.pieces[Piece::WK as usize].trailing_zeros() as u8,
            Color::White => self.pieces[Piece::BK as usize].trailing_zeros() as u8,
        };
        self.compute_checkers(king_sq, self.occ, attacker_color, &self.pieces) != 0
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
}
