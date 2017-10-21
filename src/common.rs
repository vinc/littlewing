#![allow(dead_code)]

use piece::*;
use square::*;
use bitboard::Bitboard;

pub type Direction = i8;
pub type MoveType = u8;
pub type Score = i16;
pub type Depth = i8;

pub const INF: Score = 29999;

pub const UP:    Direction = 8;
pub const DOWN:  Direction = -8;
pub const LEFT:  Direction = -1;
pub const RIGHT: Direction = 1;


pub const RANK_1: Bitboard = 0x00000000000000FF;
pub const RANK_2: Bitboard = 0x000000000000FF00;
pub const RANK_3: Bitboard = 0x0000000000FF0000;
pub const RANK_4: Bitboard = 0x00000000FF000000;
pub const RANK_5: Bitboard = 0x000000FF00000000;
pub const RANK_6: Bitboard = 0x0000FF0000000000;
pub const RANK_7: Bitboard = 0x00FF000000000000;
pub const RANK_8: Bitboard = 0xFF00000000000000;
pub const FILE_A: Bitboard = 0x0101010101010101;
pub const FILE_H: Bitboard = 0x8080808080808080;

pub const QUIET_MOVE:               MoveType = 0b0000; // 0
pub const DOUBLE_PAWN_PUSH:         MoveType = 0b0001; // 1
pub const KING_CASTLE:              MoveType = 0b0010; // 2
pub const QUEEN_CASTLE:             MoveType = 0b0011; // 3
pub const CAPTURE:                  MoveType = 0b0100; // 4
pub const EN_PASSANT:               MoveType = 0b0101; // 5
pub const NULL_MOVE:                MoveType = 0b0110; // 6
pub const KNIGHT_PROMOTION:         MoveType = 0b1000; // 8
pub const BISHOP_PROMOTION:         MoveType = 0b1001; // 9
pub const ROOK_PROMOTION:           MoveType = 0b1010; // 10
pub const QUEEN_PROMOTION:          MoveType = 0b1011; // 11
pub const KNIGHT_PROMOTION_CAPTURE: MoveType = 0b1100; // 12
pub const BISHOP_PROMOTION_CAPTURE: MoveType = 0b1101; // 13
pub const ROOK_PROMOTION_CAPTURE:   MoveType = 0b1110; // 14
pub const QUEEN_PROMOTION_CAPTURE:  MoveType = 0b1111; // 15

pub const BEST_MOVE:                MoveType = 0b00010000; // 16
pub const KILLER_MOVE:              MoveType = 0b00010001; // 17

pub const PROMOTION_MASK:           MoveType = 0b1000;
pub const PROMOTION_KIND_MASK:      MoveType = 0b1100;

pub const PROMOTION_KINDS: [Piece; 4] = [KNIGHT, BISHOP, ROOK, QUEEN];

pub const CASTLING_MASKS: [[Bitboard; 2]; 2] = [
    [1 << F1 | 1 << G1, 1 << B1 | 1 << C1 | 1 << D1],
    [1 << F8 | 1 << G8, 1 << B8 | 1 << C8 | 1 << D8]
];

pub const DEFAULT_FEN: &'static str =
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

/*
pub const SQUARES: [Square; 64] = [
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

pub const MAX_PLY: usize = 128;
pub const MAX_MOVES: usize = 256;
pub const MAX_POSITIONS: usize = 512;
pub const MAX_KILLERS: usize = 2;

pub const TT_SIZE: usize = 8 << 20; // 8 Mb

pub const XDIRS: [Direction; 2] = [LEFT, RIGHT];
pub const YDIRS: [Direction; 2] = [UP, DOWN];
pub const FILES: [Bitboard; 2] = [FILE_A, FILE_H];
pub const SEC_RANKS: [Bitboard; 2] = [RANK_3, RANK_6];
pub const END_RANKS: [Bitboard; 2] = [RANK_8, RANK_1];

lazy_static! {
    pub static ref PIECE_MASKS: [[Bitboard; 64]; 14] = { // TODO: s/12/5/
        let mut piece_masks = [[0u64; 64]; 14];

        let deltas = [-2, -1, 0, 1, 2];
        for x in 0..8 {
            for y in 0..8 {
                let from = 8 * x + y;
                for &i in &deltas {
                    for &j in &deltas {
                        for k in 1..7 {
                            let dx = x + i * k;
                            let dy = y + j * k;
                            let to = 8 * dx + dy;
                            if to == from {
                                break;
                            }
                            if dx as u8 >= 8 || dy as u8 >= 8 {
                                break; // Out of board
                            }
                            if i == -2 || j == -2 || i == 2 || j == 2 {
                                if i == -1 || j == -1 || i == 1 || j == 1 {
                                    piece_masks[KNIGHT as usize][from as usize] |= 1 << to;
                                }
                                break;
                            }
                            if k == 1 {
                                piece_masks[KING as usize][from as usize] |= 1 << to;
                            }
                            if (dx + i) as u8 >= 8 || (dy + j) as u8 >= 8 {
                                break; // Edge of the board
                            }
                            if i == 0 || j == 0 {
                                piece_masks[ROOK as usize][from as usize] |= 1 << to;
                            } else {
                                piece_masks[BISHOP as usize][from as usize] |= 1 << to;
                            }
                            piece_masks[QUEEN as usize][from as usize] |= 1 << to;
                        }
                    }
                }
            }
        };

        piece_masks
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_piece_masks() {
        assert_eq!(PIECE_MASKS[KING as usize][A1 as usize],   0x0000000000000302);
        assert_eq!(PIECE_MASKS[KING as usize][E3 as usize],   0x0000000038283800);
        assert_eq!(PIECE_MASKS[KNIGHT as usize][B1 as usize], 0x0000000000050800);
        assert_eq!(PIECE_MASKS[BISHOP as usize][A1 as usize], 0x0040201008040200);
        assert_eq!(PIECE_MASKS[BISHOP as usize][E3 as usize], 0x0000024428002800);
        assert_eq!(PIECE_MASKS[ROOK as usize][E3 as usize],   0x00101010106E1000);
        assert_eq!(PIECE_MASKS[ROOK as usize][A1 as usize],   0x000101010101017E);
    }
}
