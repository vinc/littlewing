use std::ops::Index;

use color::*;
use piece::*;
use square::*;

#[derive(Copy, Clone)]
pub struct Position {
    pub hash: u64,
    pub castling_rights: u8,
    pub halfmoves_count: u8,
    pub side: Color,
    pub capture: Piece, // TODO: use `Option<Piece>`?
    pub en_passant: Square, // TODO: use `Option<Square>`?
    pub null_move_right: bool,
}

// WHITE == 0b0000
// BLACK == 0b0001
// KING  == 0b0110 => 0b0000
// QUEEN == 0b1100 => 0b0010
fn castling_rights_index(side: Color, wing: Piece) -> u8 {
    ((wing >> 3) << 1) | side
}

impl Position {
    pub fn new() -> Position {
        Position {
            hash: 0, // TODO: is it a problem for the starting position?
            halfmoves_count: 0,
            castling_rights: 0,
            side: WHITE,
            capture: EMPTY, // TODO: use `None`?
            en_passant: OUT, // TODO: use `None`?
            null_move_right: true,
        }
    }

    pub fn castling_right(&self, side: Color, wing: Piece) -> bool {
        let i = castling_rights_index(side, wing);
        self.castling_rights & (1 << i) > 0
    }

    pub fn set_castling_right(&mut self, side: Color, wing: Piece) {
        let i = castling_rights_index(side, wing);
        self.castling_rights |= 1 << i;
    }

    pub fn reset_castling_right(&mut self, side: Color, wing: Piece) {
        let i = castling_rights_index(side, wing);
        self.castling_rights &= !(1 << i);
    }
}

const MAX_POSITIONS: usize = 1024;

#[derive(Clone)]
pub struct Positions {
    stack: [Position; MAX_POSITIONS],
    fullmoves_init: u8,
    ply: usize
}

impl Positions {
    pub fn new() -> Positions {
        Positions {
            stack: [Position::new(); MAX_POSITIONS],
            fullmoves_init: 0,
            ply: 0
        }
    }

    pub fn push(&mut self, position: Position) {
        self.stack[self.ply] = position; // FIXME: this operation is very slow
        self.ply += 1;
    }

    pub fn pop(&mut self) { // TODO: pop() should return last Position
        self.ply -= 1;
    }

    pub fn clear(&mut self) {
        self.ply = 0;
    }

    // TODO: this should be mutable.
    pub fn top(&self) -> &Position {
        &self.stack[self.ply - 1]
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.ply
    }

    pub fn halfmoves(&self) -> u8 {
        self.top().halfmoves_count
    }

    pub fn fullmoves(&self) -> u8 {
        let n = self.fullmoves_init + (self.ply / 2) as u8;
        let blacks_started = self.top().side == BLACK && self.ply % 2 == 0;

        if blacks_started { n - 1 } else { n }
    }

    pub fn set_halfmoves(&mut self, n: u8) {
        self.stack[self.ply - 1].halfmoves_count = n;
    }

    pub fn set_fullmoves(&mut self, n: u8) {
        self.fullmoves_init = n;
    }

    pub fn is_draw(&self) -> bool {
        // Fifty-move rule
        if self.top().halfmoves_count >= 99 {
            return true; // Fifty-move rule
        }

        // Threefold repetitions
        let mut repetitions_count = 0;
        let hash = self.top().hash;
        let mut i = self.len() - 1;
        while i >= 2 {
            i -= 2;
            if self[i].hash == hash {
                repetitions_count += 1;
                if repetitions_count > 0 {
                    return true;
                }
            }
            if self[i].halfmoves_count == 0 {
                break;
            }
            // TODO: allow one repetition
        }

        false
    }

    // FIXME: this should be in `Position`
    pub fn enable_null_move(&mut self) {
        debug_assert!(!self.stack[self.ply - 1].null_move_right);
        self.stack[self.ply - 1].null_move_right = true;
    }

    // FIXME: this should be in `Position`
    pub fn disable_null_move(&mut self) {
        debug_assert!(self.stack[self.ply - 1].null_move_right);
        self.stack[self.ply - 1].null_move_right = false;
    }
}

impl Index<usize> for Positions {
    type Output = Position;

    fn index(&self, i: usize) -> &Position {
        &self.stack[i]
    }
}

#[cfg(test)]
mod tests {
    use std::mem;
    use super::*;

    #[test]
    fn test_size_of_position() {
        assert_eq!(mem::size_of::<u64>(),       8); // x1
        assert_eq!(mem::size_of::<u8>(),        1); // x2
        assert_eq!(mem::size_of::<bool>(),      1); // x1
        assert_eq!(mem::size_of::<Color>(),     1); // x1
        assert_eq!(mem::size_of::<Piece>(),     1); // x1
        assert_eq!(mem::size_of::<Square>(),    1); // x1

        assert_eq!(mem::size_of::<Position>(), 16);
    }

    #[test]
    fn test_position_castling_rights() {
        let mut pos = Position::new();
        
        assert_eq!(pos.castling_right(WHITE, KING),  false);
        assert_eq!(pos.castling_right(WHITE, QUEEN), false);
        assert_eq!(pos.castling_right(BLACK, KING),  false);
        assert_eq!(pos.castling_right(BLACK, QUEEN), false);

        pos.set_castling_right(WHITE, KING);
        assert_eq!(pos.castling_right(WHITE, KING),  true);
        assert_eq!(pos.castling_right(WHITE, QUEEN), false);
        assert_eq!(pos.castling_right(BLACK, KING),  false);
        assert_eq!(pos.castling_right(BLACK, QUEEN), false);

        pos.set_castling_right(WHITE, KING);
        assert_eq!(pos.castling_right(WHITE, KING),  true);
        assert_eq!(pos.castling_right(WHITE, QUEEN), false);
        assert_eq!(pos.castling_right(BLACK, KING),  false);
        assert_eq!(pos.castling_right(BLACK, QUEEN), false);

        pos.set_castling_right(BLACK, QUEEN);
        assert_eq!(pos.castling_right(WHITE, KING),  true);
        assert_eq!(pos.castling_right(WHITE, QUEEN), false);
        assert_eq!(pos.castling_right(BLACK, KING),  false);
        assert_eq!(pos.castling_right(BLACK, QUEEN), true);

        pos.reset_castling_right(WHITE, KING);
        assert_eq!(pos.castling_right(WHITE, KING),  false);
        assert_eq!(pos.castling_right(WHITE, QUEEN), false);
        assert_eq!(pos.castling_right(BLACK, KING),  false);
        assert_eq!(pos.castling_right(BLACK, QUEEN), true);

        pos.reset_castling_right(BLACK, QUEEN);
        assert_eq!(pos.castling_right(WHITE, KING),  false);
        assert_eq!(pos.castling_right(WHITE, QUEEN), false);
        assert_eq!(pos.castling_right(BLACK, KING),  false);
        assert_eq!(pos.castling_right(BLACK, QUEEN), false);

        pos.reset_castling_right(BLACK, QUEEN);
        assert_eq!(pos.castling_right(WHITE, KING),  false);
        assert_eq!(pos.castling_right(WHITE, QUEEN), false);
        assert_eq!(pos.castling_right(BLACK, KING),  false);
        assert_eq!(pos.castling_right(BLACK, QUEEN), false);
    }
}
