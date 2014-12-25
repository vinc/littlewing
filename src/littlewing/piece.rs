pub type Piece = uint;

pub const WHITE_PAWN:   Piece = 0u;
pub const WHITE_KNIGHT: Piece = 1u;
pub const WHITE_BISHOP: Piece = 2u;
pub const WHITE_ROOK:   Piece = 3u;
pub const WHITE_QUEEN:  Piece = 4u;
pub const WHITE_KING:   Piece = 5u;
pub const BLACK_PAWN:   Piece = 6u;
pub const BLACK_KNIGHT: Piece = 7u;
pub const BLACK_BISHOP: Piece = 8u;
pub const BLACK_ROOK:   Piece = 9u;
pub const BLACK_QUEEN:  Piece = 10u;
pub const BLACK_KING:   Piece = 11u;

pub const WHITE:        Piece = 12u;
pub const BLACK:        Piece = 13u;

pub static PIECES: [Piece, ..12] = [
    WHITE_PAWN,
    WHITE_KNIGHT,
    WHITE_BISHOP,
    WHITE_ROOK,
    WHITE_QUEEN,
    WHITE_KING,
    BLACK_PAWN,
    BLACK_KNIGHT,
    BLACK_BISHOP,
    BLACK_ROOK,
    BLACK_QUEEN,
    BLACK_KING
];
