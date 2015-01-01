use littlewing::common::*;

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
    fn color(&self) -> Color;
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
