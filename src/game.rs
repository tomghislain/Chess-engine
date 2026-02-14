pub const FILE_A: u64 = 0x0101010101010101;
pub const FILE_B: u64 = 0x0202020202020202;
pub const FILE_G: u64 = 0x4040404040404040;
pub const FILE_H: u64 = 0x8080808080808080;

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

#[derive(Copy, Clone, Debug)]
pub struct Move {
    from: u8,
    to: u8,
    flags: u8,
    promotion: Option<Piece>,
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
    pub fn from_index(index: u8) -> Option<Self> {
        match index {
            0 => Some(Piece::WP),
            1 => Some(Piece::WN),
            2 => Some(Piece::WB),
            3 => Some(Piece::WR),
            4 => Some(Piece::WQ),
            5 => Some(Piece::WK),
            6 => Some(Piece::BP),
            7 => Some(Piece::BN),
            8 => Some(Piece::BB),
            9 => Some(Piece::BR),
            10 => Some(Piece::BQ),
            11 => Some(Piece::BK),
            _ => None,
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
    fn piece_at(&self, sq: u8) -> Option<Piece> {
        let mask = 1 << sq;

        for i in 0..self.pieces.len() {
            if self.pieces[i] & mask != 0 {
                return Piece::from_index(i as u8);
            }
        }
        None
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

    // Plays a move by mutating bitboards and state information, and pushes an undo entry on the stack
    pub fn make_move(&mut self, mv: Move) {
        // TODO: Update zobrist hash

        // Remove piece at mv.to if capture/behin mv.to if en passant
        let is_capture = (mv.flags & CAPTURE_FLAG) != 0;
        let captured_piece = if is_capture {
            if (mv.flags & EN_PASSANT_FLAG) != 0 {
                let (c_sq, c_p) = match self.side_to_move {
                    Color::White => (mv.to - 8, Piece::BP),
                    Color::Black => (mv.to + 8, Piece::WP),
                };
                self.remove_piece(c_p, c_sq);
                Some(c_p)
            } else {
                let c_p = self.piece_at(mv.to);
                self.remove_piece(c_p.unwrap(), mv.to);
                c_p
            }
        } else {
            None
        };

        // Once captured piece is computed, create and add undo to undo_stack
        let undo = Undo::new(self, captured_piece);
        self.undo_stack.push(undo);

        // Move piece from mv.from to mv.to (checking for promotion and castling)
        let moved_piece = self.piece_at(mv.from).unwrap();
        if (mv.flags & PROMOTION_FLAG) != 0 {
            self.remove_piece(moved_piece, mv.from);
            self.add_piece(mv.promotion.unwrap(), mv.to);
        } else {
            self.move_piece(moved_piece, mv.from, mv.to);
            if (mv.flags & CASTLING_FLAG) != 0 {
                match mv.to {
                    62 => self.move_piece(Piece::BR, 63, 61), // black king side
                    58 => self.move_piece(Piece::BR, 56, 59), // black queen side
                    6 => self.move_piece(Piece::WR, 7, 5),    // white king side
                    2 => self.move_piece(Piece::WR, 0, 3),    // white queen side
                    _ => unreachable!(),
                }
            }
        }
        // Update occupancies/en passant/castling rights/halfmove clock/side to move
        self.white_occ = {
            self.pieces[Piece::WP as usize]
                | self.pieces[Piece::WN as usize]
                | self.pieces[Piece::WB as usize]
                | self.pieces[Piece::WR as usize]
                | self.pieces[Piece::WQ as usize]
                | self.pieces[Piece::WK as usize]
        };
        self.black_occ = {
            self.pieces[Piece::BP as usize]
                | self.pieces[Piece::BN as usize]
                | self.pieces[Piece::BB as usize]
                | self.pieces[Piece::BR as usize]
                | self.pieces[Piece::BQ as usize]
                | self.pieces[Piece::BK as usize]
        };
        self.occ = self.white_occ | self.black_occ;

        let is_pawn_move = matches!(moved_piece, Piece::WP | Piece::BP);
        self.en_passant = if is_pawn_move && ((mv.from as i16 - mv.to as i16).abs() == 16) {
            (mv.from + mv.to) / 2
        } else {
            64
        };

        match moved_piece {
            Piece::WK => self.castling &= !(WK_CASTLE | WQ_CASTLE),
            Piece::BK => self.castling &= !(BK_CASTLE | BQ_CASTLE),
            Piece::WR => {
                if mv.from == 7 {
                    self.castling &= !WK_CASTLE
                }
                if mv.from == 0 {
                    self.castling &= !WQ_CASTLE
                }
            }
            Piece::BR => {
                if mv.from == 63 {
                    self.castling &= !BK_CASTLE
                }
                if mv.from == 56 {
                    self.castling &= !BQ_CASTLE
                }
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

        self.halfmove_clock = if is_capture || is_pawn_move {
            0
        } else {
            self.halfmove_clock + 1
        };

        self.side_to_move = match self.side_to_move {
            Color::White => Color::Black,
            Color::Black => Color::White,
        };
    }

    fn unmake_move(&mut self, mv: Move) {
        // Pop undo stack and restore previous state
        let undo = self.undo_stack.pop().unwrap();
        self.castling = undo.castling;
        self.en_passant = undo.en_passant;
        self.halfmove_clock = undo.halfmove_clock;
        self.zobrist = undo.zobrist;
        self.side_to_move = match self.side_to_move {
            Color::White => Color::Black,
            Color::Black => Color::White,
        };
        // Move back the piece (checking for promotion and castling)
        let moved_piece;
        if (mv.flags & PROMOTION_FLAG) != 0 {
            let promoted_piece = mv.promotion.unwrap();
            self.remove_piece(promoted_piece, mv.to);
            moved_piece = match self.side_to_move {
                Color::White => Piece::WP,
                Color::Black => Piece::BP,
            };
            self.add_piece(moved_piece, mv.from);
        } else {
            moved_piece = self.piece_at(mv.to).unwrap();
            self.move_piece(moved_piece, mv.to, mv.from);
            if (mv.flags & CASTLING_FLAG) != 0 {
                match mv.to {
                    62 => self.move_piece(Piece::BR, 61, 63), // white king side
                    58 => self.move_piece(Piece::BR, 59, 56), // white queen side
                    6 => self.move_piece(Piece::WR, 5, 7),    // black king side
                    2 => self.move_piece(Piece::WR, 3, 0),    // black queen side
                    _ => unreachable!(),
                }
            }
        }
        // In case of capture (and maybe en passant), restore the taken piece
        if (mv.flags & CAPTURE_FLAG) != 0 {
            if (mv.flags & EN_PASSANT_FLAG) != 0 {
                let (c_sq, c_p) = match moved_piece {
                    Piece::WP => (mv.to - 8, Piece::BP),
                    Piece::BP => (mv.to + 8, Piece::WP),
                    _ => unreachable!(),
                };
                self.add_piece(c_p, c_sq);
            } else {
                self.add_piece(undo.captured_piece.unwrap(), mv.to);
            }
        }
        // Recompute occupancy
        self.white_occ = {
            self.pieces[Piece::WP as usize]
                | self.pieces[Piece::WN as usize]
                | self.pieces[Piece::WB as usize]
                | self.pieces[Piece::WR as usize]
                | self.pieces[Piece::WQ as usize]
                | self.pieces[Piece::WK as usize]
        };
        self.black_occ = {
            self.pieces[Piece::BP as usize]
                | self.pieces[Piece::BN as usize]
                | self.pieces[Piece::BB as usize]
                | self.pieces[Piece::BR as usize]
                | self.pieces[Piece::BQ as usize]
                | self.pieces[Piece::BK as usize]
        };
        self.occ = self.white_occ | self.black_occ;
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
            if (56..64).contains(&(to as i32)) {
                moves.push(Move::new(from, to, PROMOTION_FLAG, Some(Piece::WN)));
                moves.push(Move::new(from, to, PROMOTION_FLAG, Some(Piece::WB)));
                moves.push(Move::new(from, to, PROMOTION_FLAG, Some(Piece::WR)));
                moves.push(Move::new(from, to, PROMOTION_FLAG, Some(Piece::WQ)));
            } else {
                moves.push(Move::new(from, to, 0, None));
            }
        }

        // Double pushes
        let rank2: u64 = 0x000000000000FF00;
        let first_push = ((pawns & rank2) << 8) & empty;
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

                let mut flag = CAPTURE_FLAG;

                if (56..64).contains(&(to as i32)) {
                    flag |= PROMOTION_FLAG;
                    moves.push(Move::new(from, to, flag, Some(Piece::WN)));
                    moves.push(Move::new(from, to, flag, Some(Piece::WB)));
                    moves.push(Move::new(from, to, flag, Some(Piece::WR)));
                    moves.push(Move::new(from, to, flag, Some(Piece::WQ)));
                } else {
                    moves.push(Move::new(from, to, flag, None));
                }
            }
        }

        // En passant
        let en_passant_mask = 1 << self.en_passant;
        if self.en_passant < 64 {
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
            if (0..8).contains(&(to as i32)) {
                moves.push(Move::new(from, to, PROMOTION_FLAG, Some(Piece::BN)));
                moves.push(Move::new(from, to, PROMOTION_FLAG, Some(Piece::BB)));
                moves.push(Move::new(from, to, PROMOTION_FLAG, Some(Piece::BR)));
                moves.push(Move::new(from, to, PROMOTION_FLAG, Some(Piece::BQ)));
            } else {
                moves.push(Move::new(from, to, 0, None));
            }
        }

        // Double pushes
        let rank7: u64 = 0x00FF000000000000;
        let first_push = ((pawns & rank7) >> 8) & empty;
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

                let mut flag = CAPTURE_FLAG;

                if (0..8).contains(&(to as i32)) {
                    flag |= PROMOTION_FLAG;
                    moves.push(Move::new(from, to, flag, Some(Piece::BN)));
                    moves.push(Move::new(from, to, flag, Some(Piece::BB)));
                    moves.push(Move::new(from, to, flag, Some(Piece::BR)));
                    moves.push(Move::new(from, to, flag, Some(Piece::BQ)));
                } else {
                    moves.push(Move::new(from, to, flag, None));
                }
            }
        }

        // En passant
        let en_passant_mask = 1 << self.en_passant;
        if self.en_passant < 64 {
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

    fn get_pawn_moves(&self, moves: &mut Vec<Move>) {
        match self.side_to_move {
            Color::White => self.get_white_pawn_moves(moves),
            Color::Black => self.get_black_pawn_moves(moves),
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

    fn get_knight_moves(&self, moves: &mut Vec<Move>, our_occ: u64, their_occ: u64) {
        let knights_index = match self.side_to_move {
            Color::White => Piece::WN as usize,
            Color::Black => Piece::BN as usize,
        };
        let mut knights = self.pieces[knights_index];

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

    fn gen_sliding_moves(&self, sq: u8, directions: &[i16]) -> u64 {
        let mut result = 0u64;

        for offset in directions {
            let mut current = sq as i16;
            loop {
                let next = current + offset;

                // Bound and wraparound checks
                if !(0..64).contains(&next)
                        // Check if file differs by more than 1
                        || ((next & 7) - (current & 7)).abs() > 1
                        // Check if rank differs by more than 1
                        || ((next >> 3) - (current >> 3)).abs() > 1
                {
                    break;
                }

                let next_mask = 1u64 << next;

                result |= next_mask;

                if self.occ & next_mask != 0 {
                    break;
                }

                current = next;
            }
        }
        result
    }

    const BISHOP_DIRECTION_OFFSETS: [i16; 4] = [9, 7, -7, -9];

    fn get_bishop_moves(&self, moves: &mut Vec<Move>, our_occ: u64, their_occ: u64) {
        let mut bishops = match self.side_to_move {
            Color::White => self.pieces[Piece::WB as usize],
            Color::Black => self.pieces[Piece::BB as usize],
        };

        while bishops != 0 {
            let from = bishops.trailing_zeros() as u8;
            bishops &= bishops - 1;
            let attacks = self.gen_sliding_moves(from, &Self::BISHOP_DIRECTION_OFFSETS) & !our_occ;
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

    const ROOK_DIRECTION_OFFSETS: [i16; 4] = [8, 1, -1, -8];

    fn get_rook_moves(&self, moves: &mut Vec<Move>, our_occ: u64, their_occ: u64) {
        let mut rooks = match self.side_to_move {
            Color::White => self.pieces[Piece::WR as usize],
            Color::Black => self.pieces[Piece::BR as usize],
        };

        while rooks != 0 {
            let from = rooks.trailing_zeros() as u8;
            rooks &= rooks - 1;
            let attacks = self.gen_sliding_moves(from, &Self::ROOK_DIRECTION_OFFSETS) & !our_occ;
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

    fn get_queen_moves(&self, moves: &mut Vec<Move>, our_occ: u64, their_occ: u64) {
        let mut queens = match self.side_to_move {
            Color::White => self.pieces[Piece::WQ as usize],
            Color::Black => self.pieces[Piece::BQ as usize],
        };

        const QUEEN_DIRECTION_OFFSETS: [i16; 8] = [9, 8, 7, 1, -1, -7, -8, -9];

        while queens != 0 {
            let from = queens.trailing_zeros() as u8;
            queens &= queens - 1;
            let attacks = self.gen_sliding_moves(from, &QUEEN_DIRECTION_OFFSETS) & !our_occ;
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

    fn get_king_moves(&self, moves: &mut Vec<Move>, our_occ: u64, their_occ: u64) {
        let king = match self.side_to_move {
            Color::White => self.pieces[Piece::WK as usize],
            Color::Black => self.pieces[Piece::BK as usize],
        };
        let from = king.trailing_zeros() as u8;

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
        let mut moves = Vec::new();

        let (our_occ, their_occ) = match self.side_to_move {
            Color::White => (self.white_occ, self.black_occ),
            Color::Black => (self.black_occ, self.white_occ),
        };

        self.get_pawn_moves(&mut moves);
        self.get_knight_moves(&mut moves, our_occ, their_occ);
        self.get_bishop_moves(&mut moves, our_occ, their_occ);
        self.get_rook_moves(&mut moves, our_occ, their_occ);
        self.get_queen_moves(&mut moves, our_occ, their_occ);
        self.get_king_moves(&mut moves, our_occ, their_occ);

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
        if (self.gen_sliding_moves(sq, &Self::BISHOP_DIRECTION_OFFSETS) & (bishops | queens)) != 0 {
            return true;
        }

        // Rook/Queen attacks
        if (self.gen_sliding_moves(sq, &Self::ROOK_DIRECTION_OFFSETS) & (rooks | queens)) != 0 {
            return true;
        }

        // King attacks
        if (Self::KING_ATTACKS[sq as usize] & king) != 0 {
            return true;
        }

        false
    }

    fn is_king_checked_for(&self, color: Color) -> bool {
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
            board.is_king_checked_for(board.side_to_move.flip())
        };
        if !illegal {
            nodes += perft(board, depth - 1);
        }

        // println!("Unmaking move at depth {}: {:?}", depth, m);
        board.unmake_move(m);
    }
    nodes
}
