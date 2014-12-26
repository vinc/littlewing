use littlewing::common::*;

pub struct FEN;

impl FEN {
    pub fn decode_piece(c: char) -> Piece {
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
    pub fn encode_piece(p: Piece) -> char {
        match p {
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
            _            => '?' // FIXME
        }
    }
}
