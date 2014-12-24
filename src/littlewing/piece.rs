#[deriving(Copy, PartialEq)]
pub enum Piece {
    WhitePawn,
    WhiteKnight,
    WhiteBishop,
    WhiteRook,
    WhiteQueen,
    WhiteKing,
    BlackPawn,
    BlackKnight,
    BlackBishop,
    BlackRook,
    BlackQueen,
    BlackKing
}

pub static PIECES: [Piece, ..12] = [
    Piece::WhitePawn,
    Piece::WhiteKnight,
    Piece::WhiteBishop,
    Piece::WhiteRook,
    Piece::WhiteQueen,
    Piece::WhiteKing,
    Piece::BlackPawn,
    Piece::BlackKnight,
    Piece::BlackBishop,
    Piece::BlackRook,
    Piece::BlackQueen,
    Piece::BlackKing
];

impl Piece {
    pub fn to_char(&self) -> char {
        match self {
            &Piece::WhitePawn   => 'p',
            &Piece::WhiteKnight => 'n',
            &Piece::WhiteBishop => 'b',
            &Piece::WhiteRook   => 'r',
            &Piece::WhiteQueen  => 'q',
            &Piece::WhiteKing   => 'k',
            &Piece::BlackPawn   => 'P',
            &Piece::BlackKnight => 'N',
            &Piece::BlackBishop => 'B',
            &Piece::BlackRook   => 'R',
            &Piece::BlackQueen  => 'Q',
            &Piece::BlackKing   => 'K'
        }
    }
}
