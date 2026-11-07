use std::fmt;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Shl, Shr};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
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

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    White = 0,
    Black = 1,
}

impl Piece {
    pub const PIECE_LIST: [Self; 12] = [
        Self::WP,
        Self::WN,
        Self::WB,
        Self::WR,
        Self::WQ,
        Self::WK,
        Self::BP,
        Self::BN,
        Self::BB,
        Self::BR,
        Self::BQ,
        Self::BK,
    ];
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::White => write!(f, "White"),
            Self::Black => write!(f, "Black"),
        }
    }
}

impl Color {
    pub const fn flip(self) -> Self {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}

pub const CAPTURE_FLAG: u8 = 1 << 0;
pub const PROMOTION_FLAG: u8 = 1 << 1;
pub const EN_PASSANT_FLAG: u8 = 1 << 2;
pub const CASTLING_FLAG: u8 = 1 << 3;
pub const DOUBLE_PAWN_PUSH_FLAG: u8 = 1 << 4;

#[derive(Copy, PartialEq, Eq, Clone, Debug)]
pub struct Move(u32);

impl Move {
    pub fn new(from: Square, to: Square, promotion_piece: Piece, flags: u8) -> Self {
        Self(
            from.as_u32() // 6 bits
             | ((to.as_u32()) << 6) // 6 bits
             | ((promotion_piece as u32)<< 12) // 4 bits
             | ((u32::from(flags)) << 16), // 5 bits
        )
    }

    pub const fn from(self) -> Square {
        Square((self.0 & 0b111_111) as u8)
    }

    pub const fn to(self) -> Square {
        Square(((self.0 >> 6) & 0b111_111) as u8)
    }

    pub const fn promotion(self) -> Piece {
        let index = ((self.0 >> 12) & 0b1111) as usize;
        if index < 12 {
            Piece::PIECE_LIST[index]
        } else {
            Piece::None
        }
    }

    pub const fn flags(self) -> u8 {
        ((self.0 >> 16) & 0b11111) as u8
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Square(pub u8);

impl Square {
    pub const fn as_u8(self) -> u8 {
        self.0
    }

    pub const fn as_u32(self) -> u32 {
        self.0 as u32
    }

    pub const fn as_usize(self) -> usize {
        self.0 as usize
    }

    pub const NO_SQUARE: Self = Self(64);
}

#[derive(Clone)]
pub struct MoveList {
    moves: [Move; 218],
    pub count: usize,
}

impl FromIterator<Move> for MoveList {
    fn from_iter<T: IntoIterator<Item = Move>>(iter: T) -> Self {
        let mut list = Self::new();
        for mv in iter {
            if list.count < 218 {
                list.push(mv);
            } else {
                break;
            }
        }
        list
    }
}

impl MoveList {
    pub fn new() -> Self {
        Self {
            moves: [Move::new(Square(0), Square(0), Piece::None, 0); 218],
            count: 0,
        }
    }
    pub const fn get(&self, index: usize) -> Option<Move> {
        if index < self.count {
            Some(self.moves[index])
        } else {
            None
        }
    }
    pub const fn push(&mut self, mv: Move) {
        self.moves[self.count] = mv;
        self.count += 1;
    }
    pub const fn clear(&mut self) {
        self.count = 0;
    }
    pub const fn swap_pop(&mut self, index: usize) -> Option<Move> {
        if index >= self.count {
            return None;
        }
        let tmp = self.moves[index];
        self.moves[index] = self.moves[self.count - 1];
        self.count -= 1;
        Some(tmp)
    }
    pub fn iter(&self) -> std::slice::Iter<'_, Move> {
        self.moves[..self.count].iter()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Bitboard(pub u64);

impl Bitboard {
    #[allow(clippy::cast_possible_truncation)]
    pub const fn pop_lsb(&mut self) -> u8 {
        let index = self.0.trailing_zeros() as u8;
        self.0 &= self.0 - 1;
        index
    }
    #[allow(clippy::cast_possible_truncation)]
    pub const fn count_ones(self) -> u8 {
        self.0.count_ones() as u8
    }
    #[allow(clippy::cast_possible_truncation)]
    pub const fn trailing_zeros(self) -> u8 {
        self.0.trailing_zeros() as u8
    }
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
    pub const fn wrapping_mul(self, rhs: u64) -> u64 {
        self.0.wrapping_mul(rhs)
    }
}

impl From<Square> for Bitboard {
    fn from(sq: Square) -> Self {
        Self(1 << sq.0)
    }
}

impl Iterator for Bitboard {
    type Item = Square;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_empty() {
            None
        } else {
            Some(Square(self.pop_lsb()))
        }
    }
}

impl BitOr for Bitboard {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}
impl BitOrAssign for Bitboard {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}
impl BitAnd for Bitboard {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}
impl BitAndAssign for Bitboard {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}
impl BitXor for Bitboard {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}
impl BitXorAssign for Bitboard {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}
impl Not for Bitboard {
    type Output = Self;
    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}
impl Shr<u8> for Bitboard {
    type Output = Self;
    fn shr(self, rhs: u8) -> Self::Output {
        Self(self.0 >> rhs)
    }
}
impl Shl<u8> for Bitboard {
    type Output = Self;
    fn shl(self, rhs: u8) -> Self::Output {
        Self(self.0 << rhs)
    }
}
