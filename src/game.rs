pub const FILE_A: u64 = 0x0101010101010101;
pub const FILE_B: u64 = 0x0202020202020202;
pub const FILE_G: u64 = 0x4040404040404040;
pub const FILE_H: u64 = 0x8080808080808080;

#[derive(Copy, Clone, PartialEq)]
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

#[derive(Copy, Clone)]
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

impl Undo {
    pub fn new(board: &Board, mv: &Move) -> Self {
        // TODO: inline function in make_move to avoid captured piece recomputation
        Undo {
            castling: board.castling,
            en_passant: board.en_passant,
            halfmove_clock: board.halfmove_clock,
            captured_piece: board.piece_at(mv.to), //TODO: check for en passant
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
        self.pieces[p as usize] |= mask;
    }

    // Sets the bit to 1 at index sq of the bitmap at index p of self.pieces
    fn add_piece(&mut self, p: Piece, sq: u8) {
        let mask = 1 << sq;
        self.pieces[p as usize] &= !mask;
    }

    // Flips the bits at the from and to index in the bitmap at index p of self.pieces
    fn move_piece(&mut self, p: Piece, from: u8, to: u8) {
        let mask = 1 << from | 1 << to;
        self.pieces[p as usize] ^= mask;
    }

    // Plays a move by mutating bitboards and state information, and pushes an undo entry on the stack
    pub fn make_move(&mut self, mv: Move) {
        let undo = Undo::new(&self, &mv);
        self.undo_stack.push(undo);

        // TODO: Update zobrist hash

        // Remove piece at mv.to if capture/behin mv.to if en passant
        let is_capture = (mv.flags & CAPTURE_FLAG) != 0;
        if is_capture {
            if (mv.flags & EN_PASSANT_FLAG) != 0 {
                let (capture_sq, captured_piece) = if self.side_to_move == Color::White {
                    //TODO: replace if/else with match
                    (mv.to - 8, Piece::BP)
                } else {
                    (mv.to + 8, Piece::WP)
                };
                assert!(Some(captured_piece) == self.piece_at(capture_sq)); //TODO: remove check
                self.remove_piece(captured_piece, capture_sq);
            } else {
                self.remove_piece(self.piece_at(mv.to).unwrap(), mv.to);
            }
        }
        // Move piece from mv.from to mv.to (checking for promotion and castling)
        let moved_piece = self.piece_at(mv.from).unwrap();
        if (mv.flags & PROMOTION_FLAG) != 0 {
            self.remove_piece(moved_piece, mv.from);
            self.add_piece(mv.promotion.unwrap(), mv.to);
        } else {
            self.move_piece(moved_piece, mv.from, mv.to);
            if (mv.flags & CASTLING_FLAG) != 0 {
                match mv.to {
                    62 => self.move_piece(Piece::WR, 63, 61), // white king side
                    58 => self.move_piece(Piece::WR, 56, 59), // white queen side
                    6 => self.move_piece(Piece::BR, 7, 5),    // black king side
                    2 => self.move_piece(Piece::BR, 0, 3),    // black queen side
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

        let is_pawn_move = match moved_piece {
            Piece::WP => true,
            Piece::BP => true,
            _ => false,
        };
        let file = mv.from % 8;
        self.en_passant = if is_pawn_move && ((mv.from as i16 - mv.to as i16).abs() == 16) {
            1 << file
        } else {
            0
        };

        match moved_piece {
            Piece::WK => self.castling &= !(WK_CASTLE | WQ_CASTLE),
            Piece::BK => self.castling &= !(BK_CASTLE | BQ_CASTLE),
            Piece::WR => {
                if mv.from == 63 {
                    self.castling &= !WK_CASTLE
                }
                if mv.from == 56 {
                    self.castling &= !WQ_CASTLE
                }
            }
            Piece::BR => {
                if mv.from == 7 {
                    self.castling &= !BK_CASTLE
                }
                if mv.from == 0 {
                    self.castling &= !BQ_CASTLE
                }
            }
            _ => unreachable!(),
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
            moved_piece = match promoted_piece {
                // TODO: compute moved_piece with side_to_move (make sure of when side_to_move is flipped)
                Piece::WN | Piece::WB | Piece::WR | Piece::WQ => Piece::WP,
                Piece::BN | Piece::BB | Piece::BR | Piece::BQ => Piece::BP,
                _ => unreachable!(),
            };
            self.add_piece(moved_piece, mv.from);
        } else {
            moved_piece = self.piece_at(mv.to).unwrap();
            self.move_piece(moved_piece, mv.to, mv.from);
            if (mv.flags & CASTLING_FLAG) != 0 {
                match mv.to {
                    62 => self.move_piece(Piece::WR, 61, 63), // white king side
                    58 => self.move_piece(Piece::WR, 59, 56), // white queen side
                    6 => self.move_piece(Piece::BR, 5, 7),    // black king side
                    2 => self.move_piece(Piece::BR, 3, 0),    // black queen side
                    _ => unreachable!(),
                }
            }
        }
        // In case of capture (and maybe en passant), restore the taken piece
        if (mv.flags & CAPTURE_FLAG) != 0 {
            if (mv.flags & EN_PASSANT_FLAG) != 0 {
                let (capture_sq, captured_piece) = if moved_piece == Piece::WP {
                    //TODO: replace if/else with match, use undo.captured_piece
                    (mv.to - 8, Piece::BP)
                } else {
                    (mv.to + 8, Piece::WP)
                };
                assert!(Some(captured_piece) == self.piece_at(capture_sq)); //TODO: remove check
                self.add_piece(captured_piece, capture_sq);
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

    fn get_white_pawn_moves(&self, moves: &mut Vec<Move>, their_occ: u64) {
        let pawns = self.pieces[Piece::WP as usize];
        let empty = !self.occ;

        // Single pushes
        let mut single_push = (pawns << 8) & empty;
        while single_push != 0 {
            let to = single_push.trailing_zeros() as u8;
            single_push &= single_push - 1;
            let from = to - 8;
            moves.push(Move::new(from, to, 0, None)); //TODO: handle promotions
        }

        // Double pushes
        let rank2: u64 = 0x000000000000FF00;
        let mut double_push = ((pawns & rank2) << 8) & empty;
        double_push = (double_push << 8) & empty;
        while double_push != 0 {
            let to = double_push.trailing_zeros() as u8;
            double_push &= double_push - 1;
            let from = to - 16;
            moves.push(Move::new(from, to, 0, None));
        }

        // Captures
        let mut left_captures = ((pawns << 7) & !FILE_H) & their_occ;
        while left_captures != 0 {
            let to = left_captures.trailing_zeros() as u8;
            left_captures &= left_captures - 1;
            let from = to - 7;
            moves.push(Move::new(from, to, CAPTURE_FLAG, None)); //TODO: handle promotions
        }
        let mut right_captures = ((pawns << 9) & !FILE_A) & their_occ;
        while right_captures != 0 {
            let to = right_captures.trailing_zeros() as u8;
            right_captures &= right_captures - 1;
            let from = to - 9;
            moves.push(Move::new(from, to, CAPTURE_FLAG, None)); //TODO: handle promotions
        }

        // En passant
        let en_passant_mask = (self.en_passant as u64) << 40;
        if en_passant_mask != 0 {
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

    fn get_black_pawn_moves(&self, moves: &mut Vec<Move>, their_occ: u64) {
        let pawns = self.pieces[Piece::BP as usize];
        let empty = !self.occ;

        // Single pushes
        let mut single_push = (pawns >> 8) & empty;
        while single_push != 0 {
            let to = single_push.trailing_zeros() as u8;
            single_push &= single_push - 1;
            let from = to + 8;
            moves.push(Move::new(from, to, 0, None)); //TODO: handle promotions
        }

        // Double pushes
        let rank2: u64 = 0x000000000000FF00;
        let mut double_push = ((pawns & rank2) >> 8) & empty;
        double_push = (double_push >> 8) & empty;
        while double_push != 0 {
            let to = double_push.trailing_zeros() as u8;
            double_push &= double_push - 1;
            let from = to + 16;
            moves.push(Move::new(from, to, 0, None));
        }

        // Captures
        let mut left_captures = ((pawns >> 7) & !FILE_A) & their_occ;
        while left_captures != 0 {
            let to = left_captures.trailing_zeros() as u8;
            left_captures &= left_captures - 1;
            let from = to + 7;
            moves.push(Move::new(from, to, CAPTURE_FLAG, None)); //TODO: handle promotions
        }
        let mut right_captures = ((pawns >> 9) & !FILE_H) & their_occ;
        while right_captures != 0 {
            let to = right_captures.trailing_zeros() as u8;
            right_captures &= right_captures - 1;
            let from = to + 9;
            moves.push(Move::new(from, to, CAPTURE_FLAG, None)); //TODO: handle promotions
        }

        // En passant
        let en_passant_mask = (self.en_passant as u64) << 16;
        if en_passant_mask != 0 {
            let mut left_en_passant = ((pawns >> 7) & !FILE_H) & en_passant_mask;
            while left_en_passant != 0 {
                let to = left_en_passant.trailing_zeros() as u8;
                left_en_passant &= left_en_passant - 1;
                let from = to + 7;
                moves.push(Move::new(from, to, CAPTURE_FLAG | EN_PASSANT_FLAG, None));
            }
            let mut right_en_passant = ((pawns >> 9) & !FILE_A) & en_passant_mask;
            while right_en_passant != 0 {
                let to = right_en_passant.trailing_zeros() as u8;
                right_en_passant &= right_en_passant - 1;
                let from = to + 9;
                moves.push(Move::new(from, to, CAPTURE_FLAG | EN_PASSANT_FLAG, None));
            }
        }
    }

    fn get_pawn_moves(&self, moves: &mut Vec<Move>, their_occ: u64) {
        match self.side_to_move {
            Color::White => self.get_white_pawn_moves(moves, their_occ),
            Color::Black => self.get_black_pawn_moves(moves, their_occ),
        }
    }

    const fn gen_knight_attacks() -> [u64; 64] {
        let mut table = [0u64; 64];
        let mut sq = 0;

        while sq < 64 {
            let bb = 1u64 << sq;

            let attacks = ((bb << 17) & !FILE_A)
                | ((bb << 15) & !FILE_H)
                | ((bb << 10) & !FILE_A & !FILE_B)
                | ((bb << 6) & !FILE_H & !FILE_G)
                | ((bb >> 17) & !FILE_H)
                | ((bb >> 15) & !FILE_A)
                | ((bb >> 10) & !FILE_H & !FILE_G)
                | ((bb >> 6) & !FILE_A & !FILE_B);

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

            let mut attacks = Self::KNIGHT_ATTACKS[from as usize];
            attacks &= our_occ;

            while attacks != 0 {
                let to = attacks.trailing_zeros() as u8;
                attacks &= attacks - 1;

                let is_capture = (1u64 << to) & their_occ != 0;

                moves.push(Move::new(from, to, is_capture as u8, None));
            }
        }
    }

    fn get_pseudo_legal_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();

        let (our_occ, their_occ) = match self.side_to_move {
            Color::White => (self.white_occ, self.black_occ),
            Color::Black => (self.black_occ, self.white_occ),
        };

        self.get_pawn_moves(&mut moves, their_occ);
        self.get_knight_moves(&mut moves, our_occ, their_occ);
        self.get_bishop_moves(&mut moves, our_occ, their_occ);
        self.get_rook_moves(&mut moves, our_occ, their_occ);
        self.get_queen_moves(&mut moves, our_occ, their_occ);
        self.get_king_moves(&mut moves, our_occ, their_occ);

        moves
    } //TODO
}

fn perft(board: &mut Board, depth: u8) -> u64 {
    if depth == 0 {
        return 1;
    }

    let mut nodes = 0;
    let moves = get_pseudo_legal_moves(board);

    for m in moves {
        board.make_move(m);
        if !is_king_checked() {
            nodes += perft(board, depth - 1);
        }
        board.unmake_move(m);
    }
    nodes
}
