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

pub struct Undo {
    pub castling: u8,
    pub en_passant: u8,
    pub halfmove_clock: u8,
    pub captured_piece: Option<Piece>,
    pub zobrist: u64,
}

#[derive(Copy, PartialEq, Clone, Debug)]
pub struct Move {
    pub from: u8,
    pub to: u8,
    flags: u8,
    pub promotion: Option<Piece>,
}

pub struct Board {
    pub pieces: [u64; 12],
    pub white_occ: u64,
    pub black_occ: u64,
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

impl Color {
    pub fn flip(&self) -> Self {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

impl Undo {
    pub fn new(board: &Board, captured_piece: Option<Piece>) -> Self {
        Undo {
            castling: board.castling,
            en_passant: board.en_passant,
            halfmove_clock: board.halfmove_clock,
            captured_piece,
            zobrist: board.zobrist,
        }
    }
}

impl Move {
    pub fn new(from: u8, to: u8, flags: u8, promotion: Option<Piece>) -> Self {
        Move {
            from,
            to,
            flags,
            promotion,
        }
    }
}

impl Board {
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

        let white_occ = pieces[Piece::WP as usize]
            | pieces[Piece::WN as usize]
            | pieces[Piece::WB as usize]
            | pieces[Piece::WR as usize]
            | pieces[Piece::WQ as usize]
            | pieces[Piece::WK as usize];

        let black_occ = pieces[Piece::BP as usize]
            | pieces[Piece::BN as usize]
            | pieces[Piece::BB as usize]
            | pieces[Piece::BR as usize]
            | pieces[Piece::BQ as usize]
            | pieces[Piece::BK as usize];

        let occ = white_occ | black_occ;

        let side_to_move = Color::White;

        let castling = WK_CASTLE | WQ_CASTLE | BK_CASTLE | BQ_CASTLE;

        let en_passant = 0;
        let halfmove_clock = 0;
        let zobrist = 0;
        let history = Vec::new();
        let undo_stack = Vec::new();

        Board {
            pieces,
            white_occ,
            black_occ,
            occ,
            side_to_move,
            castling,
            en_passant,
            halfmove_clock,
            zobrist,
            history,
            undo_stack,
        }
    }
    pub fn piece_at(&self, sq: u8) -> Option<Piece> {
        let mask = 1 << sq;

        Piece::PIECE_LIST
            .into_iter()
            .find(|&p| self.pieces[p as usize] & mask != 0)
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
        let is_capture = mv.flags & CAPTURE_FLAG != 0;
        let is_en_passant = mv.flags & EN_PASSANT_FLAG != 0;
        let is_promotion = mv.flags & PROMOTION_FLAG != 0;
        let is_castling = mv.flags & CASTLING_FLAG != 0;
        let mut moving_side_mask = 0u64;
        let mut opponent_side_mask = 0u64;
        let us = self.side_to_move;

        // Remove piece if capture
        let captured_piece = if is_capture {
            let (c_sq, c_p) = if is_en_passant {
                if us == Color::White {
                    (mv.to - 8, Piece::BP)
                } else {
                    (mv.to + 8, Piece::WP)
                }
            } else {
                (mv.to, self.piece_at(mv.to).unwrap())
            };
            self.remove_piece(c_p, c_sq);
            let mask = 1u64 << c_sq;
            opponent_side_mask |= mask;
            Some(c_p)
        } else {
            None
        };

        // Save undo information
        let undo = Undo::new(self, captured_piece);
        self.undo_stack.push(undo);

        // Move the played piece (plus rook for castling)
        let moved_piece = self.piece_at(mv.from).unwrap();
        let moved_mask = (1u64 << mv.from) | (1u64 << mv.to);
        moving_side_mask |= moved_mask;

        if is_promotion {
            self.remove_piece(moved_piece, mv.from);
            let promoted_piece = mv.promotion.unwrap();
            self.add_piece(promoted_piece, mv.to);
        } else {
            self.move_piece(moved_piece, mv.from, mv.to);

            if is_castling {
                let (rook, rook_from, rook_to) = match mv.to {
                    62 => (Piece::BR, 63, 61), // black king side
                    58 => (Piece::BR, 56, 59), // black queen side
                    6 => (Piece::WR, 7, 5),    // white king side
                    2 => (Piece::WR, 0, 3),    // white queen side
                    _ => unreachable!(),
                };

                self.move_piece(rook, rook_from, rook_to);

                let rook_mask = (1u64 << rook_from) | (1u64 << rook_to);
                moving_side_mask |= rook_mask;
            }
        }

        // Update occupancy
        self.occ ^= moving_side_mask ^ opponent_side_mask;
        match self.side_to_move {
            Color::White => {
                self.white_occ ^= moving_side_mask;
                self.black_occ ^= opponent_side_mask;
            }
            Color::Black => {
                self.black_occ ^= moving_side_mask;
                self.white_occ ^= opponent_side_mask;
            }
        }

        // Update en passant square
        let is_pawn_move = matches!(moved_piece, Piece::WP | Piece::BP);
        self.en_passant = if is_pawn_move && ((mv.from as i16 - mv.to as i16).abs() == 16) {
            (mv.from + mv.to) / 2
        } else {
            64
        };

        // Update castling rights
        match moved_piece {
            Piece::WK => self.castling &= !(WK_CASTLE | WQ_CASTLE),
            Piece::BK => self.castling &= !(BK_CASTLE | BQ_CASTLE),
            Piece::WR => {
                if mv.from == 7 {
                    self.castling &= !WK_CASTLE
                };
                if mv.from == 0 {
                    self.castling &= !WQ_CASTLE
                };
            }
            Piece::BR => {
                if mv.from == 63 {
                    self.castling &= !BK_CASTLE
                };
                if mv.from == 56 {
                    self.castling &= !BQ_CASTLE
                };
            }
            _ => (),
        }

        if let Some(piece) = captured_piece {
            match piece {
                Piece::WR => {
                    if mv.to == 7 {
                        self.castling &= !WK_CASTLE
                    }
                    if mv.to == 0 {
                        self.castling &= !WQ_CASTLE
                    }
                }
                Piece::BR => {
                    if mv.to == 63 {
                        self.castling &= !BK_CASTLE
                    }
                    if mv.to == 56 {
                        self.castling &= !BQ_CASTLE
                    }
                }
                _ => (),
            }
        }

        // Update halfmove clock
        self.halfmove_clock = if is_capture || is_pawn_move {
            0
        } else {
            self.halfmove_clock + 1
        };

        // Update side_to_move
        self.side_to_move = self.side_to_move.flip();
    }

    fn unmake_move(&mut self, mv: Move) {
        // Pop undo stack and restore previous state
        let undo = self.undo_stack.pop().unwrap();
        self.castling = undo.castling;
        self.en_passant = undo.en_passant;
        self.halfmove_clock = undo.halfmove_clock;
        self.zobrist = undo.zobrist;
        self.side_to_move = self.side_to_move.flip();

        let is_capture = mv.flags & CAPTURE_FLAG != 0;
        let is_en_passant = mv.flags & EN_PASSANT_FLAG != 0;
        let is_castling = mv.flags & CASTLING_FLAG != 0;
        let is_promotion = mv.flags & PROMOTION_FLAG != 0;
        let mut moving_side_mask: u64 = 0;
        let mut opponent_side_mask: u64 = 0;

        // Move back the piece (checking for promotion and castling)
        moving_side_mask |= (1u64 << mv.to) | (1u64 << mv.from);
        if is_promotion {
            let promoted_piece = mv.promotion.unwrap();
            self.remove_piece(promoted_piece, mv.to);

            match self.side_to_move {
                Color::White => {
                    self.add_piece(Piece::WP, mv.from);
                }
                Color::Black => {
                    self.add_piece(Piece::BP, mv.from);
                }
            }
        } else {
            let moved_piece = self.piece_at(mv.to).unwrap();
            self.move_piece(moved_piece, mv.to, mv.from);

            if is_castling {
                let (rook, rook_from, rook_to) = match mv.to {
                    62 => (Piece::BR, 61, 63), // black king side
                    58 => (Piece::BR, 59, 56), // black queen side
                    6 => (Piece::WR, 5, 7),    // white king side
                    2 => (Piece::WR, 3, 0),    // white queen side
                    _ => unreachable!(),
                };

                self.move_piece(rook, rook_from, rook_to);

                let rook_mask = (1u64 << rook_from) | (1u64 << rook_to);
                moving_side_mask |= rook_mask;
            }
        }

        // In case of capture (and maybe en passant), restore the taken piece
        if is_capture {
            let sq = if is_en_passant {
                match self.side_to_move {
                    Color::White => mv.to - 8,
                    Color::Black => mv.to + 8,
                }
            } else {
                mv.to
            };
            let sq_mask = 1u64 << sq;
            opponent_side_mask |= sq_mask;
            self.add_piece(undo.captured_piece.unwrap(), sq);
        }

        // Recompute occupancy
        self.occ ^= moving_side_mask ^ opponent_side_mask;
        match self.side_to_move {
            Color::White => {
                self.white_occ ^= moving_side_mask;
                self.black_occ ^= opponent_side_mask;
            }
            Color::Black => {
                self.black_occ ^= moving_side_mask;
                self.white_occ ^= opponent_side_mask;
            }
        }
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

    fn get_white_pawn_moves(&self, moves: &mut Vec<Move>) {
        let pawns = self.pieces[Piece::WP as usize];
        let empty = !self.occ;

        // Single pushes
        let mut single_push = (pawns << 8) & empty;
        while single_push != 0 {
            let to = single_push.trailing_zeros() as u8;
            single_push &= single_push - 1;
            let from = to - 8;
            let is_promotion = to > 55;
            if is_promotion {
                moves.push(Move::new(from, to, PROMOTION_FLAG, Some(Piece::WN)));
                moves.push(Move::new(from, to, PROMOTION_FLAG, Some(Piece::WB)));
                moves.push(Move::new(from, to, PROMOTION_FLAG, Some(Piece::WR)));
                moves.push(Move::new(from, to, PROMOTION_FLAG, Some(Piece::WQ)));
            } else {
                moves.push(Move::new(from, to, 0, None));
            }
        }

        // Double pushes
        let first_push = ((pawns & RANK_2) << 8) & empty;
        let mut double_push = (first_push << 8) & empty;
        while double_push != 0 {
            let to = double_push.trailing_zeros() as u8;
            double_push &= double_push - 1;
            let from = to - 16;
            moves.push(Move::new(from, to, 0, None));
        }

        // Captures
        let mut captures = pawns;
        while captures != 0 {
            let from = captures.trailing_zeros() as u8;
            captures &= captures - 1;

            let mut attacks = Self::WHITE_PAWN_ATTACKS[from as usize] & self.black_occ;

            while attacks != 0 {
                let to = attacks.trailing_zeros() as u8;
                attacks &= attacks - 1;

                let is_promotion = to > 55;
                if is_promotion {
                    moves.push(Move::new(
                        from,
                        to,
                        CAPTURE_FLAG | PROMOTION_FLAG,
                        Some(Piece::WN),
                    ));
                    moves.push(Move::new(
                        from,
                        to,
                        CAPTURE_FLAG | PROMOTION_FLAG,
                        Some(Piece::WB),
                    ));
                    moves.push(Move::new(
                        from,
                        to,
                        CAPTURE_FLAG | PROMOTION_FLAG,
                        Some(Piece::WR),
                    ));
                    moves.push(Move::new(
                        from,
                        to,
                        CAPTURE_FLAG | PROMOTION_FLAG,
                        Some(Piece::WQ),
                    ));
                } else {
                    moves.push(Move::new(from, to, CAPTURE_FLAG, None));
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
                moves.push(Move::new(from, to, CAPTURE_FLAG | EN_PASSANT_FLAG, None));
            }
            let mut right_en_passant = ((pawns << 9) & !FILE_A) & en_passant_mask;
            while right_en_passant != 0 {
                let to = right_en_passant.trailing_zeros() as u8;
                right_en_passant &= right_en_passant - 1;
                let from = to - 9;
                moves.push(Move::new(from, to, CAPTURE_FLAG | EN_PASSANT_FLAG, None));
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

    fn get_black_pawn_moves(&self, moves: &mut Vec<Move>) {
        let pawns = self.pieces[Piece::BP as usize];
        let empty = !self.occ;

        // Single pushes
        let mut single_push = (pawns >> 8) & empty;
        while single_push != 0 {
            let to = single_push.trailing_zeros() as u8;
            single_push &= single_push - 1;
            let from = to + 8;
            let is_promotion = to < 8;
            if is_promotion {
                moves.push(Move::new(from, to, PROMOTION_FLAG, Some(Piece::BN)));
                moves.push(Move::new(from, to, PROMOTION_FLAG, Some(Piece::BB)));
                moves.push(Move::new(from, to, PROMOTION_FLAG, Some(Piece::BR)));
                moves.push(Move::new(from, to, PROMOTION_FLAG, Some(Piece::BQ)));
            } else {
                moves.push(Move::new(from, to, 0, None));
            }
        }

        // Double pushes
        let first_push = ((pawns & RANK_7) >> 8) & empty;
        let mut double_push = (first_push >> 8) & empty;
        while double_push != 0 {
            let to = double_push.trailing_zeros() as u8;
            double_push &= double_push - 1;
            let from = to + 16;
            moves.push(Move::new(from, to, 0, None));
        }

        // Captures
        let mut captures = pawns;
        while captures != 0 {
            let from = captures.trailing_zeros() as u8;
            captures &= captures - 1;

            let mut attacks = Self::BLACK_PAWN_ATTACKS[from as usize] & self.white_occ;

            while attacks != 0 {
                let to = attacks.trailing_zeros() as u8;
                attacks &= attacks - 1;

                let is_promotion = to < 8;
                if is_promotion {
                    moves.push(Move::new(
                        from,
                        to,
                        CAPTURE_FLAG | PROMOTION_FLAG,
                        Some(Piece::BN),
                    ));
                    moves.push(Move::new(
                        from,
                        to,
                        CAPTURE_FLAG | PROMOTION_FLAG,
                        Some(Piece::BB),
                    ));
                    moves.push(Move::new(
                        from,
                        to,
                        CAPTURE_FLAG | PROMOTION_FLAG,
                        Some(Piece::BR),
                    ));
                    moves.push(Move::new(
                        from,
                        to,
                        CAPTURE_FLAG | PROMOTION_FLAG,
                        Some(Piece::BQ),
                    ));
                } else {
                    moves.push(Move::new(from, to, CAPTURE_FLAG, None));
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
                moves.push(Move::new(from, to, CAPTURE_FLAG | EN_PASSANT_FLAG, None));
            }
            let mut right_en_passant = ((pawns >> 9) & !FILE_H) & en_passant_mask;
            while right_en_passant != 0 {
                let to = right_en_passant.trailing_zeros() as u8;
                right_en_passant &= right_en_passant - 1;
                let from = to + 9;
                moves.push(Move::new(from, to, CAPTURE_FLAG | EN_PASSANT_FLAG, None));
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
        mut knights: u64,
        our_occ: u64,
        their_occ: u64,
    ) {
        while knights != 0 {
            let from = knights.trailing_zeros() as u8;
            knights &= knights - 1;

            let mut attacks = Self::KNIGHT_ATTACKS[from as usize] & !our_occ;

            while attacks != 0 {
                let to = attacks.trailing_zeros() as u8;
                attacks &= attacks - 1;

                let flag = if (1u64 << to) & their_occ != 0 {
                    CAPTURE_FLAG
                } else {
                    0
                };

                moves.push(Move::new(from, to, flag, None));
            }
        }
    }

    const DIR_N: usize = 0;
    const DIR_NE: usize = 1;
    const DIR_E: usize = 2;
    const DIR_SE: usize = 3;
    const DIR_S: usize = 4;
    const DIR_SW: usize = 5;
    const DIR_W: usize = 6;
    const DIR_NW: usize = 7;

    const fn compute_rays() -> [[u64; 64]; 8] {
        let mut rays = [[0; 64]; 8];
        let mut sq = 0;
        while sq < 64 {
            let r = sq / 8;
            let f = sq % 8;

            let mut i = r + 1;
            while i < 8 {
                let target = i * 8 + f;
                rays[Self::DIR_N][sq] |= 1 << target;
                i += 1;
            }

            let mut i = r as i8 - 1;
            while i >= 0 {
                let target = i as usize * 8 + f;
                rays[Self::DIR_S][sq] |= 1 << target;
                i -= 1;
            }

            let mut j = f + 1;
            while j < 8 {
                let target = r * 8 + j;
                rays[Self::DIR_E][sq] |= 1 << target;
                j += 1;
            }

            let mut j = f as i8 - 1;
            while j >= 0 {
                let target = r * 8 + j as usize;
                rays[Self::DIR_W][sq] |= 1 << target;
                j -= 1;
            }

            let mut i = r + 1;
            let mut j = f + 1;
            while i < 8 && j < 8 {
                let target = i * 8 + j;
                rays[Self::DIR_NE][sq] |= 1 << target;
                i += 1;
                j += 1;
            }

            let mut i = r + 1;
            let mut j = f as i8 - 1;
            while i < 8 && j >= 0 {
                let target = i * 8 + j as usize;
                rays[Self::DIR_NW][sq] |= 1 << target;
                i += 1;
                j -= 1;
            }

            let mut i = r as i8 - 1;
            let mut j = f + 1;
            while i >= 0 && j < 8 {
                let target = i as usize * 8 + j;
                rays[Self::DIR_SE][sq] |= 1 << target;
                i -= 1;
                j += 1;
            }

            let mut i = r as i8 - 1;
            let mut j = f as i8 - 1;
            while i >= 0 && j >= 0 {
                let target = i as usize * 8 + j as usize;
                rays[Self::DIR_SW][sq] |= 1 << target;
                i -= 1;
                j -= 1;
            }

            sq += 1;
        }

        rays
    }

    const RAYS: [[u64; 64]; 8] = Self::compute_rays();

    const fn dir_is_pos(d: usize) -> bool {
        matches!(d, Self::DIR_N | Self::DIR_NE | Self::DIR_E | Self::DIR_NW)
    }

    fn gen_sliding_moves(&self, sq: u8, directions: &[usize]) -> u64 {
        let mut attacks = 0u64;
        for dir in directions {
            let ray = Self::RAYS[*dir][sq as usize];
            let blockers = ray & self.occ;
            if blockers == 0 {
                attacks |= ray;
            } else {
                let first_blocker_sq = if Self::dir_is_pos(*dir) {
                    blockers.trailing_zeros() as u8
                } else {
                    (63 - blockers.leading_zeros()) as u8
                };
                attacks |= ray & !(Self::RAYS[*dir][first_blocker_sq as usize]);
            }
        }
        attacks
    }

    const BISHOP_DIRECTIONS: [usize; 4] = [Self::DIR_SW, Self::DIR_SE, Self::DIR_NW, Self::DIR_NE];

    fn get_bishop_moves(
        &self,
        moves: &mut Vec<Move>,
        mut bishops: u64,
        our_occ: u64,
        their_occ: u64,
    ) {
        while bishops != 0 {
            let from = bishops.trailing_zeros() as u8;
            bishops &= bishops - 1;
            let attacks = self.gen_sliding_moves(from, &Self::BISHOP_DIRECTIONS) & !our_occ;
            let mut non_captures = attacks & !their_occ;
            let mut captures = attacks & their_occ;
            while non_captures != 0 {
                let to = non_captures.trailing_zeros() as u8;
                non_captures &= non_captures - 1;
                moves.push(Move::new(from, to, 0, None));
            }
            while captures != 0 {
                let to = captures.trailing_zeros() as u8;
                captures &= captures - 1;
                moves.push(Move::new(from, to, CAPTURE_FLAG, None));
            }
        }
    }

    const ROOK_DIRECTIONS: [usize; 4] = [Self::DIR_N, Self::DIR_S, Self::DIR_W, Self::DIR_E];

    fn get_rook_moves(&self, moves: &mut Vec<Move>, mut rooks: u64, our_occ: u64, their_occ: u64) {
        while rooks != 0 {
            let from = rooks.trailing_zeros() as u8;
            rooks &= rooks - 1;
            let attacks = self.gen_sliding_moves(from, &Self::ROOK_DIRECTIONS) & !our_occ;
            let mut non_captures = attacks & !their_occ;
            let mut captures = attacks & their_occ;
            while non_captures != 0 {
                let to = non_captures.trailing_zeros() as u8;
                non_captures &= non_captures - 1;
                moves.push(Move::new(from, to, 0, None));
            }
            while captures != 0 {
                let to = captures.trailing_zeros() as u8;
                captures &= captures - 1;
                moves.push(Move::new(from, to, CAPTURE_FLAG, None));
            }
        }
    }

    fn get_queen_moves(
        &self,
        moves: &mut Vec<Move>,
        mut queens: u64,
        our_occ: u64,
        their_occ: u64,
    ) {
        const QUEEN_DIRECTIONS: [usize; 8] = [
            Board::DIR_N,
            Board::DIR_NE,
            Board::DIR_E,
            Board::DIR_SE,
            Board::DIR_S,
            Board::DIR_SW,
            Board::DIR_W,
            Board::DIR_NW,
        ];

        while queens != 0 {
            let from = queens.trailing_zeros() as u8;
            queens &= queens - 1;
            let attacks = self.gen_sliding_moves(from, &QUEEN_DIRECTIONS) & !our_occ;
            let mut non_captures = attacks & !their_occ;
            let mut captures = attacks & their_occ;
            while non_captures != 0 {
                let to = non_captures.trailing_zeros() as u8;
                non_captures &= non_captures - 1;
                moves.push(Move::new(from, to, 0, None));
            }
            while captures != 0 {
                let to = captures.trailing_zeros() as u8;
                captures &= captures - 1;
                moves.push(Move::new(from, to, CAPTURE_FLAG, None));
            }
        }
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

    fn get_king_moves(&self, moves: &mut Vec<Move>, king: u64, our_occ: u64, their_occ: u64) {
        let from = king.trailing_zeros() as u8;
        debug_assert!(from <= 64);

        // Generate the classical moves
        let mut attacks = Self::KING_ATTACKS[from as usize] & !our_occ;
        while attacks != 0 {
            let to = attacks.trailing_zeros() as u8;
            attacks &= attacks - 1;

            let flag = if their_occ & (1u64 << to) != 0 {
                CAPTURE_FLAG
            } else {
                0
            };

            moves.push(Move::new(from, to, flag, None));
        }

        // Generate castling moves
        if self.side_to_move == Color::White {
            if self.castling & WK_CASTLE != 0 && self.occ & ((1u64 << 5) | (1u64 << 6)) == 0 {
                moves.push(Move::new(4, 6, CASTLING_FLAG, None));
            }
            if self.castling & WQ_CASTLE != 0
                && self.occ & ((1u64 << 3) | (1u64 << 2) | (1u64 << 1)) == 0
            {
                moves.push(Move::new(4, 2, CASTLING_FLAG, None));
            }
        } else {
            if self.castling & BK_CASTLE != 0 && self.occ & ((1u64 << 61) | (1u64 << 62)) == 0 {
                moves.push(Move::new(60, 62, CASTLING_FLAG, None));
            }
            if self.castling & BQ_CASTLE != 0
                && self.occ & ((1u64 << 59) | (1u64 << 58) | (1u64 << 57)) == 0
            {
                moves.push(Move::new(60, 58, CASTLING_FLAG, None));
            }
        }
    }

    pub fn get_pseudo_legal_moves(&self) -> Vec<Move> {
        let mut moves = Vec::with_capacity(218);

        // Single self.side_to_move match
        match self.side_to_move {
            Color::White => {
                let our_occ = self.white_occ;
                let their_occ = self.black_occ;
                self.get_white_pawn_moves(&mut moves);
                self.get_knight_moves(
                    &mut moves,
                    self.pieces[Piece::WN as usize],
                    our_occ,
                    their_occ,
                );
                self.get_bishop_moves(
                    &mut moves,
                    self.pieces[Piece::WB as usize],
                    our_occ,
                    their_occ,
                );
                self.get_rook_moves(
                    &mut moves,
                    self.pieces[Piece::WR as usize],
                    our_occ,
                    their_occ,
                );
                self.get_queen_moves(
                    &mut moves,
                    self.pieces[Piece::WQ as usize],
                    our_occ,
                    their_occ,
                );
                self.get_king_moves(
                    &mut moves,
                    self.pieces[Piece::WK as usize],
                    our_occ,
                    their_occ,
                );
            }
            Color::Black => {
                let our_occ = self.black_occ;
                let their_occ = self.white_occ;
                self.get_black_pawn_moves(&mut moves);
                self.get_knight_moves(
                    &mut moves,
                    self.pieces[Piece::BN as usize],
                    our_occ,
                    their_occ,
                );
                self.get_bishop_moves(
                    &mut moves,
                    self.pieces[Piece::BB as usize],
                    our_occ,
                    their_occ,
                );
                self.get_rook_moves(
                    &mut moves,
                    self.pieces[Piece::BR as usize],
                    our_occ,
                    their_occ,
                );
                self.get_queen_moves(
                    &mut moves,
                    self.pieces[Piece::BQ as usize],
                    our_occ,
                    their_occ,
                );
                self.get_king_moves(
                    &mut moves,
                    self.pieces[Piece::BK as usize],
                    our_occ,
                    their_occ,
                );
            }
        }

        moves
    }

    fn is_square_attacked(&self, sq: u8, by: Color) -> bool {
        let (pawns, knights, bishops, rooks, queens, king) = match by {
            Color::White => (
                self.pieces[Piece::WP as usize],
                self.pieces[Piece::WN as usize],
                self.pieces[Piece::WB as usize],
                self.pieces[Piece::WR as usize],
                self.pieces[Piece::WQ as usize],
                self.pieces[Piece::WK as usize],
            ),
            Color::Black => (
                self.pieces[Piece::BP as usize],
                self.pieces[Piece::BN as usize],
                self.pieces[Piece::BB as usize],
                self.pieces[Piece::BR as usize],
                self.pieces[Piece::BQ as usize],
                self.pieces[Piece::BK as usize],
            ),
        };

        // Pawn attacks
        match by {
            Color::White => {
                if (Self::BLACK_PAWN_ATTACKS[sq as usize] & pawns) != 0 {
                    return true;
                }
            }
            Color::Black => {
                if (Self::WHITE_PAWN_ATTACKS[sq as usize] & pawns) != 0 {
                    return true;
                }
            }
        }

        // Knight attacks
        if (Self::KNIGHT_ATTACKS[sq as usize] & knights) != 0 {
            return true;
        }

        // Bishop/Queen attacks
        for &dir in &Self::BISHOP_DIRECTIONS {
            let ray = Self::RAYS[dir][sq as usize];
            let blockers = ray & self.occ;
            if blockers != 0 {
                let first = if Self::dir_is_pos(dir) {
                    blockers.trailing_zeros() as u8
                } else {
                    (63 - blockers.leading_zeros()) as u8
                };
                if (1u64 << first) & (bishops | queens) != 0 {
                    return true;
                }
            }
        }

        // Rook/Queen attacks
        for &dir in &Self::ROOK_DIRECTIONS {
            let ray = Self::RAYS[dir][sq as usize];
            let blockers = ray & self.occ;
            if blockers != 0 {
                let first = if Self::dir_is_pos(dir) {
                    blockers.trailing_zeros() as u8
                } else {
                    (63 - blockers.leading_zeros()) as u8
                };
                if (1u64 << first) & (rooks | queens) != 0 {
                    return true;
                }
            }
        }

        // King attacks
        if (Self::KING_ATTACKS[sq as usize] & king) != 0 {
            return true;
        }

        false
    }

    fn is_king_checked(&self, color: Color) -> bool {
        match color {
            Color::White => self.is_square_attacked(
                self.pieces[Piece::WK as usize].trailing_zeros() as u8,
                Color::Black,
            ),
            Color::Black => self.is_square_attacked(
                self.pieces[Piece::BK as usize].trailing_zeros() as u8,
                Color::White,
            ),
        }
    }

    fn is_castle_legal(&self, mv: Move, by: Color) -> bool {
        let through = match mv.to {
            6 => 5,
            2 => 3,
            62 => 61,
            58 => 59,
            _ => panic!("Castle check for non castle move"),
        };

        let enemy = by.flip();

        self.is_square_attacked(mv.from, enemy)
            || self.is_square_attacked(through, enemy)
            || self.is_square_attacked(mv.to, enemy)
    }
}

pub fn perft(board: &mut Board, depth: u8) -> u64 {
    if depth == 0 {
        return 1;
    }

    let mut nodes = 0;
    let moves = board.get_pseudo_legal_moves();

    for m in moves {
        // println!("Making move at depth {}: {:?}", depth, m);
        board.make_move(m);

        let illegal = if m.flags & CASTLING_FLAG != 0 {
            board.is_castle_legal(m, board.side_to_move.flip())
        } else {
            board.is_king_checked(board.side_to_move.flip())
        };
        if !illegal {
            nodes += perft(board, depth - 1);
        }

        // println!("Unmaking move at depth {}: {:?}", depth, m);
        board.unmake_move(m);
    }
    nodes
}

pub fn get_legal_moves(board: &mut Board) -> Vec<Move> {
    let pseudo_legal_moves = board.get_pseudo_legal_moves();
    let mut legal_moves = Vec::new();
    for mv in pseudo_legal_moves {
        board.make_move(mv);
        let illegal = if mv.flags & CASTLING_FLAG != 0 {
            board.is_castle_legal(mv, board.side_to_move.flip())
        } else {
            board.is_king_checked(board.side_to_move.flip())
        };
        if !illegal {
            legal_moves.push(mv);
        }
        board.unmake_move(mv);
    }

    legal_moves
}
