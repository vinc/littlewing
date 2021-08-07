#![allow(dead_code)]

use piece::*;
use square::*;
use bitboard::Bitboard;

use alloc::string::ToString;
use core::sync::atomic::{AtomicBool, Ordering};

pub type Shift = i8;
pub type Direction = usize;
pub type PieceMoveType = u8;
pub type Score = i16;
pub type Depth = i8;

pub const INF: Score = 29999;

pub const UP:    Shift = 8;
pub const DOWN:  Shift = -8;
pub const LEFT:  Shift = -1;
pub const RIGHT: Shift = 1;

pub const NORTH:     Direction = 0; // 0b0000
pub const SOUTH:     Direction = 1; // 0b0001
pub const WEST:      Direction = 2; // 0b0010
pub const EAST:      Direction = 3; // 0b0011
pub const NORTHWEST: Direction = 4; // 0b0100
pub const NORTHEAST: Direction = 5; // 0b0101
pub const SOUTHWEST: Direction = 6; // 0b0110
pub const SOUTHEAST: Direction = 7; // 0b0111

pub trait DirectionExt {
    fn is_north(self) -> bool;
    fn is_south(self) -> bool;
    fn is_west(self) -> bool;
    fn is_east(self) -> bool;
}

impl DirectionExt for Direction {
    fn is_north(self) -> bool {
        DIRECTION_SHIFTS[self] > 1
    }
    fn is_south(self) -> bool {
        DIRECTION_SHIFTS[self] < 1
    }
    fn is_east(self) -> bool {
        self > 1 && self % 2 == 0
    }
    fn is_west(self) -> bool {
        self > 1 && self % 2 == 1
    }
}

pub const DIRECTION_SHIFTS: [Shift; 8] = [
    UP,
    DOWN,
    LEFT,
    RIGHT,
    UP + LEFT,
    UP + RIGHT,
    DOWN + LEFT,
    DOWN + RIGHT
];

pub const DIRECTION_MASKS: [Bitboard; 8] = [
    0xFFFFFFFFFFFFFFFF,
    0xFFFFFFFFFFFFFFFF,
    0x7F7F7F7F7F7F7F7F,
    0xFEFEFEFEFEFEFEFE,
    0x7F7F7F7F7F7F7F7F,
    0xFEFEFEFEFEFEFEFE,
    0x7F7F7F7F7F7F7F7F,
    0xFEFEFEFEFEFEFEFE
];

pub const RANK_1: Bitboard = 0x00000000000000FF;
pub const RANK_2: Bitboard = 0x000000000000FF00;
pub const RANK_3: Bitboard = 0x0000000000FF0000;
pub const RANK_4: Bitboard = 0x00000000FF000000;
pub const RANK_5: Bitboard = 0x000000FF00000000;
pub const RANK_6: Bitboard = 0x0000FF0000000000;
pub const RANK_7: Bitboard = 0x00FF000000000000;
pub const RANK_8: Bitboard = 0xFF00000000000000;

pub const RANKS: [Bitboard; 8] = [
    RANK_1,
    RANK_2,
    RANK_3,
    RANK_4,
    RANK_5,
    RANK_6,
    RANK_7,
    RANK_8,
];

pub const FILE_A: Bitboard = 0x0101010101010101;
pub const FILE_B: Bitboard = 0x0202020202020202;
pub const FILE_C: Bitboard = 0x0404040404040404;
pub const FILE_D: Bitboard = 0x0808080808080808;
pub const FILE_E: Bitboard = 0x1010101010101010;
pub const FILE_F: Bitboard = 0x2020202020202020;
pub const FILE_G: Bitboard = 0x4040404040404040;
pub const FILE_H: Bitboard = 0x8080808080808080;

pub const FILES: [Bitboard; 8] = [
    FILE_A,
    FILE_B,
    FILE_C,
    FILE_D,
    FILE_E,
    FILE_F,
    FILE_G,
    FILE_H,
];

pub const QUIET_MOVE:               PieceMoveType = 0b0000; // 0
pub const DOUBLE_PAWN_PUSH:         PieceMoveType = 0b0001; // 1
pub const KING_CASTLE:              PieceMoveType = 0b0010; // 2
pub const QUEEN_CASTLE:             PieceMoveType = 0b0011; // 3
pub const CAPTURE:                  PieceMoveType = 0b0100; // 4
pub const EN_PASSANT:               PieceMoveType = 0b0101; // 5
pub const NULL_MOVE:                PieceMoveType = 0b0110; // 6
pub const KNIGHT_PROMOTION:         PieceMoveType = 0b1000; // 8
pub const BISHOP_PROMOTION:         PieceMoveType = 0b1001; // 9
pub const ROOK_PROMOTION:           PieceMoveType = 0b1010; // 10
pub const QUEEN_PROMOTION:          PieceMoveType = 0b1011; // 11
pub const KNIGHT_PROMOTION_CAPTURE: PieceMoveType = 0b1100; // 12
pub const BISHOP_PROMOTION_CAPTURE: PieceMoveType = 0b1101; // 13
pub const ROOK_PROMOTION_CAPTURE:   PieceMoveType = 0b1110; // 14
pub const QUEEN_PROMOTION_CAPTURE:  PieceMoveType = 0b1111; // 15

pub const BEST_MOVE:                PieceMoveType = 0b00010000; // 16
pub const KILLER_MOVE:              PieceMoveType = 0b00010001; // 17

pub const PROMOTION_MASK:           PieceMoveType = 0b1000;
pub const PROMOTION_KIND_MASK:      PieceMoveType = 0b1100;

pub const PROMOTION_KINDS: [Piece; 4] = [KNIGHT, BISHOP, ROOK, QUEEN];

pub const CASTLING_MASKS: [[Bitboard; 2]; 2] = [
    [1 << F1 | 1 << G1, 1 << B1 | 1 << C1 | 1 << D1],
    [1 << F8 | 1 << G8, 1 << B8 | 1 << C8 | 1 << D8]
];

pub const DEFAULT_FEN: &str =
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
pub const MAX_KILLERS: usize = 2;

pub const TT_SIZE: usize = 8 << 20; // 8 Mb

pub const XSHIFTS: [Shift; 2] = [LEFT, RIGHT];
pub const YSHIFTS: [Shift; 2] = [UP, DOWN];
pub const END_FILES: [Bitboard; 2] = [FILE_A, FILE_H];
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

    static ref COLORIZE: AtomicBool = AtomicBool::new(true);
}

pub fn colorize(b: bool) {
    COLORIZE.store(b, Ordering::Relaxed);
}

pub fn bold(s: &str) -> String {
    if COLORIZE.load(Ordering::Relaxed) {
        format!("\x1b[1m{}\x1b[0m", s)
    } else {
        s.to_string()
    }
}

pub fn bold_green(s: &str) -> String {
    if COLORIZE.load(Ordering::Relaxed) {
        format!("\x1b[1;31m{}\x1b[0m", s)
    } else {
        s.to_string()
    }
}

pub fn bold_red(s: &str) -> String {
    if COLORIZE.load(Ordering::Relaxed) {
        format!("\x1b[1;32m{}\x1b[0m", s)
    } else {
        s.to_string()
    }
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
