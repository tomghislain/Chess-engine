use crate::engine::bitboards::*;
use crate::engine::board::*;
use crate::engine::types::*;

impl Board {
    fn get_white_pawn_moves(
        &self,
        moves: &mut MoveList,
        block_mask: Bitboard,
        capture_mask: Bitboard,
        pinned: Bitboard,
        pin_rays: &[Bitboard; 64],
        king_sq: Square,
    ) {
        let pawns = self.pieces[Piece::WP as usize];
        let empty = !self.occ;

        // Single pushes
        let single_push = (pawns << 8) & empty;
        for to in single_push {
            let from = Square(to.as_u8() - 8);
            let is_promotion = to.as_u8() > 55;
            let is_pinned = !(pinned & Bitboard::from(from)).is_empty();
            let is_legal = if is_pinned {
                !(Bitboard::from(to) & pin_rays[from.as_usize()] & block_mask).is_empty()
            } else {
                !(block_mask & Bitboard::from(to)).is_empty()
            };
            if is_legal {
                if is_promotion {
                    moves.push(Move::new(from, to, Piece::WN, PROMOTION_FLAG));
                    moves.push(Move::new(from, to, Piece::WB, PROMOTION_FLAG));
                    moves.push(Move::new(from, to, Piece::WR, PROMOTION_FLAG));
                    moves.push(Move::new(from, to, Piece::WQ, PROMOTION_FLAG));
                } else {
                    moves.push(Move::new(from, to, Piece::None, 0));
                }
            }
        }

        // Double pushes
        let first_push = ((pawns & RANK_2) << 8) & empty;
        let double_push = (first_push << 8) & empty;
        for to in double_push {
            let from = Square(to.as_u8() - 16);
            let is_pinned = !(pinned & Bitboard::from(from)).is_empty();
            let is_legal = if is_pinned {
                !((Bitboard::from(to)) & pin_rays[from.as_usize()] & block_mask).is_empty()
            } else {
                !(block_mask & Bitboard::from(to)).is_empty()
            };
            if is_legal {
                moves.push(Move::new(from, to, Piece::None, DOUBLE_PAWN_PUSH_FLAG))
            };
        }

        // Captures
        let captures = pawns;
        for from in captures {
            let attacks = WHITE_PAWN_ATTACKS[from.as_usize()] & self.color_occ[1];

            for to in attacks {
                let is_promotion = to.as_u8() > 55;
                let is_pinned = !(pinned & Bitboard::from(from)).is_empty();
                let is_legal = if is_pinned {
                    !((Bitboard::from(to)) & pin_rays[from.as_usize()] & capture_mask).is_empty()
                } else {
                    !(capture_mask & Bitboard::from(to)).is_empty()
                };
                if is_legal {
                    if is_promotion {
                        moves.push(Move::new(
                            from,
                            to,
                            Piece::WN,
                            CAPTURE_FLAG | PROMOTION_FLAG,
                        ));
                        moves.push(Move::new(
                            from,
                            to,
                            Piece::WB,
                            CAPTURE_FLAG | PROMOTION_FLAG,
                        ));
                        moves.push(Move::new(
                            from,
                            to,
                            Piece::WR,
                            CAPTURE_FLAG | PROMOTION_FLAG,
                        ));
                        moves.push(Move::new(
                            from,
                            to,
                            Piece::WQ,
                            CAPTURE_FLAG | PROMOTION_FLAG,
                        ));
                    } else {
                        moves.push(Move::new(from, to, Piece::None, CAPTURE_FLAG));
                    }
                }
            }
        }

        // En passant
        if self.en_passant().as_u8() < 64 {
            let en_passant_mask = Bitboard::from(self.en_passant());
            let left_en_passant = ((pawns << 7) & !FILE_H) & en_passant_mask;
            for to in left_en_passant {
                let from = Square(to.as_u8() - 7);
                let captured_sq = Square(to.as_u8() - 8);
                let changed_occ = self.occ
                    ^ (Bitboard::from(from))
                    ^ (Bitboard::from(captured_sq))
                    ^ (Bitboard::from(to));
                let mut changed_pieces = self.pieces;
                changed_pieces[Piece::WP as usize] ^= (Bitboard::from(from)) | (Bitboard::from(to));
                changed_pieces[Piece::BP as usize] ^= Bitboard::from(captured_sq);

                if !self.is_square_attacked(king_sq, Color::Black, changed_occ, &changed_pieces) {
                    moves.push(Move::new(
                        from,
                        to,
                        Piece::None,
                        CAPTURE_FLAG | EN_PASSANT_FLAG,
                    ));
                }
            }
            let right_en_passant = ((pawns << 9) & !FILE_A) & en_passant_mask;
            for to in right_en_passant {
                let from = Square(to.as_u8() - 9);
                let captured_sq = Square(to.as_u8() - 8);
                let changed_occ = self.occ
                    ^ Bitboard::from(from)
                    ^ Bitboard::from(captured_sq)
                    ^ Bitboard::from(to);
                let mut changed_pieces = self.pieces;
                changed_pieces[Piece::WP as usize] ^= (Bitboard::from(from)) | (Bitboard::from(to));
                changed_pieces[Piece::BP as usize] ^= Bitboard::from(captured_sq);

                if !self.is_square_attacked(king_sq, Color::Black, changed_occ, &changed_pieces) {
                    moves.push(Move::new(
                        from,
                        to,
                        Piece::None,
                        CAPTURE_FLAG | EN_PASSANT_FLAG,
                    ));
                }
            }
        }
    }

    fn get_black_pawn_moves(
        &self,
        moves: &mut MoveList,
        block_mask: Bitboard,
        capture_mask: Bitboard,
        pinned: Bitboard,
        pin_rays: &[Bitboard; 64],
        king_sq: Square,
    ) {
        let pawns = self.pieces[Piece::BP as usize];
        let empty = !self.occ;

        // Single pushes
        let single_push = (pawns >> 8) & empty;
        for to in single_push {
            let from = Square(to.as_u8() + 8);
            let is_promotion = to.as_u8() < 8;
            let is_pinned = !(pinned & Bitboard::from(from)).is_empty();
            let is_legal = if is_pinned {
                !(Bitboard::from(to) & pin_rays[from.as_usize()] & block_mask).is_empty()
            } else {
                !(block_mask & Bitboard::from(to)).is_empty()
            };
            if is_legal {
                if is_promotion {
                    moves.push(Move::new(from, to, Piece::BN, PROMOTION_FLAG));
                    moves.push(Move::new(from, to, Piece::BB, PROMOTION_FLAG));
                    moves.push(Move::new(from, to, Piece::BR, PROMOTION_FLAG));
                    moves.push(Move::new(from, to, Piece::BQ, PROMOTION_FLAG));
                } else {
                    moves.push(Move::new(from, to, Piece::None, 0));
                }
            }
        }

        // Double pushes
        let first_push = ((pawns & RANK_7) >> 8) & empty;
        let double_push = (first_push >> 8) & empty;
        for to in double_push {
            let from = Square(to.as_u8() + 16);
            let is_pinned = !(pinned & Bitboard::from(from)).is_empty();
            let is_legal = if is_pinned {
                !(Bitboard::from(to) & pin_rays[from.as_usize()] & block_mask).is_empty()
            } else {
                !(block_mask & Bitboard::from(to)).is_empty()
            };
            if is_legal {
                moves.push(Move::new(from, to, Piece::None, DOUBLE_PAWN_PUSH_FLAG));
            }
        }

        // Captures
        let captures = pawns;
        for from in captures {
            let attacks = BLACK_PAWN_ATTACKS[from.as_usize()] & self.color_occ[0];
            for to in attacks {
                let is_promotion = to.as_u8() < 8;
                let is_pinned = !(pinned & Bitboard::from(from)).is_empty();
                let is_legal = if is_pinned {
                    !(Bitboard::from(to) & pin_rays[from.as_usize()] & capture_mask).is_empty()
                } else {
                    !(capture_mask & Bitboard::from(to)).is_empty()
                };
                if is_legal {
                    if is_promotion {
                        moves.push(Move::new(
                            from,
                            to,
                            Piece::BN,
                            CAPTURE_FLAG | PROMOTION_FLAG,
                        ));
                        moves.push(Move::new(
                            from,
                            to,
                            Piece::BB,
                            CAPTURE_FLAG | PROMOTION_FLAG,
                        ));
                        moves.push(Move::new(
                            from,
                            to,
                            Piece::BR,
                            CAPTURE_FLAG | PROMOTION_FLAG,
                        ));
                        moves.push(Move::new(
                            from,
                            to,
                            Piece::BQ,
                            CAPTURE_FLAG | PROMOTION_FLAG,
                        ));
                    } else {
                        moves.push(Move::new(from, to, Piece::None, CAPTURE_FLAG));
                    }
                }
            }
        }

        // En passant
        if self.en_passant().as_u8() < 64 {
            let en_passant_mask = Bitboard::from(self.en_passant());
            let left_en_passant = ((pawns >> 7) & !FILE_A) & en_passant_mask;
            for to in left_en_passant {
                let from = Square(to.as_u8() + 7);
                let captured_sq = Square(to.as_u8() + 8);
                let changed_occ = self.occ
                    ^ Bitboard::from(from)
                    ^ Bitboard::from(captured_sq)
                    ^ Bitboard::from(to);

                let mut changed_pieces = self.pieces;
                changed_pieces[Piece::BP as usize] ^= (Bitboard::from(from)) | (Bitboard::from(to));
                changed_pieces[Piece::WP as usize] ^= Bitboard::from(captured_sq);

                if !self.is_square_attacked(king_sq, Color::White, changed_occ, &changed_pieces) {
                    moves.push(Move::new(
                        from,
                        to,
                        Piece::None,
                        CAPTURE_FLAG | EN_PASSANT_FLAG,
                    ));
                }
            }
            let right_en_passant = ((pawns >> 9) & !FILE_H) & en_passant_mask;
            for to in right_en_passant {
                let from = Square(to.as_u8() + 9);
                let captured_sq = Square(to.as_u8() + 8);
                let changed_occ = self.occ
                    ^ Bitboard::from(from)
                    ^ Bitboard::from(captured_sq)
                    ^ Bitboard::from(to);
                let mut changed_pieces = self.pieces;
                changed_pieces[Piece::BP as usize] ^= (Bitboard::from(from)) | (Bitboard::from(to));
                changed_pieces[Piece::WP as usize] ^= Bitboard::from(captured_sq);

                if !self.is_square_attacked(king_sq, Color::White, changed_occ, &changed_pieces) {
                    moves.push(Move::new(
                        from,
                        to,
                        Piece::None,
                        CAPTURE_FLAG | EN_PASSANT_FLAG,
                    ));
                }
            }
        }
    }

    fn get_knight_moves(
        &self,
        moves: &mut MoveList,
        knight_piece: Piece,
        block_mask: Bitboard,
        capture_mask: Bitboard,
        pinned: Bitboard,
    ) {
        let side_to_move = self.side_to_move;
        let knights = self.pieces[knight_piece as usize] & !pinned;
        for from in knights {
            let attacks = KNIGHT_ATTACKS[from.as_usize()]
                & !self.color_occ[side_to_move as usize]
                & (block_mask | capture_mask);

            for to in attacks {
                let flag = if self.piece_at(to) == Piece::None {
                    0
                } else {
                    CAPTURE_FLAG
                };

                moves.push(Move::new(from, to, Piece::None, flag));
            }
        }
    }

    fn get_rook_moves(
        &self,
        moves: &mut MoveList,
        rook_piece: Piece,
        block_mask: Bitboard,
        capture_mask: Bitboard,
        pinned: Bitboard,
        pin_rays: &[Bitboard; 64],
    ) {
        let side_to_move = self.side_to_move;
        let rooks = self.pieces[rook_piece as usize];
        for from in rooks {
            let is_pinned = !(pinned & Bitboard::from(from)).is_empty();

            let mask = ROOK_MASKS[from.as_usize()];
            let blockers = self.occ & mask;
            let idx = (blockers.wrapping_mul(ROOK_MAGIC_NUMBERS[from.as_usize()])
                >> ROOK_SHIFTS[from.as_usize()]) as usize;
            let legal_mask = if is_pinned {
                !self.color_occ[side_to_move as usize]
                    & (block_mask | capture_mask)
                    & pin_rays[from.as_usize()]
            } else {
                !self.color_occ[side_to_move as usize] & (block_mask | capture_mask)
            };
            let attacks = ROOK_ATTACKS[from.as_usize()][idx] & legal_mask;

            let remaining = attacks;
            for to in remaining {
                let flag = if self.piece_at(to) == Piece::None {
                    0
                } else {
                    CAPTURE_FLAG
                };
                moves.push(Move::new(from, to, Piece::None, flag));
            }
        }
    }

    fn get_bishop_moves(
        &self,
        moves: &mut MoveList,
        bishop_piece: Piece,
        block_mask: Bitboard,
        capture_mask: Bitboard,
        pinned: Bitboard,
        pin_rays: &[Bitboard; 64],
    ) {
        let side_to_move = self.side_to_move;
        let bishops = self.pieces[bishop_piece as usize];
        for from in bishops {
            let is_pinned = !(pinned & Bitboard::from(from)).is_empty();

            let mask = BISHOP_MASKS[from.as_usize()];
            let blockers = self.occ & mask;
            let idx = (blockers.wrapping_mul(BISHOP_MAGIC_NUMBERS[from.as_usize()])
                >> BISHOP_SHIFTS[from.as_usize()]) as usize;
            let legal_mask = if is_pinned {
                !self.color_occ[side_to_move as usize]
                    & (block_mask | capture_mask)
                    & pin_rays[from.as_usize()]
            } else {
                !self.color_occ[side_to_move as usize] & (block_mask | capture_mask)
            };
            let attacks = BISHOP_ATTACKS[from.as_usize()][idx] & legal_mask;

            let remaining = attacks;
            for to in remaining {
                let flag = if self.piece_at(to) == Piece::None {
                    0
                } else {
                    CAPTURE_FLAG
                };
                moves.push(Move::new(from, to, Piece::None, flag));
            }
        }
    }

    fn get_queen_moves(
        &self,
        moves: &mut MoveList,
        queen_piece: Piece,
        block_mask: Bitboard,
        capture_mask: Bitboard,
        pinned: Bitboard,
        pin_rays: &[Bitboard; 64],
    ) {
        let side_to_move = self.side_to_move;
        let queens = self.pieces[queen_piece as usize];
        for from in queens {
            let is_pinned = !(pinned & Bitboard::from(from)).is_empty();

            let orth_mask = ROOK_MASKS[from.as_usize()];
            let orth_blockers = self.occ & orth_mask;
            let orth_idx = (orth_blockers.wrapping_mul(ROOK_MAGIC_NUMBERS[from.as_usize()])
                >> ROOK_SHIFTS[from.as_usize()]) as usize;
            let orth_attacks = ROOK_ATTACKS[from.as_usize()][orth_idx];

            let diag_mask = BISHOP_MASKS[from.as_usize()];
            let diag_blockers = self.occ & diag_mask;
            let diag_idx = (diag_blockers.wrapping_mul(BISHOP_MAGIC_NUMBERS[from.as_usize()])
                >> BISHOP_SHIFTS[from.as_usize()]) as usize;
            let diag_attacks = BISHOP_ATTACKS[from.as_usize()][diag_idx];

            let legal_mask = if is_pinned {
                !self.color_occ[side_to_move as usize]
                    & (block_mask | capture_mask)
                    & pin_rays[from.as_usize()]
            } else {
                !self.color_occ[side_to_move as usize] & (block_mask | capture_mask)
            };

            let attacks = (orth_attacks | diag_attacks) & legal_mask;

            let remaining = attacks;
            for to in remaining {
                let flag = if self.piece_at(to) == Piece::None {
                    0
                } else {
                    CAPTURE_FLAG
                };
                moves.push(Move::new(from, to, Piece::None, flag));
            }
        }
    }

    fn compute_danger(&self, king_sq: Square) -> Bitboard {
        let mut danger = Bitboard(0);
        let (pawns, pawn_attack_table, knights, orth_pieces, diag_pieces, enemy_king) =
            match self.side_to_move {
                Color::White => (
                    self.pieces[Piece::BP as usize],
                    BLACK_PAWN_ATTACKS,
                    self.pieces[Piece::BN as usize],
                    self.pieces[Piece::BR as usize] | self.pieces[Piece::BQ as usize],
                    self.pieces[Piece::BB as usize] | self.pieces[Piece::BQ as usize],
                    self.pieces[Piece::BK as usize],
                ),
                Color::Black => (
                    self.pieces[Piece::WP as usize],
                    WHITE_PAWN_ATTACKS,
                    self.pieces[Piece::WN as usize],
                    self.pieces[Piece::WR as usize] | self.pieces[Piece::WQ as usize],
                    self.pieces[Piece::WB as usize] | self.pieces[Piece::WQ as usize],
                    self.pieces[Piece::WK as usize],
                ),
            };
        let kingless_occ = self.occ & !Bitboard::from(king_sq);
        for sq in pawns {
            danger |= pawn_attack_table[sq.as_usize()];
        }
        for sq in knights {
            danger |= KNIGHT_ATTACKS[sq.as_usize()];
        }
        for sq in diag_pieces {
            let mask = BISHOP_MASKS[sq.as_usize()];
            let blockers = kingless_occ & mask;
            let idx = (blockers.wrapping_mul(BISHOP_MAGIC_NUMBERS[sq.as_usize()])
                >> BISHOP_SHIFTS[sq.as_usize()]) as usize;
            danger |= BISHOP_ATTACKS[sq.as_usize()][idx];
        }
        for sq in orth_pieces {
            let mask = ROOK_MASKS[sq.as_usize()];
            let blockers = kingless_occ & mask;
            let idx = (blockers.wrapping_mul(ROOK_MAGIC_NUMBERS[sq.as_usize()])
                >> ROOK_SHIFTS[sq.as_usize()]) as usize;
            danger |= ROOK_ATTACKS[sq.as_usize()][idx];
        }
        danger |= KING_ATTACKS[enemy_king.trailing_zeros() as usize];

        danger
    }

    fn get_king_moves(&self, moves: &mut MoveList, king_sq: Square, danger_mask: Bitboard) {
        let side_to_move = self.side_to_move;
        let attacks = KING_ATTACKS[king_sq.as_usize()]
            & !self.color_occ[side_to_move as usize]
            & !danger_mask;
        for to in attacks {
            let flag = if self.piece_at(to) == Piece::None {
                0
            } else {
                CAPTURE_FLAG
            };

            moves.push(Move::new(king_sq, to, Piece::None, flag));
        }
    }

    pub fn compute_checkers(
        &self,
        king_sq: Square,
        occ: Bitboard,
        attacker_color: Color,
        pieces: &[Bitboard; 12],
    ) -> Bitboard {
        let mut checkers = Bitboard(0);
        let (pawns, pawn_attack_table, knights, orth_pieces, diag_pieces) = match attacker_color {
            Color::Black => (
                pieces[Piece::BP as usize],
                WHITE_PAWN_ATTACKS,
                pieces[Piece::BN as usize],
                pieces[Piece::BR as usize] | pieces[Piece::BQ as usize],
                pieces[Piece::BB as usize] | pieces[Piece::BQ as usize],
            ),
            Color::White => (
                pieces[Piece::WP as usize],
                BLACK_PAWN_ATTACKS,
                pieces[Piece::WN as usize],
                pieces[Piece::WR as usize] | pieces[Piece::WQ as usize],
                pieces[Piece::WB as usize] | pieces[Piece::WQ as usize],
            ),
        };
        let pawn_attacks = pawn_attack_table[king_sq.as_usize()];
        checkers |= pawn_attacks & pawns;

        let knight_attacks = KNIGHT_ATTACKS[king_sq.as_usize()];
        checkers |= knight_attacks & knights;

        let diag_mask = BISHOP_MASKS[king_sq.as_usize()];
        let diag_blockers = diag_mask & occ;
        let diag_idx = (diag_blockers.wrapping_mul(BISHOP_MAGIC_NUMBERS[king_sq.as_usize()])
            >> BISHOP_SHIFTS[king_sq.as_usize()]) as usize;
        let diag_attacks = BISHOP_ATTACKS[king_sq.as_usize()][diag_idx];
        checkers |= diag_attacks & diag_pieces;

        let orth_mask = ROOK_MASKS[king_sq.as_usize()];
        let orth_blockers = orth_mask & occ;
        let orth_idx = (orth_blockers.wrapping_mul(ROOK_MAGIC_NUMBERS[king_sq.as_usize()])
            >> ROOK_SHIFTS[king_sq.as_usize()]) as usize;
        let orth_attacks = ROOK_ATTACKS[king_sq.as_usize()][orth_idx];
        checkers |= orth_attacks & orth_pieces;

        checkers
    }

    fn compute_pins(&self, king_sq: Square) -> (Bitboard, [Bitboard; 64]) {
        let mut pinned = Bitboard(0);
        let mut pin_rays = [Bitboard(0); 64];

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

        let potential_diag_attackers = BISHOP_ATTACKS[king_sq.as_usize()][0] & diag_pieces;
        let potential_orth_attackers = ROOK_ATTACKS[king_sq.as_usize()][0] & orth_pieces;

        let attackers = potential_diag_attackers | potential_orth_attackers;

        for attacker_sq in attackers {
            let between = BETWEEN[king_sq.as_usize()][attacker_sq.as_usize()];
            let blockers = between & self.occ;
            let our_blockers = between & self.color_occ[self.side_to_move as usize];

            if blockers.count_ones() == 1 && !our_blockers.is_empty() {
                let pinned_sq = blockers.trailing_zeros() as usize;
                pinned |= blockers;
                pin_rays[pinned_sq] = between | Bitboard::from(attacker_sq);
            }
        }

        (pinned, pin_rays)
    }

    fn compute_block_mask(&self, king_sq: Square, checkers: Bitboard) -> Bitboard {
        let checker_sq = Square(checkers.trailing_zeros());
        let checker_piece = self.piece_at(checker_sq);

        match checker_piece {
            Piece::BN | Piece::WN | Piece::BP | Piece::WP => Bitboard(0),
            Piece::BR | Piece::WR | Piece::BB | Piece::WB | Piece::BQ | Piece::WQ => {
                BETWEEN[king_sq.as_usize()][checker_sq.as_usize()]
            }
            _ => Bitboard(0),
        }
    }

    fn gen_castling_moves(&self, moves: &mut MoveList, king_danger_squares: Bitboard) {
        let castling_rights = self.castling();
        match self.side_to_move {
            Color::White => {
                if castling_rights & WK_CASTLE != 0
                    && (self.occ & (Bitboard::from(Square(5)) | Bitboard::from(Square(6))))
                        .is_empty()
                    && (king_danger_squares
                        & (Bitboard::from(Square(4))
                            | Bitboard::from(Square(5))
                            | Bitboard::from(Square(6))))
                    .is_empty()
                {
                    moves.push(Move::new(Square(4), Square(6), Piece::None, CASTLING_FLAG));
                }
                if castling_rights & WQ_CASTLE != 0
                    && (self.occ
                        & (Bitboard::from(Square(1))
                            | Bitboard::from(Square(2))
                            | Bitboard::from(Square(3))))
                    .is_empty()
                    && (king_danger_squares
                        & (Bitboard::from(Square(4))
                            | Bitboard::from(Square(3))
                            | Bitboard::from(Square(2))))
                    .is_empty()
                {
                    moves.push(Move::new(Square(4), Square(2), Piece::None, CASTLING_FLAG));
                }
            }
            Color::Black => {
                if castling_rights & BK_CASTLE != 0
                    && (self.occ & (Bitboard::from(Square(61)) | Bitboard::from(Square(62))))
                        .is_empty()
                    && (king_danger_squares
                        & (Bitboard::from(Square(60))
                            | Bitboard::from(Square(61))
                            | Bitboard::from(Square(62))))
                    .is_empty()
                {
                    moves.push(Move::new(
                        Square(60),
                        Square(62),
                        Piece::None,
                        CASTLING_FLAG,
                    ));
                }
                if castling_rights & BQ_CASTLE != 0
                    && (self.occ
                        & (Bitboard::from(Square(57))
                            | Bitboard::from(Square(58))
                            | Bitboard::from(Square(59))))
                    .is_empty()
                    && (king_danger_squares
                        & (Bitboard::from(Square(60))
                            | Bitboard::from(Square(59))
                            | Bitboard::from(Square(58))))
                    .is_empty()
                {
                    moves.push(Move::new(
                        Square(60),
                        Square(58),
                        Piece::None,
                        CASTLING_FLAG,
                    ));
                }
            }
        };
    }

    fn get_non_king_moves(
        &self,
        moves: &mut MoveList,
        block_mask: Bitboard,
        capture_mask: Bitboard,
        pinned: Bitboard,
        pin_rays: [Bitboard; 64],
        king_sq: Square,
    ) {
        match self.side_to_move {
            Color::White => {
                self.get_white_pawn_moves(
                    moves,
                    block_mask,
                    capture_mask,
                    pinned,
                    &pin_rays,
                    king_sq,
                );
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
                self.get_black_pawn_moves(
                    moves,
                    block_mask,
                    capture_mask,
                    pinned,
                    &pin_rays,
                    king_sq,
                );
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

    pub fn get_legal_moves(&self, moves: &mut MoveList) {
        moves.clear();
        let side_to_move = self.side_to_move;

        let king_sq = Square(
            self.pieces[if side_to_move == Color::White {
                Piece::WK
            } else {
                Piece::BK
            } as usize]
                .trailing_zeros(),
        );
        debug_assert!(king_sq.as_u8() <= 64);

        let king_danger_mask = self.compute_danger(king_sq); // Squares the king cannot move to, i.e attacked squares if the king did not exist

        self.get_king_moves(moves, king_sq, king_danger_mask);
        let checkers: Bitboard =
            self.compute_checkers(king_sq, self.occ, side_to_move.flip(), &self.pieces); // Bitmap of pieces that put the king in check

        if checkers.count_ones() > 1 {
            return; // If it is a double check, only king moves can be legal
        }

        // pinned: bitmap of pieces that cannot move because they would put the king in check
        // pin_rays: list of bitmaps for each square a piece could be on, that show where that piece can move without puting the king in check
        let (pinned, pin_rays) = self.compute_pins(king_sq);

        let in_check = !checkers.is_empty();
        // block_mask: bitmap of squares a piece can go to in order to stop the check (if the checker is a sliding piece, otherwise no squares can block it without capturing)
        let (block_mask, capture_mask) = if in_check {
            (self.compute_block_mask(king_sq, checkers), checkers)
        } else {
            self.gen_castling_moves(moves, king_danger_mask); // Castling moves can only happen when not in check
            (!Bitboard(0), !Bitboard(0)) // All squares are valid to move to and capture on if not check are in place (except for pins)
        };

        self.get_non_king_moves(moves, block_mask, capture_mask, pinned, pin_rays, king_sq); // Careful for ep
    }

    pub fn is_square_attacked(
        &self,
        sq: Square,
        attacker: Color,
        occ: Bitboard,
        pieces: &[Bitboard; 12],
    ) -> bool {
        let sq_usize = sq.as_usize();
        let (pawn, pawn_table, knight, bishop, rook, queen, king) = match attacker {
            Color::White => (
                Piece::WP,
                BLACK_PAWN_ATTACKS,
                Piece::WN,
                Piece::WB,
                Piece::WR,
                Piece::WQ,
                Piece::WK,
            ),
            Color::Black => (
                Piece::BP,
                WHITE_PAWN_ATTACKS,
                Piece::BN,
                Piece::BB,
                Piece::BR,
                Piece::BQ,
                Piece::BK,
            ),
        };

        if !(pawn_table[sq_usize] & pieces[pawn as usize]).is_empty() {
            return true;
        }

        if !(KNIGHT_ATTACKS[sq_usize] & pieces[knight as usize]).is_empty() {
            return true;
        }

        let diag_attackers = pieces[bishop as usize] | pieces[queen as usize];
        if !diag_attackers.is_empty() {
            let diag_mask = BISHOP_MASKS[sq_usize];
            let diag_blockers = occ & diag_mask;
            let diag_idx = (diag_blockers.wrapping_mul(BISHOP_MAGIC_NUMBERS[sq_usize])
                >> BISHOP_SHIFTS[sq_usize]) as usize;
            let diag_attacks = BISHOP_ATTACKS[sq_usize][diag_idx];
            if !(diag_attacks & diag_attackers).is_empty() {
                return true;
            }
        }

        let orth_attackers = pieces[rook as usize] | pieces[queen as usize];
        if !orth_attackers.is_empty() {
            let orth_mask = ROOK_MASKS[sq_usize];
            let orth_blockers = occ & orth_mask;
            let orth_idx = (orth_blockers.wrapping_mul(ROOK_MAGIC_NUMBERS[sq_usize])
                >> ROOK_SHIFTS[sq_usize]) as usize;
            let orth_attacks = ROOK_ATTACKS[sq_usize][orth_idx];
            if !(orth_attacks & orth_attackers).is_empty() {
                return true;
            }
        }

        if !(KING_ATTACKS[sq_usize] & pieces[king as usize]).is_empty() {
            return true;
        }

        false
    }
}
