use littlewing::common::*;

use littlewing::bitboard::BitwiseOperations;
use littlewing::game::Game;
use littlewing::position::Stack;
use littlewing::square::SquareString;

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

        let pawns = self.bitboards[side ^ 1 | PAWN];

        // TODO: Precompute this
        const XDIRS: [[Direction, ..2], ..2] = [[LEFT, RIGHT], [RIGHT, LEFT]];
        const YDIRS: [Direction, ..2] = [DOWN, UP];
        const FILES: [Bitboard, ..2] = [FILE_A, FILE_H];
        let mut attacks = 0;
        for i in range(0, 2) {
            let dir = YDIRS[side ^ 1] + XDIRS[side ^ 1][i];
            attacks |= (1 << square).shift(dir) & !FILES[i];
        }

        //(1 << square).debug();
        //attacks.debug();
        //self.bitboards[side ^ 1 | PAWN].debug();

        if attacks & pawns > 0 {
            return true;
        }

        let knights = self.bitboards[side ^ 1 | KNIGHT];
        let attacks = self.moves.knight_mask[square];
        if attacks & knights > 0 {
            return true;
        }

        let king = self.bitboards[side ^ 1 | KING];
        let attacks = self.moves.king_mask[square];
        if attacks & king > 0 {
            return true;
        }

        false
    }
}
