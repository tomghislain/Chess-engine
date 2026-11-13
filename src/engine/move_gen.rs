use crate::engine::bitboards::{
    BETWEEN, BISHOP_ATTACKS, BISHOP_OFFSETS, BLACK_PAWN_ATTACKS, FILE_A, FILE_H, KING_ATTACKS,
    KNIGHT_ATTACKS, RANK_2, RANK_7, ROOK_ATTACKS, ROOK_OFFSETS, WHITE_PAWN_ATTACKS, bishop_attacks,
    rook_attacks,
};
use crate::engine::board::{BK_CASTLE, BQ_CASTLE, Board, WK_CASTLE, WQ_CASTLE};
use crate::engine::types::{
    Bitboard, CAPTURE_FLAG, CASTLING_FLAG, Color, DOUBLE_PAWN_PUSH_FLAG, EN_PASSANT_FLAG, Move,
    MoveList, PROMOTION_FLAG, Piece, Square,
};

impl Board {
    #[allow(clippy::too_many_lines)]
    fn get_pawn_moves<const IS_WHITE: bool>(
        &self,
        f: &mut impl FnMut(Move),
        move_mask: Bitboard,
        pinned: Bitboard,
        pin_rays: &[Bitboard; 64],
        king_sq: Square,
    ) {
        let pawns = if IS_WHITE {
            self.pieces[Piece::WP as usize]
        } else {
            self.pieces[Piece::BP as usize]
        };
        let promotion_pieces = if IS_WHITE {
            [Piece::WN, Piece::WB, Piece::WR, Piece::WQ]
        } else {
            [Piece::BN, Piece::BB, Piece::BR, Piece::BQ]
        };
        let empty = !self.occ;
        let promotion_pawns = if IS_WHITE {
            pawns & RANK_7
        } else {
            pawns & RANK_2
        };
        let non_promotion_pawns = if IS_WHITE {
            pawns & !RANK_7
        } else {
            pawns & !RANK_2
        };

        // Non-promotions
        let free_pawns = non_promotion_pawns & !pinned;
        let pinned_pawns = non_promotion_pawns & pinned;
        // Free
        let single_push = if IS_WHITE {
            (free_pawns << 8) & empty & move_mask
        } else {
            (free_pawns >> 8) & empty & move_mask
        };
        for to in single_push {
            let from = if IS_WHITE {
                Square(to.as_u8() - 8)
            } else {
                Square(to.as_u8() + 8)
            };
            f(Move::new(from, to, Piece::None, 0));
        }
        let double_push = if IS_WHITE {
            ((((free_pawns & RANK_2) << 8) & empty) << 8) & empty & move_mask
        } else {
            ((((free_pawns & RANK_7) >> 8) & empty) >> 8) & empty & move_mask
        };
        for to in double_push {
            let from = if IS_WHITE {
                Square(to.as_u8() - 16)
            } else {
                Square(to.as_u8() + 16)
            };
            f(Move::new(from, to, Piece::None, DOUBLE_PAWN_PUSH_FLAG));
        }
        let enemy_occ = if IS_WHITE {
            self.color_occ[Color::Black as usize]
        } else {
            self.color_occ[Color::White as usize]
        };

        let left_captures = if IS_WHITE {
            (free_pawns << 7) & !FILE_H & enemy_occ & move_mask
        } else {
            (free_pawns >> 7) & !FILE_A & enemy_occ & move_mask
        };
        for to in left_captures {
            let from = if IS_WHITE {
                Square(to.as_u8() - 7)
            } else {
                Square(to.as_u8() + 7)
            };
            f(Move::new(from, to, Piece::None, CAPTURE_FLAG));
        }

        let right_captures = if IS_WHITE {
            (free_pawns << 9) & !FILE_A & enemy_occ & move_mask
        } else {
            (free_pawns >> 9) & !FILE_H & enemy_occ & move_mask
        };
        for to in right_captures {
            let from = if IS_WHITE {
                Square(to.as_u8() - 9)
            } else {
                Square(to.as_u8() + 9)
            };
            f(Move::new(from, to, Piece::None, CAPTURE_FLAG));
        }
        // Pinned
        for from in pinned_pawns {
            let ray = pin_rays[from.as_usize()];
            let from_bb = Bitboard::from(from);
            let single_push = if IS_WHITE {
                (from_bb << 8) & empty & move_mask & ray
            } else {
                (from_bb >> 8) & empty & move_mask & ray
            };
            for to in single_push {
                f(Move::new(from, to, Piece::None, 0));
            }
            let double_push = if IS_WHITE {
                ((((from_bb & RANK_2) << 8) & empty) << 8) & empty & move_mask & ray
            } else {
                ((((from_bb & RANK_7) >> 8) & empty) >> 8) & empty & move_mask & ray
            };
            for to in double_push {
                f(Move::new(from, to, Piece::None, DOUBLE_PAWN_PUSH_FLAG));
            }
            let attacks = if IS_WHITE {
                WHITE_PAWN_ATTACKS[from.as_usize()]
                    & self.color_occ[Color::Black as usize]
                    & move_mask
                    & ray
            } else {
                BLACK_PAWN_ATTACKS[from.as_usize()]
                    & self.color_occ[Color::White as usize]
                    & move_mask
                    & ray
            };
            for to in attacks {
                f(Move::new(from, to, Piece::None, CAPTURE_FLAG));
            }
        }

        // Promotions
        let free_pawns = promotion_pawns & !pinned;
        let pinned_pawns = promotion_pawns & pinned;
        // Free
        let single_push = if IS_WHITE {
            (free_pawns << 8) & empty & move_mask
        } else {
            (free_pawns >> 8) & empty & move_mask
        };
        for to in single_push {
            let from = if IS_WHITE {
                Square(to.as_u8() - 8)
            } else {
                Square(to.as_u8() + 8)
            };
            for piece in promotion_pieces {
                f(Move::new(from, to, piece, PROMOTION_FLAG));
            }
        }
        let left_captures = if IS_WHITE {
            (free_pawns << 7) & !FILE_H & enemy_occ & move_mask
        } else {
            (free_pawns >> 7) & !FILE_A & enemy_occ & move_mask
        };
        for to in left_captures {
            let from = if IS_WHITE {
                Square(to.as_u8() - 7)
            } else {
                Square(to.as_u8() + 7)
            };
            for piece in promotion_pieces {
                f(Move::new(from, to, piece, PROMOTION_FLAG | CAPTURE_FLAG));
            }
        }

        let right_captures = if IS_WHITE {
            (free_pawns << 9) & !FILE_A & enemy_occ & move_mask
        } else {
            (free_pawns >> 9) & !FILE_H & enemy_occ & move_mask
        };
        for to in right_captures {
            let from = if IS_WHITE {
                Square(to.as_u8() - 9)
            } else {
                Square(to.as_u8() + 9)
            };
            for piece in promotion_pieces {
                f(Move::new(from, to, piece, PROMOTION_FLAG | CAPTURE_FLAG));
            }
        }
        // Pinned
        for from in pinned_pawns {
            let ray = pin_rays[from.as_usize()];
            let attacks = if IS_WHITE {
                WHITE_PAWN_ATTACKS[from.as_usize()]
                    & self.color_occ[Color::Black as usize]
                    & move_mask
                    & ray
            } else {
                BLACK_PAWN_ATTACKS[from.as_usize()]
                    & self.color_occ[Color::White as usize]
                    & move_mask
                    & ray
            };
            for to in attacks {
                for piece in promotion_pieces {
                    f(Move::new(from, to, piece, PROMOTION_FLAG | CAPTURE_FLAG));
                }
            }
        }

        // En passant
        if self.en_passant != Square::NO_SQUARE {
            let to = self.en_passant;
            let en_passant_mask = Bitboard::from(to);
            let captured_mask = if IS_WHITE {
                en_passant_mask >> 8
            } else {
                en_passant_mask << 8
            };
            let attacks = if IS_WHITE {
                (WHITE_PAWN_ATTACKS[to.as_usize()] >> 16) & pawns
            } else {
                (BLACK_PAWN_ATTACKS[to.as_usize()] << 16) & pawns
            }; // Pawn attack table reuse for cache locality
            for from in attacks {
                let changed_occ =
                    self.occ ^ (Bitboard::from(from)) ^ (captured_mask) ^ (en_passant_mask);
                let changed_pawns = if IS_WHITE {
                    self.pieces[Piece::BP as usize] ^ captured_mask
                } else {
                    self.pieces[Piece::WP as usize] ^ captured_mask
                };
                if !self.would_en_passant_check(
                    king_sq,
                    if IS_WHITE { Color::Black } else { Color::White },
                    changed_occ,
                    changed_pawns,
                ) {
                    f(Move::new(
                        from,
                        to,
                        Piece::None,
                        CAPTURE_FLAG | EN_PASSANT_FLAG,
                    ));
                }
            }
        }
    }

    fn would_en_passant_check(
        &self,
        sq: Square,
        attacker: Color,
        changed_occ: Bitboard,
        attacker_pawns: Bitboard,
    ) -> bool {
        let sq_usize = sq.as_usize();
        let (pawn_table, attacker_bishops, attacker_rooks, attacker_queens) = match attacker {
            Color::White => (
                BLACK_PAWN_ATTACKS,
                self.pieces[Piece::WB as usize],
                self.pieces[Piece::WR as usize],
                self.pieces[Piece::WQ as usize],
            ),
            Color::Black => (
                WHITE_PAWN_ATTACKS,
                self.pieces[Piece::BB as usize],
                self.pieces[Piece::BR as usize],
                self.pieces[Piece::BQ as usize],
            ),
        };

        if !(pawn_table[sq_usize] & attacker_pawns).is_empty() {
            return true;
        }

        let diag_attackers = attacker_bishops | attacker_queens;
        if !diag_attackers.is_empty() {
            let diag_attacks = bishop_attacks(sq_usize, changed_occ);
            if !(diag_attacks & diag_attackers).is_empty() {
                return true;
            }
        }

        let orth_attackers = attacker_rooks | attacker_queens;
        if !orth_attackers.is_empty() {
            let orth_attacks = rook_attacks(sq_usize, changed_occ);
            if !(orth_attacks & orth_attackers).is_empty() {
                return true;
            }
        }

        false
    }

    fn get_knight_moves(
        &self,
        f: &mut impl FnMut(Move),
        knights: Bitboard,
        move_mask: Bitboard,
        pinned: Bitboard,
    ) {
        let side_to_move = self.side_to_move;
        let free_knights = knights & !pinned;
        for from in free_knights {
            let attacks = KNIGHT_ATTACKS[from.as_usize()]
                & !self.color_occ[side_to_move as usize]
                & move_mask;

            for to in attacks {
                let flag = if self.piece_at(to) == Piece::None {
                    0
                } else {
                    CAPTURE_FLAG
                };

                f(Move::new(from, to, Piece::None, flag));
            }
        }
    }

    fn get_sliding_moves<const IS_ROOK: bool, const IS_BISHOP: bool>(
        &self,
        f: &mut impl FnMut(Move),
        pieces: Bitboard,
        move_mask: Bitboard,
        pinned: Bitboard,
        pin_rays: &[Bitboard; 64],
    ) {
        let free_pieces = pieces & !pinned;
        let pinned_pieces = pieces & pinned;
        let base_mask = !self.color_occ[self.side_to_move as usize] & move_mask;
        for from in free_pieces {
            let mut attacks = Bitboard(0);
            if IS_ROOK {
                attacks |= rook_attacks(from.as_usize(), self.occ);
            }
            if IS_BISHOP {
                attacks |= bishop_attacks(from.as_usize(), self.occ);
            }
            attacks &= base_mask;

            for to in attacks {
                let flag = if self.piece_at(to) == Piece::None {
                    0
                } else {
                    CAPTURE_FLAG
                };
                f(Move::new(from, to, Piece::None, flag));
            }
        }
        for from in pinned_pieces {
            let mut attacks = Bitboard(0);
            if IS_ROOK {
                attacks |= rook_attacks(from.as_usize(), self.occ);
            }
            if IS_BISHOP {
                attacks |= bishop_attacks(from.as_usize(), self.occ);
            }
            attacks &= base_mask & pin_rays[from.as_usize()];

            for to in attacks {
                let flag = if self.piece_at(to) == Piece::None {
                    0
                } else {
                    CAPTURE_FLAG
                };
                f(Move::new(from, to, Piece::None, flag));
            }
        }
    }

    fn compute_danger(&self, king_sq: Square) -> Bitboard {
        let mut danger = Bitboard(0);
        let (pawn_attacks, knights, orth_pieces, diag_pieces, enemy_king) = match self.side_to_move
        {
            Color::White => (
                {
                    let pawns = self.pieces[Piece::BP as usize];
                    ((pawns >> 7) & !FILE_A) | ((pawns >> 9) & !FILE_H)
                },
                self.pieces[Piece::BN as usize],
                self.pieces[Piece::BR as usize] | self.pieces[Piece::BQ as usize],
                self.pieces[Piece::BB as usize] | self.pieces[Piece::BQ as usize],
                self.pieces[Piece::BK as usize],
            ),
            Color::Black => (
                {
                    let pawns = self.pieces[Piece::WP as usize];
                    ((pawns << 7) & !FILE_H) | ((pawns << 9) & !FILE_A)
                },
                self.pieces[Piece::WN as usize],
                self.pieces[Piece::WR as usize] | self.pieces[Piece::WQ as usize],
                self.pieces[Piece::WB as usize] | self.pieces[Piece::WQ as usize],
                self.pieces[Piece::WK as usize],
            ),
        };
        let kingless_occ = self.occ ^ Bitboard::from(king_sq);
        danger |= pawn_attacks;
        for sq in knights {
            danger |= KNIGHT_ATTACKS[sq.as_usize()];
        }
        for sq in diag_pieces {
            danger |= bishop_attacks(sq.as_usize(), kingless_occ);
        }
        for sq in orth_pieces {
            danger |= rook_attacks(sq.as_usize(), kingless_occ);
        }
        danger |= KING_ATTACKS[enemy_king.trailing_zeros() as usize];

        danger
    }

    fn get_king_moves(&self, f: &mut impl FnMut(Move), king_sq: Square, danger_mask: Bitboard) {
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

            f(Move::new(king_sq, to, Piece::None, flag));
        }
    }

    pub fn compute_checkers(&self, king_sq: Square) -> Bitboard {
        let mut checkers = Bitboard(0);
        let (pawns, pawn_attack_table, knights, orth_pieces, diag_pieces) =
            match self.side_to_move.flip() {
                Color::Black => (
                    self.pieces[Piece::BP as usize],
                    WHITE_PAWN_ATTACKS,
                    self.pieces[Piece::BN as usize],
                    self.pieces[Piece::BR as usize] | self.pieces[Piece::BQ as usize],
                    self.pieces[Piece::BB as usize] | self.pieces[Piece::BQ as usize],
                ),
                Color::White => (
                    self.pieces[Piece::WP as usize],
                    BLACK_PAWN_ATTACKS,
                    self.pieces[Piece::WN as usize],
                    self.pieces[Piece::WR as usize] | self.pieces[Piece::WQ as usize],
                    self.pieces[Piece::WB as usize] | self.pieces[Piece::WQ as usize],
                ),
            };
        let pawn_attacks = pawn_attack_table[king_sq.as_usize()];
        checkers |= pawn_attacks & pawns;

        let knight_attacks = KNIGHT_ATTACKS[king_sq.as_usize()];
        checkers |= knight_attacks & knights;

        let diag_attacks = bishop_attacks(king_sq.as_usize(), self.occ);
        checkers |= diag_attacks & diag_pieces;

        let orth_attacks = rook_attacks(king_sq.as_usize(), self.occ);
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

        let potential_diag_attackers =
            BISHOP_ATTACKS[BISHOP_OFFSETS[king_sq.as_usize()]] & diag_pieces;
        let potential_orth_attackers = ROOK_ATTACKS[ROOK_OFFSETS[king_sq.as_usize()]] & orth_pieces;

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
            Piece::BR | Piece::WR | Piece::BB | Piece::WB | Piece::BQ | Piece::WQ => {
                BETWEEN[king_sq.as_usize()][checker_sq.as_usize()]
            }
            _ => Bitboard(0),
        }
    }

    fn gen_castling_moves(&self, f: &mut impl FnMut(Move), king_danger_squares: Bitboard) {
        let castling_rights = self.castling;
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
                    f(Move::new(Square(4), Square(6), Piece::None, CASTLING_FLAG));
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
                    f(Move::new(Square(4), Square(2), Piece::None, CASTLING_FLAG));
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
                    f(Move::new(
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
                    f(Move::new(
                        Square(60),
                        Square(58),
                        Piece::None,
                        CASTLING_FLAG,
                    ));
                }
            }
        }
    }

    fn get_non_king_moves(
        &self,
        f: &mut impl FnMut(Move),
        move_mask: Bitboard,
        pinned: Bitboard,
        pin_rays: &[Bitboard; 64],
        king_sq: Square,
    ) {
        match self.side_to_move {
            Color::White => {
                let (knights, rooks, bishops, queens) = (
                    self.pieces[Piece::WN as usize],
                    self.pieces[Piece::WR as usize],
                    self.pieces[Piece::WB as usize],
                    self.pieces[Piece::WQ as usize],
                );
                self.get_pawn_moves::<true>(f, move_mask, pinned, pin_rays, king_sq);
                self.get_knight_moves(f, knights, move_mask, pinned);
                self.get_sliding_moves::<true, false>(f, rooks, move_mask, pinned, pin_rays);
                self.get_sliding_moves::<false, true>(f, bishops, move_mask, pinned, pin_rays);
                self.get_sliding_moves::<true, true>(f, queens, move_mask, pinned, pin_rays);
            }
            Color::Black => {
                let (knights, rooks, bishops, queens) = (
                    self.pieces[Piece::BN as usize],
                    self.pieces[Piece::BR as usize],
                    self.pieces[Piece::BB as usize],
                    self.pieces[Piece::BQ as usize],
                );
                self.get_pawn_moves::<false>(f, move_mask, pinned, pin_rays, king_sq);
                self.get_knight_moves(f, knights, move_mask, pinned);
                self.get_sliding_moves::<true, false>(f, rooks, move_mask, pinned, pin_rays);
                self.get_sliding_moves::<false, true>(f, bishops, move_mask, pinned, pin_rays);
                self.get_sliding_moves::<true, true>(f, queens, move_mask, pinned, pin_rays);
            }
        }
    }

    pub fn get_legal_moves_list(&self, moves: &mut MoveList) {
        moves.clear();
        self.feed_legal_moves(|mv| {
            moves.push(mv);
        });
    }

    pub fn feed_legal_moves(&self, mut f: impl FnMut(Move)) {
        let side_to_move = self.side_to_move;

        let king_sq = Square(
            self.pieces[if side_to_move == Color::White {
                Piece::WK
            } else {
                Piece::BK
            } as usize]
                .trailing_zeros(),
        );
        debug_assert!(king_sq.as_u8() < 64);

        let king_danger_mask = self.compute_danger(king_sq); // Squares the king cannot move to, i.e attacked squares if the king did not exist

        let in_check = !(king_danger_mask & Bitboard::from(king_sq)).is_empty();

        self.get_king_moves(&mut f, king_sq, king_danger_mask);

        let move_mask = if in_check {
            let checkers: Bitboard = self.compute_checkers(king_sq); // Bitmap of pieces that put the king in check

            if checkers.count_ones() > 1 {
                return; // If it is a double check, only king moves can be legal
            }
            // block_mask: bitmap of squares a piece can go to in order to stop the check (if the checker is a sliding piece, otherwise no squares can block it without capturing)
            self.compute_block_mask(king_sq, checkers) | checkers
        } else {
            self.gen_castling_moves(&mut f, king_danger_mask); // Castling moves can only happen when not in check
            !Bitboard(0) // All squares are valid to move to and capture on if not check are in place (except for pins)
        };

        // pinned: bitmap of pieces that cannot move because they would put the king in check
        // pin_rays: list of bitmaps for each square a piece could be on, that show where that piece can move without puting the king in check
        let (pinned, pin_rays) = self.compute_pins(king_sq);

        self.get_non_king_moves(&mut f, move_mask, pinned, &pin_rays, king_sq);
    }

    pub fn is_square_attacked(&self, sq: Square, attacker: Color) -> bool {
        let sq_usize = sq.as_usize();
        let (pawn_table, pawn, knight, bishop, rook, queen, king) = match attacker {
            Color::White => (
                BLACK_PAWN_ATTACKS,
                Piece::WP,
                Piece::WN,
                Piece::WB,
                Piece::WR,
                Piece::WQ,
                Piece::WK,
            ),
            Color::Black => (
                WHITE_PAWN_ATTACKS,
                Piece::BP,
                Piece::BN,
                Piece::BB,
                Piece::BR,
                Piece::BQ,
                Piece::BK,
            ),
        };

        if !(pawn_table[sq_usize] & self.pieces[pawn as usize]).is_empty() {
            return true;
        }

        if !(KNIGHT_ATTACKS[sq_usize] & self.pieces[knight as usize]).is_empty() {
            return true;
        }

        let diag_attackers = self.pieces[bishop as usize] | self.pieces[queen as usize];
        if !diag_attackers.is_empty() {
            let diag_attacks = bishop_attacks(sq_usize, self.occ);
            if !(diag_attacks & diag_attackers).is_empty() {
                return true;
            }
        }

        let orth_attackers = self.pieces[rook as usize] | self.pieces[queen as usize];
        if !orth_attackers.is_empty() {
            let orth_attacks = rook_attacks(sq_usize, self.occ);
            if !(orth_attacks & orth_attackers).is_empty() {
                return true;
            }
        }

        !(KING_ATTACKS[sq_usize] & self.pieces[king as usize]).is_empty()
    }
}
