use std::prelude::v1::*;
use std::fmt;

use color::*;
use piece::*;
use square::*;
use common::*;
use piece::PieceChar;
use square::SquareExt;

pub const BEST_MOVE_SCORE:    u8 = 255;
pub const KILLER_MOVE_SCORE:  u8 = 254;
pub const GOOD_CAPTURE_SCORE: u8 = 64;
pub const QUIET_MOVE_SCORE:   u8 = 0;

#[derive(Copy, Clone, PartialEq)]
pub struct PieceMove(u16);

impl PieceMove {
    pub fn new(from: Square, to: Square, mt: PieceMoveType) -> PieceMove {
        // from: 6 bits
        // to:   6 bits
        // mt:   4 bits
        PieceMove(((from as u16) << 10) | ((to as u16) << 4) | mt as u16)
    }

    pub fn new_null() -> PieceMove {
        PieceMove(0)
    }

    pub fn from(self) -> Square {
        (self.0 >> 10) as Square
    }

    pub fn to(self) -> Square {
        ((self.0 >> 4) & 0b111111) as Square
    }

    pub fn kind(self) -> PieceMoveType {
        (self.0 & 0b1111) as PieceMoveType
    }

    pub fn is_null(self) -> bool {
        self.0 == 0
    }

    // TODO: Add en passant?
    pub fn is_capture(self) -> bool {
        self.kind() == CAPTURE || self.kind() & PROMOTION_KIND_MASK == PROMOTION_KIND_MASK
    }

    pub fn is_en_passant(self) -> bool {
        self.kind() == EN_PASSANT
    }

    pub fn is_castle(self) -> bool {
        self.kind() == KING_CASTLE || self.kind() == QUEEN_CASTLE
    }

    pub fn castle_kind(self) -> Piece {
        QUEEN_CASTLE << self.kind() - 1
    }

    pub fn is_promotion(self) -> bool {
        self.kind() & PROMOTION_MASK > 0
    }

    pub fn promotion_kind(self) -> Piece {
        PROMOTION_KINDS[(self.kind() & PROMOTION_KIND_MASK >> 2) as usize]
    }

    pub fn to_lan(self) -> String {
        self.to_string()
    }
}

impl fmt::Display for PieceMove {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut out = String::new();
        out.push_str(self.from().to_coord().as_str());
        out.push_str(self.to().to_coord().as_str());
        if self.is_promotion() {
            out.push((BLACK | self.promotion_kind()).to_char());
        }
        write!(f, "{}", out)
    }
}

impl fmt::Debug for PieceMove {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}


#[cfg(test)]
mod tests {
    use piece::*;
    use common::*;
    use super::*;

    #[test]
    fn test_move_is_castle() {
        assert_eq!(PieceMove::new(E2, E3, QUIET_MOVE).is_castle(), false);
        assert_eq!(PieceMove::new(E1, G1, KING_CASTLE).is_castle(), true);
        assert_eq!(PieceMove::new(E1, C1, QUEEN_CASTLE).is_castle(), true);
    }

    #[test]
    fn test_move_castle_kind() {
        assert_eq!(PieceMove::new(E1, G1, KING_CASTLE).castle_kind(), KING);
        assert_eq!(PieceMove::new(E1, C1, QUEEN_CASTLE).castle_kind(), QUEEN);
    }

    #[test]
    fn test_move_is_promotion() {
        assert_eq!(PieceMove::new(E2, E3, QUIET_MOVE).is_promotion(), false);
        assert_eq!(PieceMove::new(E7, E8, QUEEN_PROMOTION).is_promotion(), true);
        assert_eq!(PieceMove::new(E7, D8, QUEEN_PROMOTION_CAPTURE).is_promotion(), true);
    }

    #[test]
    fn test_move_promotion_kind() {
        assert_eq!(PieceMove::new(E7, E8, QUEEN_PROMOTION).promotion_kind(), QUEEN);
        assert_eq!(PieceMove::new(E7, D8, ROOK_PROMOTION_CAPTURE).promotion_kind(), ROOK);
    }
}
