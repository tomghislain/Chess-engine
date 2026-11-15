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

#[derive(Clone, Copy)]
struct PinInfo {
    orth_rays: Bitboard,
    diag_rays: Bitboard,
    // pinned_orth: pieces pinned verticaly or horizontaly
    pinned_orth: Bitboard,
    // pinned_diag: pieces pinned diagonaly
    pinned_diag: Bitboard,
}

const WK_EMPTY: Bitboard = Bitboard((1 << 5) | (1 << 6));
const WQ_EMPTY: Bitboard = Bitboard((1 << 1) | (1 << 2) | (1 << 3));

const WK_SAFE: Bitboard = Bitboard((1 << 5) | (1 << 6));
const WQ_SAFE: Bitboard = Bitboard((1 << 3) | (1 << 2));

const BK_EMPTY: Bitboard = Bitboard((1 << 61) | (1 << 62));
const BQ_EMPTY: Bitboard = Bitboard((1 << 57) | (1 << 58) | (1 << 59));

const BK_SAFE: Bitboard = Bitboard((1 << 61) | (1 << 62));
const BQ_SAFE: Bitboard = Bitboard((1 << 59) | (1 << 58));

impl Board {
    #[allow(clippy::too_many_lines)]
    #[inline(always)]
    fn get_pawn_moves<const IS_WHITE: bool>(
        &self,
        f: &mut impl FnMut(Move),
        move_mask: Bitboard,
        king_sq: Square,
        pins: PinInfo,
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

        let push_targets = empty & move_mask;

        // Non-promotions
        let free_pawns = non_promotion_pawns & !(pins.pinned_orth | pins.pinned_diag);
        let pinned_orth_pawns = non_promotion_pawns & pins.pinned_orth;
        let pinned_diag_pawns = non_promotion_pawns & pins.pinned_diag;
        // Free
        let single_push = if IS_WHITE {
            (free_pawns << 8) & push_targets
        } else {
            (free_pawns >> 8) & push_targets
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
            ((((free_pawns & RANK_2) << 8) & empty) << 8) & push_targets
        } else {
            ((((free_pawns & RANK_7) >> 8) & empty) >> 8) & push_targets
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

        let capture_targets = enemy_occ & move_mask;

        let left_captures = if IS_WHITE {
            (free_pawns << 7) & !FILE_H & capture_targets
        } else {
            (free_pawns >> 7) & !FILE_A & capture_targets
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
            (free_pawns << 9) & !FILE_A & capture_targets
        } else {
            (free_pawns >> 9) & !FILE_H & capture_targets
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
        for from in pinned_orth_pawns {
            let from_bb = Bitboard::from(from);
            let single_push = if IS_WHITE {
                (from_bb << 8) & push_targets & pins.orth_rays
            } else {
                (from_bb >> 8) & push_targets & pins.orth_rays
            };
            for to in single_push {
                f(Move::new(from, to, Piece::None, 0));
            }
            let double_push = if IS_WHITE {
                ((((from_bb & RANK_2) << 8) & empty) << 8) & push_targets & pins.orth_rays
            } else {
                ((((from_bb & RANK_7) >> 8) & empty) >> 8) & push_targets & pins.orth_rays
            };
            for to in double_push {
                f(Move::new(from, to, Piece::None, DOUBLE_PAWN_PUSH_FLAG));
            }
        }
        for from in pinned_diag_pawns {
            let attacks = if IS_WHITE {
                unsafe {
                    *WHITE_PAWN_ATTACKS.get_unchecked(from.as_usize())
                        & self.color_occ[Color::Black as usize]
                        & move_mask
                        & pins.diag_rays
                }
            } else {
                unsafe {
                    *BLACK_PAWN_ATTACKS.get_unchecked(from.as_usize())
                        & self.color_occ[Color::White as usize]
                        & move_mask
                        & pins.diag_rays
                }
            };
            for to in attacks {
                f(Move::new(from, to, Piece::None, CAPTURE_FLAG));
            }
        }

        // Promotions
        let free_pawns = promotion_pawns & !(pins.pinned_orth | pins.pinned_diag);
        let pinned_diag_pawns = promotion_pawns & pins.pinned_diag;
        // Free
        let single_push = if IS_WHITE {
            (free_pawns << 8) & push_targets
        } else {
            (free_pawns >> 8) & push_targets
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
            (free_pawns << 7) & !FILE_H & capture_targets
        } else {
            (free_pawns >> 7) & !FILE_A & capture_targets
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
            (free_pawns << 9) & !FILE_A & capture_targets
        } else {
            (free_pawns >> 9) & !FILE_H & capture_targets
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
        for from in pinned_diag_pawns {
            let attacks = if IS_WHITE {
                unsafe {
                    *WHITE_PAWN_ATTACKS.get_unchecked(from.as_usize())
                        & self.color_occ[Color::Black as usize]
                        & move_mask
                        & pins.diag_rays
                }
            } else {
                unsafe {
                    *BLACK_PAWN_ATTACKS.get_unchecked(from.as_usize())
                        & self.color_occ[Color::White as usize]
                        & move_mask
                        & pins.diag_rays
                }
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
            let to_bb = Bitboard::from(to);
            let captured_bb = if IS_WHITE { to_bb >> 8 } else { to_bb << 8 };
            let candidates = if IS_WHITE {
                (WHITE_PAWN_ATTACKS[to.as_usize()] >> 16) & pawns
            } else {
                (BLACK_PAWN_ATTACKS[to.as_usize()] << 16) & pawns
            };
            for from in candidates {
                let changed_occ = self.occ ^ (Bitboard::from(from)) ^ (captured_bb) ^ (to_bb);
                let changed_pawns = if IS_WHITE {
                    self.pieces[Piece::BP as usize] ^ captured_bb
                } else {
                    self.pieces[Piece::WP as usize] ^ captured_bb
                };
                if !self.would_en_passant_check::<IS_WHITE>(king_sq, changed_occ, changed_pawns) {
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

    #[inline(always)]
    fn would_en_passant_check<const IS_WHITE: bool>(
        &self,
        king_sq: Square,
        changed_occ: Bitboard,
        attacker_pawns: Bitboard,
    ) -> bool {
        let sq_usize = king_sq.as_usize();
        let (pawn_table, attacker_bishops, attacker_rooks, attacker_queens) = if IS_WHITE {
            (
                WHITE_PAWN_ATTACKS,
                self.pieces[Piece::BB as usize],
                self.pieces[Piece::BR as usize],
                self.pieces[Piece::BQ as usize],
            )
        } else {
            (
                BLACK_PAWN_ATTACKS,
                self.pieces[Piece::WB as usize],
                self.pieces[Piece::WR as usize],
                self.pieces[Piece::WQ as usize],
            )
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

    #[inline(always)]
    fn get_knight_moves(
        &self,
        f: &mut impl FnMut(Move),
        knights: Bitboard,
        move_mask: Bitboard,
        pins: PinInfo,
    ) {
        let free_knights = knights & !(pins.pinned_orth | pins.pinned_diag);
        let occ = self.occ.0;
        for from in free_knights {
            let attacks = unsafe { *KNIGHT_ATTACKS.get_unchecked(from.as_usize()) } & move_mask;

            for to in attacks {
                let flag = CAPTURE_FLAG & (occ >> to.as_u8()) as u8;

                f(Move::new(from, to, Piece::None, flag));
            }
        }
    }

    #[inline(always)]
    fn get_sliding_moves<const IS_ROOK: bool, const IS_BISHOP: bool>(
        &self,
        f: &mut impl FnMut(Move),
        pieces: Bitboard,
        move_mask: Bitboard,
        pins: PinInfo,
    ) {
        let free_pieces = pieces & !(pins.pinned_orth | pins.pinned_diag);
        let pinned_orth_pieces = pieces & pins.pinned_orth;
        let pinned_diag_pieces = pieces & pins.pinned_diag;

        let occ = self.occ.0;
        for from in free_pieces {
            let mut attacks = Bitboard(0);
            if IS_ROOK {
                attacks |= rook_attacks(from.as_usize(), self.occ);
            }
            if IS_BISHOP {
                attacks |= bishop_attacks(from.as_usize(), self.occ);
            }
            attacks &= move_mask;

            for to in attacks {
                let flag = CAPTURE_FLAG & (occ >> to.as_u8()) as u8;
                f(Move::new(from, to, Piece::None, flag));
            }
        }
        if IS_ROOK {
            for from in pinned_orth_pieces {
                let attacks = rook_attacks(from.as_usize(), self.occ) & move_mask & pins.orth_rays;

                for to in attacks {
                    let flag = CAPTURE_FLAG & (occ >> to.as_u8()) as u8;
                    f(Move::new(from, to, Piece::None, flag));
                }
            }
        }
        if IS_BISHOP {
            for from in pinned_diag_pieces {
                let attacks =
                    bishop_attacks(from.as_usize(), self.occ) & move_mask & pins.diag_rays;

                for to in attacks {
                    let flag = CAPTURE_FLAG & (occ >> to.as_u8()) as u8;
                    f(Move::new(from, to, Piece::None, flag));
                }
            }
        }
    }

    #[inline(always)]
    fn compute_danger<const IS_WHITE: bool>(&self, king_sq: Square) -> Bitboard {
        let mut danger = Bitboard(0);
        let (pawn_attacks, knights, orth_pieces, diag_pieces, enemy_king) = if IS_WHITE {
            (
                {
                    let pawns = self.pieces[Piece::BP as usize];
                    ((pawns >> 7) & !FILE_A) | ((pawns >> 9) & !FILE_H)
                },
                self.pieces[Piece::BN as usize],
                self.pieces[Piece::BR as usize] | self.pieces[Piece::BQ as usize],
                self.pieces[Piece::BB as usize] | self.pieces[Piece::BQ as usize],
                self.pieces[Piece::BK as usize],
            )
        } else {
            (
                {
                    let pawns = self.pieces[Piece::WP as usize];
                    ((pawns << 7) & !FILE_H) | ((pawns << 9) & !FILE_A)
                },
                self.pieces[Piece::WN as usize],
                self.pieces[Piece::WR as usize] | self.pieces[Piece::WQ as usize],
                self.pieces[Piece::WB as usize] | self.pieces[Piece::WQ as usize],
                self.pieces[Piece::WK as usize],
            )
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

    #[inline(always)]
    fn get_king_moves(&self, f: &mut impl FnMut(Move), king_sq: Square, nomove_mask: Bitboard) {
        let attacks = unsafe { *KING_ATTACKS.get_unchecked(king_sq.as_usize()) } & !nomove_mask;
        let occ = self.occ.0;
        for to in attacks {
            let flag = CAPTURE_FLAG & (occ >> to.as_u8()) as u8;

            f(Move::new(king_sq, to, Piece::None, flag));
        }
    }

    #[inline(always)]
    pub fn compute_checkers<const IS_WHITE: bool>(&self, king_sq: Square) -> Bitboard {
        let mut checkers = Bitboard(0);
        let (pawns, pawn_attack_table, knights, orth_pieces, diag_pieces) = if IS_WHITE {
            (
                self.pieces[Piece::BP as usize],
                WHITE_PAWN_ATTACKS,
                self.pieces[Piece::BN as usize],
                self.pieces[Piece::BR as usize] | self.pieces[Piece::BQ as usize],
                self.pieces[Piece::BB as usize] | self.pieces[Piece::BQ as usize],
            )
        } else {
            (
                self.pieces[Piece::WP as usize],
                BLACK_PAWN_ATTACKS,
                self.pieces[Piece::WN as usize],
                self.pieces[Piece::WR as usize] | self.pieces[Piece::WQ as usize],
                self.pieces[Piece::WB as usize] | self.pieces[Piece::WQ as usize],
            )
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

    #[inline(always)]
    fn compute_pins<const IS_WHITE: bool>(
        &self,
        king_sq: Square,
        us: Bitboard,
        orth_rays: Bitboard,
        diag_rays: Bitboard,
    ) -> PinInfo {
        let mut pinned_orth = Bitboard(0);
        let mut pinned_diag = Bitboard(0);

        let (orth_pieces, diag_pieces) = if IS_WHITE {
            (
                self.pieces[Piece::BR as usize] | self.pieces[Piece::BQ as usize],
                self.pieces[Piece::BB as usize] | self.pieces[Piece::BQ as usize],
            )
        } else {
            (
                self.pieces[Piece::WR as usize] | self.pieces[Piece::WQ as usize],
                self.pieces[Piece::WB as usize] | self.pieces[Piece::WQ as usize],
            )
        };

        let potential_orth_attackers = orth_rays & orth_pieces;

        for attacker_sq in potential_orth_attackers {
            let between = unsafe {
                *BETWEEN
                    .get_unchecked(king_sq.as_usize())
                    .get_unchecked(attacker_sq.as_usize())
            };
            let blockers = between & self.occ;
            let our_blockers = between & us;

            if blockers.count_ones() == 1 && !our_blockers.is_empty() {
                pinned_orth |= blockers;
            }
        }

        let potential_diag_attackers = diag_rays & diag_pieces;

        for attacker_sq in potential_diag_attackers {
            let between = unsafe {
                *BETWEEN
                    .get_unchecked(king_sq.as_usize())
                    .get_unchecked(attacker_sq.as_usize())
            };
            let blockers = between & self.occ;
            let our_blockers = between & us;

            if blockers.count_ones() == 1 && !our_blockers.is_empty() {
                pinned_diag |= blockers;
            }
        }

        PinInfo {
            orth_rays,
            diag_rays,
            pinned_orth,
            pinned_diag,
        }
    }

    #[inline(always)]
    fn compute_block_mask(&self, king_sq: Square, checkers: Bitboard) -> Bitboard {
        let checker_sq = Square(checkers.trailing_zeros());
        let checker_piece = self.piece_at(checker_sq);

        match checker_piece {
            Piece::BR | Piece::WR | Piece::BB | Piece::WB | Piece::BQ | Piece::WQ => unsafe {
                *BETWEEN
                    .get_unchecked(king_sq.as_usize())
                    .get_unchecked(checker_sq.as_usize())
            },
            _ => Bitboard(0),
        }
    }

    #[inline(always)]
    fn get_castling_moves<const IS_WHITE: bool, const KING_SIDE: bool, const QUEEN_SIDE: bool>(
        &self,
        f: &mut impl FnMut(Move),
        king_danger_squares: Bitboard,
    ) {
        if IS_WHITE {
            if KING_SIDE
                && (self.occ & WK_EMPTY).is_empty()
                && (king_danger_squares & WK_SAFE).is_empty()
            {
                f(Move::new(Square(4), Square(6), Piece::None, CASTLING_FLAG));
            }
            if QUEEN_SIDE
                && (self.occ & WQ_EMPTY).is_empty()
                && (king_danger_squares & WQ_SAFE).is_empty()
            {
                f(Move::new(Square(4), Square(2), Piece::None, CASTLING_FLAG));
            }
        } else {
            if KING_SIDE
                && (self.occ & BK_EMPTY).is_empty()
                && (king_danger_squares & BK_SAFE).is_empty()
            {
                f(Move::new(
                    Square(60),
                    Square(62),
                    Piece::None,
                    CASTLING_FLAG,
                ));
            }
            if QUEEN_SIDE
                && (self.occ & BQ_EMPTY).is_empty()
                && (king_danger_squares & BQ_SAFE).is_empty()
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

    #[inline(always)]
    fn get_non_king_moves<const IS_WHITE: bool>(
        &self,
        f: &mut impl FnMut(Move),
        move_mask: Bitboard,
        king_sq: Square,
        pins: PinInfo,
    ) {
        let (knights, rooks, bishops, queens) = if IS_WHITE {
            (
                self.pieces[Piece::WN as usize],
                self.pieces[Piece::WR as usize],
                self.pieces[Piece::WB as usize],
                self.pieces[Piece::WQ as usize],
            )
        } else {
            (
                self.pieces[Piece::BN as usize],
                self.pieces[Piece::BR as usize],
                self.pieces[Piece::BB as usize],
                self.pieces[Piece::BQ as usize],
            )
        };
        self.get_pawn_moves::<IS_WHITE>(f, move_mask, king_sq, pins);
        self.get_knight_moves(f, knights, move_mask, pins);
        self.get_sliding_moves::<true, false>(f, rooks, move_mask, pins);
        self.get_sliding_moves::<false, true>(f, bishops, move_mask, pins);
        self.get_sliding_moves::<true, true>(f, queens, move_mask, pins);
    }

    pub fn get_legal_moves_list(&self, moves: &mut MoveList) {
        moves.clear();
        self.feed_legal_moves(|mv| {
            moves.push(mv);
        });
    }

    #[inline(always)]
    pub fn feed_legal_moves(&self, f: impl FnMut(Move)) {
        match self.side_to_move {
            Color::White => self.feed_legal_moves_impl::<true>(f),
            Color::Black => self.feed_legal_moves_impl::<false>(f),
        }
    }

    #[inline(always)]
    fn feed_legal_moves_impl<const IS_WHITE: bool>(&self, mut f: impl FnMut(Move)) {
        let king_sq = Square(
            self.pieces[if IS_WHITE { Piece::WK } else { Piece::BK } as usize].trailing_zeros(),
        );
        debug_assert!(king_sq.as_u8() < 64);

        let us = self.color_occ[if IS_WHITE {
            Color::White as usize
        } else {
            Color::Black as usize
        }];

        let king_danger_mask = self.compute_danger::<IS_WHITE>(king_sq); // Squares the king cannot move to, i.e attacked squares if the king did not exist
        let king_nomove_mask = king_danger_mask | us;

        let in_check = !(king_danger_mask & Bitboard::from(king_sq)).is_empty();

        self.get_king_moves(&mut f, king_sq, king_nomove_mask);

        let move_mask = if in_check {
            let checkers: Bitboard = self.compute_checkers::<IS_WHITE>(king_sq); // Bitmap of pieces that put the king in check

            if checkers.count_ones() > 1 {
                return; // If it is a double check, only king moves can be legal
            }
            // block_mask: bitmap of squares a piece can go to in order to stop the check (if the checker is a sliding piece, otherwise no squares can block it without capturing)
            self.compute_block_mask(king_sq, checkers) | checkers
        } else {
            // Dispatch castling according to rights and color
            let king_side = if IS_WHITE { WK_CASTLE } else { BK_CASTLE };
            let queen_side = if IS_WHITE { WQ_CASTLE } else { BQ_CASTLE };

            let rights = self.castling & (king_side | queen_side);

            match rights {
                0 => (),
                r if r == king_side => {
                    self.get_castling_moves::<IS_WHITE, true, false>(&mut f, king_danger_mask)
                }
                r if r == queen_side => {
                    self.get_castling_moves::<IS_WHITE, false, true>(&mut f, king_danger_mask)
                }
                _ => self.get_castling_moves::<IS_WHITE, true, true>(&mut f, king_danger_mask),
            }

            !Bitboard(0) // All squares are valid to move to and capture on if not check are in place (except for pins)
        } & !us;

        // Rays from the king in horizontal/vertical and diagonal directions, to compute pins
        let orth_rays =
            unsafe { *ROOK_ATTACKS.get_unchecked(*ROOK_OFFSETS.get_unchecked(king_sq.as_usize())) };
        let diag_rays = unsafe {
            *BISHOP_ATTACKS.get_unchecked(*BISHOP_OFFSETS.get_unchecked(king_sq.as_usize()))
        };

        let pins = self.compute_pins::<IS_WHITE>(king_sq, us, orth_rays, diag_rays);

        self.get_non_king_moves::<IS_WHITE>(&mut f, move_mask, king_sq, pins);
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
