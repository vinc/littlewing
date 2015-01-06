use littlewing::common::*;

use littlewing::bitboard::BitwiseOperations;
use littlewing::bitboard::dumb7fill;
use littlewing::game::Game;
use littlewing::position::Stack;

pub trait Attack {
    fn is_check(&self) -> bool;
    fn is_attacked(&self, square: Square, side: Color) -> bool;
}

impl Attack for Game {
    fn is_check(&self) -> bool {
        let side = self.positions.top().side ^ 1;
        let king = self.bitboards[side | KING];
        if king == 0 {
            return true; // FIXME: Obviously...
        }
        let square = king.ffs();
        self.is_attacked(square, side)
    }
    fn is_attacked(&self, square: Square, side: Color) -> bool {
        let occupied = self.bitboards[WHITE] | self.bitboards[BLACK];

        let pawns = self.bitboards[side ^ 1 | PAWN];
        let attacks = PAWN_ATTACKS[side][square];
        if attacks & pawns > 0 {
            return true;
        }

        let knights = self.bitboards[side ^ 1 | KNIGHT];
        let attacks = PIECE_MASKS[KNIGHT][square];
        if attacks & knights > 0 {
            return true;
        }

        let king = self.bitboards[side ^ 1 | KING];
        let attacks = PIECE_MASKS[KING][square];
        if attacks & king > 0 {
            return true;
        }

        let queens = self.bitboards[side ^ 1 | QUEEN];

        let bishops = self.bitboards[side ^ 1 | BISHOP];
        let attacks = bishop_attacks(square, occupied);
        if attacks & (bishops | queens) > 0 {
            return true;
        }

        let rooks = self.bitboards[side ^ 1 | ROOK];
        let attacks = rook_attacks(square, occupied);
        if attacks & (rooks | queens) > 0 {
            return true;
        }

        false
    }
}

pub fn bishop_attacks(from: Square, occupied: Bitboard) -> Bitboard {
    const DIRS: [Square; 4] = [
        UP + LEFT,
        DOWN + LEFT,
        DOWN + RIGHT,
        UP + RIGHT
    ];
    const WRAPS: [Bitboard; 4] = [
        0x7F7F7F7F7F7F7F7F,
        0x7F7F7F7F7F7F7F7F,
        0xFEFEFEFEFEFEFEFE,
        0xFEFEFEFEFEFEFEFE
    ];

    let mut targets = 0;
    for i in range(0u, 4) {
        let occluded = dumb7fill(1 << from, !occupied & WRAPS[i], DIRS[i]);
        targets |= occluded.shift(DIRS[i]) & WRAPS[i];
    }

    targets
}
pub fn rook_attacks(from: Square, occupied: Bitboard) -> Bitboard {
    const DIRS: [Square; 4] = [
        UP,
        DOWN,
        LEFT,
        RIGHT
    ];
    const WRAPS: [Bitboard; 4] = [
        0xFFFFFFFFFFFFFFFF,
        0xFFFFFFFFFFFFFFFF,
        0x7F7F7F7F7F7F7F7F,
        0xFEFEFEFEFEFEFEFE
    ];

    let mut targets = 0;
    for i in range(0u, 4) {
        let occluded = dumb7fill(1 << from, !occupied & WRAPS[i], DIRS[i]);
        targets |= occluded.shift(DIRS[i]) & WRAPS[i];
    }

    targets
}

lazy_static! {
    pub static ref PAWN_ATTACKS: [[Bitboard; 64]; 2] = {
        let xdirs = [LEFT, RIGHT];
        let ydirs = [DOWN, UP];
        let files = [FILE_H, FILE_A];
        let mut attacks = [[0; 64]; 2];
        for side in range(0u, 2) {
            for square in range(0u, 64) {
                for i in range(0, 2) {
                    let dir = ydirs[side ^ 1] + xdirs[i];
                    attacks[side][square] |= (1 << square).shift(dir) & !files[i];
                }
            }
        }
        attacks
    };
}
