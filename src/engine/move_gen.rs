use crate::engine::bitboards::*;
use crate::engine::board::{BK_CASTLE, BQ_CASTLE, WK_CASTLE, WQ_CASTLE};
use crate::engine::types::*;
use crate::engine::{Board, Move, Piece};
impl Board {
    fn get_white_pawn_moves(
        &self,
        moves: &mut MoveList,
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
            let to = Square::from(single_push.trailing_zeros() as u8);
            single_push &= single_push - 1;
            let from = Square::from(to.as_u8() - 8);
            let is_promotion = to.as_u8() > 55;
            let is_pinned = (pinned >> from.as_u8()) & 1 != 0;
            let is_legal = if is_pinned {
                (1u64 << to.as_u8()) & pin_rays[from.as_usize()] & block_mask != 0
            } else {
                (block_mask >> to.as_u8()) & 1 != 0
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
        let mut double_push = (first_push << 8) & empty;
        while double_push != 0 {
            let to = Square::from(double_push.trailing_zeros() as u8);
            double_push &= double_push - 1;
            let from = Square::from(to.as_u8() - 16);
            let is_pinned = (pinned >> from.as_u8()) & 1 != 0;
            let is_legal = if is_pinned {
                (1u64 << to.as_u8()) & pin_rays[from.as_usize()] & block_mask != 0
            } else {
                (block_mask >> to.as_u8()) & 1 != 0
            };
            if is_legal {
                moves.push(Move::new(from, to, Piece::None, DOUBLE_PAWN_PUSH_FLAG))
            };
        }

        // Captures
        let mut captures = pawns;
        while captures != 0 {
            let from = Square::from(captures.trailing_zeros() as u8);
            captures &= captures - 1;

            let mut attacks = WHITE_PAWN_ATTACKS[from.as_usize()] & self.color_occ[1];

            while attacks != 0 {
                let to = Square::from(attacks.trailing_zeros() as u8);
                attacks &= attacks - 1;

                let is_promotion = to.as_u8() > 55;
                let is_pinned = (pinned >> from.as_u8()) & 1 != 0;
                let is_legal = if is_pinned {
                    (1u64 << to.as_u8()) & pin_rays[from.as_usize()] & capture_mask != 0
                } else {
                    (capture_mask >> to.as_u8()) & 1 != 0
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
        if self.en_passant.as_u8() < 64 {
            let en_passant_mask = 1 << self.en_passant.as_u8();
            let mut left_en_passant = ((pawns << 7) & !FILE_H) & en_passant_mask;
            while left_en_passant != 0 {
                let to = Square::from(left_en_passant.trailing_zeros() as u8);
                left_en_passant &= left_en_passant - 1;
                let from = Square::from(to.as_u8() - 7);
                let captured_sq = to.as_u8() - 8;
                let changed_occ = self.occ
                    ^ (1u64 << from.as_u8())
                    ^ (1u64 << captured_sq)
                    ^ (1u64 << to.as_u8());
                let mut changed_pieces = self.pieces;
                changed_pieces[Piece::BP as usize] ^= 1u64 << captured_sq;
                changed_pieces[Piece::WP as usize] ^= (1u64 << to.as_u8()) | (1u64 << from.as_u8());
                let checkers = self.compute_checkers(
                    Square::from(self.pieces[Piece::WK as usize].trailing_zeros() as u8),
                    changed_occ,
                    Color::Black,
                    &changed_pieces,
                );
                if checkers == 0 {
                    moves.push(Move::new(
                        from,
                        to,
                        Piece::None,
                        CAPTURE_FLAG | EN_PASSANT_FLAG,
                    ));
                }
            }
            let mut right_en_passant = ((pawns << 9) & !FILE_A) & en_passant_mask;
            while right_en_passant != 0 {
                let to = Square::from(right_en_passant.trailing_zeros() as u8);
                right_en_passant &= right_en_passant - 1;
                let from = Square::from(to.as_u8() - 9);
                let captured_sq = Square::from(to.as_u8() - 8);
                let changed_occ = self.occ
                    ^ (1u64 << from.as_u8())
                    ^ (1u64 << captured_sq.as_u8())
                    ^ (1u64 << to.as_u8());
                let mut changed_pieces = self.pieces;
                changed_pieces[Piece::BP as usize] ^= 1u64 << captured_sq.as_u8();
                changed_pieces[Piece::WP as usize] ^= (1u64 << to.as_u8()) | (1u64 << from.as_u8());
                let checkers = self.compute_checkers(
                    Square::from(self.pieces[Piece::WK as usize].trailing_zeros() as u8),
                    changed_occ,
                    Color::Black,
                    &changed_pieces,
                );
                if checkers == 0 {
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
            let to = Square::from(single_push.trailing_zeros() as u8);
            single_push &= single_push - 1;
            let from = Square::from(to.as_u8() + 8);
            let is_promotion = to.as_u8() < 8;
            let is_pinned = (pinned >> from.as_u8()) & 1 != 0;
            let is_legal = if is_pinned {
                (1u64 << to.as_u8()) & pin_rays[from.as_usize()] & block_mask != 0
            } else {
                (block_mask >> to.as_u8()) & 1 != 0
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
        let mut double_push = (first_push >> 8) & empty;
        while double_push != 0 {
            let to = Square::from(double_push.trailing_zeros() as u8);
            double_push &= double_push - 1;
            let from = Square::from(to.as_u8() + 16);
            let is_pinned = (pinned >> from.as_u8()) & 1 != 0;
            let is_legal = if is_pinned {
                (1u64 << to.as_u8()) & pin_rays[from.as_usize()] & block_mask != 0
            } else {
                (block_mask >> to.as_u8()) & 1 != 0
            };
            if is_legal {
                moves.push(Move::new(from, to, Piece::None, DOUBLE_PAWN_PUSH_FLAG));
            }
        }

        // Captures
        let mut captures = pawns;
        while captures != 0 {
            let from = Square::from(captures.trailing_zeros() as u8);
            captures &= captures - 1;

            let mut attacks = BLACK_PAWN_ATTACKS[from.as_usize()] & self.color_occ[0];

            while attacks != 0 {
                let to = Square::from(attacks.trailing_zeros() as u8);
                attacks &= attacks - 1;

                let is_promotion = to.as_u8() < 8;
                let is_pinned = (pinned >> from.as_u8()) & 1 != 0;
                let is_legal = if is_pinned {
                    (1u64 << to.as_u8()) & pin_rays[from.as_usize()] & capture_mask != 0
                } else {
                    (capture_mask >> to.as_u8()) & 1 != 0
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
        if self.en_passant.as_u8() < 64 {
            let en_passant_mask = 1 << self.en_passant.as_u8();
            let mut left_en_passant = ((pawns >> 7) & !FILE_A) & en_passant_mask;
            while left_en_passant != 0 {
                let to = Square::from(left_en_passant.trailing_zeros() as u8);
                left_en_passant &= left_en_passant - 1;
                let from = Square::from(to.as_u8() + 7);
                let captured_sq = Square::from(to.as_u8() + 8);
                let changed_occ = self.occ
                    ^ (1u64 << from.as_u8())
                    ^ (1u64 << captured_sq.as_u8())
                    ^ (1u64 << to.as_u8());
                let mut changed_pieces = self.pieces;
                changed_pieces[Piece::WP as usize] ^= 1u64 << captured_sq.as_u8();
                changed_pieces[Piece::BP as usize] ^= (1u64 << to.as_u8()) | (1u64 << from.as_u8());
                let checkers = self.compute_checkers(
                    Square::from(self.pieces[Piece::BK as usize].trailing_zeros() as u8),
                    changed_occ,
                    Color::White,
                    &changed_pieces,
                );
                if checkers == 0 {
                    moves.push(Move::new(
                        from,
                        to,
                        Piece::None,
                        CAPTURE_FLAG | EN_PASSANT_FLAG,
                    ));
                }
            }
            let mut right_en_passant = ((pawns >> 9) & !FILE_H) & en_passant_mask;
            while right_en_passant != 0 {
                let to = Square::from(right_en_passant.trailing_zeros() as u8);
                right_en_passant &= right_en_passant - 1;
                let from = Square::from(to.as_u8() + 9);
                let captured_sq = Square::from(to.as_u8() + 8);
                let changed_occ = self.occ
                    ^ (1u64 << from.as_u8())
                    ^ (1u64 << captured_sq.as_u8())
                    ^ (1u64 << to.as_u8());
                let mut changed_pieces = self.pieces;
                changed_pieces[Piece::WP as usize] ^= 1u64 << captured_sq.as_u8();
                changed_pieces[Piece::BP as usize] ^= (1u64 << to.as_u8()) | (1u64 << from.as_u8());
                let checkers = self.compute_checkers(
                    Square::from(self.pieces[Piece::BK as usize].trailing_zeros() as u8),
                    changed_occ,
                    Color::White,
                    &changed_pieces,
                );
                if checkers == 0 {
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
        block_mask: u64,
        capture_mask: u64,
        pinned: u64,
    ) {
        let us = self.side_to_move;
        let mut knights = self.pieces[knight_piece as usize] & !pinned;
        while knights != 0 {
            let from = Square::from(knights.trailing_zeros() as u8);
            knights &= knights - 1;

            let mut attacks = KNIGHT_ATTACKS[from.as_usize()]
                & !self.color_occ[us as usize]
                & (block_mask | capture_mask);

            while attacks != 0 {
                let to = Square::from(attacks.trailing_zeros() as u8);
                attacks &= attacks - 1;

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
        block_mask: u64,
        capture_mask: u64,
        pinned: u64,
        pin_rays: &[u64; 64],
    ) {
        let us = self.side_to_move;
        let mut rooks = self.pieces[rook_piece as usize];
        while rooks != 0 {
            let from = Square::from(rooks.trailing_zeros() as u8);
            rooks &= rooks - 1;

            let is_pinned = (pinned >> from.as_u8()) & 1 != 0;

            let mask = ROOK_MASKS[from.as_usize()];
            let blockers = self.occ & mask;
            let idx = (blockers.wrapping_mul(ROOK_MAGIC_NUMBERS[from.as_usize()])
                >> ROOK_SHIFTS[from.as_usize()]) as usize;
            let legal_mask = if is_pinned {
                !self.color_occ[us as usize]
                    & (block_mask | capture_mask)
                    & pin_rays[from.as_usize()]
            } else {
                !self.color_occ[us as usize] & (block_mask | capture_mask)
            };
            let attacks = ROOK_ATTACKS[from.as_usize()][idx] & legal_mask;

            let mut remaining = attacks;
            while remaining != 0 {
                let to = Square::from(remaining.trailing_zeros() as u8);
                remaining &= remaining - 1;

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
        block_mask: u64,
        capture_mask: u64,
        pinned: u64,
        pin_rays: &[u64; 64],
    ) {
        let us = self.side_to_move;
        let mut bishops = self.pieces[bishop_piece as usize];
        while bishops != 0 {
            let from = Square::from(bishops.trailing_zeros() as u8);
            bishops &= bishops - 1;

            let is_pinned = (pinned >> from.as_u8()) & 1 != 0;

            let mask = BISHOP_MASKS[from.as_usize()];
            let blockers = self.occ & mask;
            let idx = (blockers.wrapping_mul(BISHOP_MAGIC_NUMBERS[from.as_usize()])
                >> BISHOP_SHIFTS[from.as_usize()]) as usize;
            let legal_mask = if is_pinned {
                !self.color_occ[us as usize]
                    & (block_mask | capture_mask)
                    & pin_rays[from.as_usize()]
            } else {
                !self.color_occ[us as usize] & (block_mask | capture_mask)
            };
            let attacks = BISHOP_ATTACKS[from.as_usize()][idx] & legal_mask;

            let mut remaining = attacks;
            while remaining != 0 {
                let to = Square::from(remaining.trailing_zeros() as u8);
                remaining &= remaining - 1;

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
        block_mask: u64,
        capture_mask: u64,
        pinned: u64,
        pin_rays: &[u64; 64],
    ) {
        let us = self.side_to_move;
        let mut queens = self.pieces[queen_piece as usize];
        while queens != 0 {
            let from = Square::from(queens.trailing_zeros() as u8);
            queens &= queens - 1;

            let is_pinned = (pinned >> from.as_u8()) & 1 != 0;

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
                !self.color_occ[us as usize]
                    & (block_mask | capture_mask)
                    & pin_rays[from.as_usize()]
            } else {
                !self.color_occ[us as usize] & (block_mask | capture_mask)
            };

            let attacks = (orth_attacks | diag_attacks) & legal_mask;

            let mut remaining = attacks;
            while remaining != 0 {
                let to = Square::from(remaining.trailing_zeros() as u8);
                remaining &= remaining - 1;

                let flag = if self.piece_at(to) == Piece::None {
                    0
                } else {
                    CAPTURE_FLAG
                };
                moves.push(Move::new(from, to, Piece::None, flag));
            }
        }
    }

    fn compute_danger(&self, king_sq: Square) -> u64 {
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
        let kingless_occ = self.occ & !(1u64 << king_sq.as_u8());
        while pawns != 0 {
            let sq = pawns.trailing_zeros() as u8;
            pawns &= pawns - 1;
            danger |= pawn_attack_table[sq as usize];
        }
        while knights != 0 {
            let sq = knights.trailing_zeros() as u8;
            knights &= knights - 1;
            danger |= KNIGHT_ATTACKS[sq as usize];
        }
        while diag_pieces != 0 {
            let sq = diag_pieces.trailing_zeros();
            diag_pieces &= diag_pieces - 1;
            let mask = BISHOP_MASKS[sq as usize];
            let blockers = kingless_occ & mask;
            let idx = (blockers.wrapping_mul(BISHOP_MAGIC_NUMBERS[sq as usize])
                >> BISHOP_SHIFTS[sq as usize]) as usize;
            danger |= BISHOP_ATTACKS[sq as usize][idx];
        }
        while orth_pieces != 0 {
            let sq = orth_pieces.trailing_zeros();
            orth_pieces &= orth_pieces - 1;
            let mask = ROOK_MASKS[sq as usize];
            let blockers = kingless_occ & mask;
            let idx = (blockers.wrapping_mul(ROOK_MAGIC_NUMBERS[sq as usize])
                >> ROOK_SHIFTS[sq as usize]) as usize;
            danger |= ROOK_ATTACKS[sq as usize][idx];
        }
        danger |= KING_ATTACKS[enemy_king.trailing_zeros() as usize];

        danger
    }

    fn get_king_moves(&self, moves: &mut MoveList, king_sq: Square, danger_mask: u64) {
        let us = self.side_to_move;
        let mut attacks =
            KING_ATTACKS[king_sq.as_usize()] & !self.color_occ[us as usize] & !danger_mask;
        while attacks != 0 {
            let to = Square::from(attacks.trailing_zeros() as u8);
            attacks &= attacks - 1;

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
        occ: u64,
        attacker_color: Color,
        pieces: &[u64; 12],
    ) -> u64 {
        let mut checkers = 0;
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

    fn compute_pins(&self, king_sq: Square) -> (u64, [u64; 64]) {
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

        let potential_diag_attackers = BISHOP_ATTACKS[king_sq.as_usize()][0] & diag_pieces;
        let potential_orth_attackers = ROOK_ATTACKS[king_sq.as_usize()][0] & orth_pieces;

        let mut attackers = potential_diag_attackers | potential_orth_attackers;

        while attackers != 0 {
            let attacker_sq = attackers.trailing_zeros() as usize;
            attackers &= attackers - 1;

            let between = BETWEEN[king_sq.as_usize()][attacker_sq];
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

    fn compute_block_mask(&self, king_sq: Square, checkers: u64) -> u64 {
        let checker_sq = Square::from(checkers.trailing_zeros() as u8);
        let checker_piece = self.piece_at(checker_sq);

        match checker_piece {
            Piece::BN | Piece::WN | Piece::BP | Piece::WP => 0,
            Piece::BR | Piece::WR | Piece::BB | Piece::WB | Piece::BQ | Piece::WQ => {
                BETWEEN[king_sq.as_usize()][checker_sq.as_usize()]
            }
            _ => 0,
        }
    }

    fn gen_castling_moves(&self, moves: &mut MoveList, king_danger_squares: u64) {
        let castling_rights = self.castling;
        match self.side_to_move {
            Color::White => {
                if castling_rights & WK_CASTLE != 0
                    && self.occ & ((1u64 << 5) | (1u64 << 6)) == 0
                    && king_danger_squares & ((1u64 << 4) | (1u64 << 5) | (1u64 << 6)) == 0
                {
                    moves.push(Move::new(
                        Square::from(4),
                        Square::from(6),
                        Piece::None,
                        CASTLING_FLAG,
                    ));
                }
                if castling_rights & WQ_CASTLE != 0
                    && self.occ & ((1u64 << 1) | (1u64 << 2) | (1u64 << 3)) == 0
                    && king_danger_squares & ((1u64 << 4) | (1u64 << 3) | (1u64 << 2)) == 0
                {
                    moves.push(Move::new(
                        Square::from(4),
                        Square::from(2),
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
                        Square::from(60),
                        Square::from(62),
                        Piece::None,
                        CASTLING_FLAG,
                    ));
                }
                if castling_rights & BQ_CASTLE != 0
                    && self.occ & ((1u64 << 57) | (1u64 << 58) | (1u64 << 59)) == 0
                    && king_danger_squares & ((1u64 << 60) | (1u64 << 59) | (1u64 << 58)) == 0
                {
                    moves.push(Move::new(
                        Square::from(60),
                        Square::from(58),
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

    pub fn get_legal_moves(&self, moves: &mut MoveList) {
        moves.clear();
        let us = self.side_to_move;

        let king_sq = Square::from(
            self.pieces[if us == Color::White {
                Piece::WK
            } else {
                Piece::BK
            } as usize]
                .trailing_zeros() as u8,
        );
        debug_assert!(king_sq.as_u8() <= 64);

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
}
