use std::ops::Index;

use common::*;

#[derive(Copy, Clone)]
pub struct Position {
    pub halfmoves_count: u8,
    pub hash: u64,
    pub side: Color,
    pub capture: Piece, // TODO: use `Option<Piece>`?
    pub en_passant: Square, // TODO: use `Option<Square>`?
    pub null_move_right: bool,
    pub castling_rights: [[bool; 2]; 2]
}

impl Position {
    pub fn new() -> Position {
        Position {
            halfmoves_count: 0,
            hash: 0, // TODO: is it a problem for the starting position?
            side: WHITE,
            capture: EMPTY, // TODO: use `None`?
            en_passant: OUT, // TODO: use `None`?
            null_move_right: true,
            castling_rights: [[false; 2]; 2]
        }
    }

}

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
        self.stack[self.ply] = position;
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
            return true // Fifty-move rule
        }

        // Threefold repetitions
        let hash = self.top().hash;
        let mut i = self.len() - 1;
        while i >= 2 {
            i -= 2;
            if self[i].hash == hash {
                return true
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
        self.stack[self.ply - 1].null_move_right = true;
    }

    // FIXME: this should be in `Position`
    pub fn disable_null_move(&mut self) {
        self.stack[self.ply - 1].null_move_right = false;
    }
}

impl Index<usize> for Positions {
    type Output = Position;

    fn index(&self, _index: usize) -> &Position {
        &self.stack[_index]
    }
}
