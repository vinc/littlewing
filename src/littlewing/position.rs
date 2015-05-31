use std::ops::Index;

use littlewing::common::*;

#[derive(Copy, Clone)]
pub struct Position {
    pub hash: u64,
    pub side: Color,
    pub capture: Piece,
    pub en_passant: Square,
    pub castling_rights: [[bool; 2]; 2]
}

impl Position {
    pub fn new() -> Position {
        Position {
            hash: 0, // FIXME
            side: WHITE,
            capture: EMPTY,
            en_passant: OUT,
            castling_rights: [[false; 2]; 2]
        }
    }
}

pub struct Positions {
    stack: [Position; MAX_PLY],
    ply: usize
}

impl Positions {
    pub fn new() -> Positions {
        Positions {
            stack: [Position::new(); MAX_PLY],
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
    pub fn top(&self) -> &Position {
        &self.stack[self.ply - 1]
    }
    pub fn len(&self) -> usize {
        self.ply
    }
}

impl Index<usize> for Positions {
    type Output = Position;
    fn index(&self, _index: usize) -> &Position {
        &self.stack[_index]
    }
}
