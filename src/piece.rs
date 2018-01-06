use color::*;

pub type Piece = u8;

#[allow(dead_code)]
pub const LEAPER: Piece = 0b0000;

#[allow(dead_code)]
pub const SLIDER: Piece = 0b1000;

pub const EMPTY:  Piece = 0b0000; // 0
pub const PAWN:   Piece = 0b0010; // 2
pub const KNIGHT: Piece = 0b0100; // 4
pub const KING:   Piece = 0b0110; // 6
pub const BISHOP: Piece = 0b1000; // 8
pub const ROOK:   Piece = 0b1010; // 10
pub const QUEEN:  Piece = 0b1100; // 12

pub const PIECES: [Piece; 6] = [PAWN, KNIGHT, BISHOP, ROOK, QUEEN, KING];

pub const WHITE_PAWN:   Piece = WHITE | PAWN;
pub const WHITE_KNIGHT: Piece = WHITE | KNIGHT;
pub const WHITE_BISHOP: Piece = WHITE | BISHOP;
pub const WHITE_ROOK:   Piece = WHITE | ROOK;
pub const WHITE_QUEEN:  Piece = WHITE | QUEEN;
pub const WHITE_KING:   Piece = WHITE | KING;
pub const BLACK_PAWN:   Piece = BLACK | PAWN;
pub const BLACK_KNIGHT: Piece = BLACK | KNIGHT;
pub const BLACK_BISHOP: Piece = BLACK | BISHOP;
pub const BLACK_ROOK:   Piece = BLACK | ROOK;
pub const BLACK_QUEEN:  Piece = BLACK | QUEEN;
pub const BLACK_KING:   Piece = BLACK | KING;

pub trait PieceChar {
    fn from_char(c: char) -> Self;
    fn to_char(&self) -> char;
}

impl PieceChar for Piece {
    fn from_char(c: char) -> Piece {
        match c {
            'P' => WHITE_PAWN,
            'N' => WHITE_KNIGHT,
            'B' => WHITE_BISHOP,
            'R' => WHITE_ROOK,
            'Q' => WHITE_QUEEN,
            'K' => WHITE_KING,
            'p' => BLACK_PAWN,
            'n' => BLACK_KNIGHT,
            'b' => BLACK_BISHOP,
            'r' => BLACK_ROOK,
            'q' => BLACK_QUEEN,
            'k' => BLACK_KING,
            _   => EMPTY // FIXME
        }
    }
    fn to_char(&self) -> char {
        match *self {
            WHITE_PAWN   => 'P',
            WHITE_KNIGHT => 'N',
            WHITE_BISHOP => 'B',
            WHITE_ROOK   => 'R',
            WHITE_QUEEN  => 'Q',
            WHITE_KING   => 'K',
            BLACK_PAWN   => 'p',
            BLACK_KNIGHT => 'n',
            BLACK_BISHOP => 'b',
            BLACK_ROOK   => 'r',
            BLACK_QUEEN  => 'q',
            BLACK_KING   => 'k',
            EMPTY        => ' ', // FIXME: not really for FEN format
            _            => '?' // FIXME
        }
    }
}

pub trait PieceAttr {
    /// Get the color of a piece
    fn color(&self) -> Color;

    /// Get a piece without its color
    fn kind(&self) -> Piece;
}

impl PieceAttr for Piece {
    fn color(&self) -> Color {
        *self & 0b0001
    }
    fn kind(&self) -> Piece {
        *self & 0b1110
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_piece() {
        assert_eq!(LEAPER & PAWN,   LEAPER);
        assert_eq!(LEAPER & KNIGHT, LEAPER);
        assert_eq!(LEAPER & KING,   LEAPER);
        assert_eq!(LEAPER & BISHOP, EMPTY);
        assert_eq!(LEAPER & ROOK,   EMPTY);
        assert_eq!(LEAPER & QUEEN,  EMPTY);

        assert_eq!(SLIDER & PAWN,   EMPTY);
        assert_eq!(SLIDER & KNIGHT, EMPTY);
        assert_eq!(SLIDER & KING,   EMPTY);
        assert_eq!(SLIDER & BISHOP, SLIDER);
        assert_eq!(SLIDER & ROOK,   SLIDER);
        assert_eq!(SLIDER & QUEEN,  SLIDER);
    }
}
