pub type Bitboard = u64;
pub type Color = uint;
pub type Direction = uint; // FIXME: Use `int` instead
pub type MoveType = uint;
pub type Piece = uint;
pub type Square = uint;

pub const WHITE:  Color = 0b0000;
pub const BLACK:  Color = 0b0001;

//pub const LEAPER: Piece = 0b0000;
//pub const SLIDER: Piece = 0b1000;

pub const EMPTY:  Piece = 0b0000;
pub const PAWN:   Piece = 0b0010;
pub const KNIGHT: Piece = 0b0100;
pub const KING:   Piece = 0b0110;
pub const BISHOP: Piece = 0b1000;
pub const ROOK:   Piece = 0b1010;
pub const QUEEN:  Piece = 0b1100;

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

pub const UP:    Direction = 8;
pub const DOWN:  Direction = -8;
pub const LEFT:  Direction = -1;
pub const RIGHT: Direction = 1;

pub const A1: Square = 0;
//pub const B1: Square = 1;
//pub const C1: Square = 2;
//pub const D1: Square = 3;
//pub const E1: Square = 4;
//pub const F1: Square = 5;
//pub const G1: Square = 6;
pub const H1: Square = 7;
//pub const A2: Square = 8;
//pub const B2: Square = 9;
//pub const C2: Square = 10;
//pub const D2: Square = 11;
pub const E2: Square = 12;
//pub const F2: Square = 13;
//pub const G2: Square = 14;
//pub const H2: Square = 15;
//pub const A3: Square = 16;
//pub const B3: Square = 17;
//pub const C3: Square = 18;
//pub const D3: Square = 19;
pub const E3: Square = 20;
//pub const F3: Square = 21;
//pub const G3: Square = 22;
//pub const H3: Square = 23;
//pub const A4: Square = 24;
//pub const B4: Square = 25;
//pub const C4: Square = 26;
//pub const D4: Square = 27;
//pub const E4: Square = 28;
//pub const F4: Square = 29;
//pub const G4: Square = 30;
//pub const H4: Square = 31;
//pub const A5: Square = 32;
//pub const B5: Square = 33;
//pub const C5: Square = 34;
//pub const D5: Square = 35;
//pub const E5: Square = 36;
//pub const F5: Square = 37;
//pub const G5: Square = 38;
//pub const H5: Square = 39;
//pub const A6: Square = 40;
//pub const B6: Square = 41;
//pub const C6: Square = 42;
//pub const D6: Square = 43;
//pub const E6: Square = 44;
//pub const F6: Square = 45;
//pub const G6: Square = 46;
//pub const H6: Square = 47;
//pub const A7: Square = 48;
//pub const B7: Square = 49;
//pub const C7: Square = 50;
//pub const D7: Square = 51;
//pub const E7: Square = 52;
//pub const F7: Square = 53;
//pub const G7: Square = 54;
//pub const H7: Square = 55;
pub const A8: Square = 56;
//pub const B8: Square = 57;
//pub const C8: Square = 58;
//pub const D8: Square = 59;
//pub const E8: Square = 60;
//pub const F8: Square = 61;
//pub const G8: Square = 62;
pub const H8: Square = 63;

//pub const RANK_1: Bitboard = 0x00000000000000FF;
//pub const RANK_2: Bitboard = 0x000000000000FF00;
pub const RANK_3: Bitboard = 0x0000000000FF0000;
//pub const RANK_4: Bitboard = 0x00000000FF000000;
//pub const RANK_5: Bitboard = 0x000000FF00000000;
pub const RANK_6: Bitboard = 0x0000FF0000000000;
//pub const RANK_7: Bitboard = 0x00FF000000000000;
//pub const RANK_8: Bitboard = 0xFF00000000000000;
pub const FILE_A: Bitboard = 0x0101010101010101;
pub const FILE_H: Bitboard = 0x8080808080808080;

pub const QUIET_MOVE:               MoveType = 0; 
pub const DOUBLE_PAWN_PUSH:         MoveType = 1;
//pub const KING_CASTLE:              MoveType = 2;
//pub const QUEEN_CASTLE:             MoveType = 3;
pub const CAPTURE:                  MoveType = 4;
//pub const EN_PASSANT:               MoveType = 5;
//pub const NULL_MOVE:                MoveType = 6;

//pub const KNIGHT_PROMOTION:         MoveType = 8;
//pub const BISHOP_PROMOTION:         MoveType = 9;
//pub const ROOK_PROMOTION:           MoveType = 10;
//pub const QUEEN_PROMOTION:          MoveType = 11;
//pub const KNIGHT_PROMOTION_CAPTURE: MoveType = 12;
//pub const BISHOP_PROMOTION_CAPTURE: MoveType = 13;
//pub const ROOK_PROMOTION_CAPTURE:   MoveType = 14;
//pub const QUEEN_PROMOTION_CAPTURE:  MoveType = 15;

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

/*
pub const SQUARES: [Square, ..64] = [
    A1, B1, C1, D1, E1, F1, G1, H1,
    A2, B2, C2, D2, E2, F2, G2, H2,
    A3, B3, C3, D3, E3, F3, G3, H3,
    A4, B4, C4, D4, E4, F4, G4, H4,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A8, B8, C8, D8, E8, F8, G8, H8
];
*/
