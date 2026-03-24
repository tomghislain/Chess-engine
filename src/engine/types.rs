use std::fmt;

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
    pub fn flip(self) -> Self {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

pub const CAPTURE_FLAG: u8 = 1 << 0;
pub const PROMOTION_FLAG: u8 = 1 << 1;
pub const EN_PASSANT_FLAG: u8 = 1 << 2;
pub const CASTLING_FLAG: u8 = 1 << 3;
pub const DOUBLE_PAWN_PUSH_FLAG: u8 = 1 << 4;

#[derive(Copy, PartialEq, Clone, Debug)]
pub struct Move(u32);

impl Move {
    pub fn new(from: Square, to: Square, promotion_piece: Piece, flags: u8) -> Self {
        Move(
            from.as_u32() // 6 bits
             | ((to.as_u32()) << 6) // 6 bits
             | ((promotion_piece as u32)<< 12) // 4 bits
             | ((flags as u32) << 16), // 5 bits
        )
    }

    pub fn from(self) -> Square {
        Square((self.0 & 0b111111) as u8)
    }

    pub fn to(self) -> Square {
        Square(((self.0 >> 6) & 0b111111) as u8)
    }

    pub fn promotion(self) -> Piece {
        let index = ((self.0 >> 12) & 0b1111) as usize;
        if index < 12 {
            Piece::PIECE_LIST[index]
        } else {
            Piece::None
        }
    }

    pub fn flags(self) -> u8 {
        ((self.0 >> 16) & 0b11111) as u8
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct Square(u8);

impl Square {
    pub fn as_u8(self) -> u8 {
        self.0
    }

    pub fn as_u32(self) -> u32 {
        self.0 as u32
    }

    pub fn as_usize(self) -> usize {
        self.0 as usize
    }
}

impl From<u8> for Square {
    fn from(v: u8) -> Self {
        Square(v)
    }
}

#[derive(Clone)]
pub struct MoveList {
    moves: [Move; 218],
    pub count: usize,
}

impl FromIterator<Move> for MoveList {
    fn from_iter<T: IntoIterator<Item = Move>>(iter: T) -> Self {
        let mut list = MoveList::new();
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
        MoveList {
            moves: [Move::new(Square::from(0), Square::from(0), Piece::None, 0); 218],
            count: 0,
        }
    }
    pub fn get(&self, index: usize) -> Option<Move> {
        if index < self.count {
            Some(self.moves[index])
        } else {
            None
        }
    }
    pub fn push(&mut self, mv: Move) {
        self.moves[self.count] = mv;
        self.count += 1;
    }
    pub fn clear(&mut self) {
        self.count = 0;
    }
    pub fn swap_pop(&mut self, index: usize) -> Option<Move> {
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
