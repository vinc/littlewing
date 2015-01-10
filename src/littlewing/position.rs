use std::ops::Index;

use littlewing::common::*;

#[derive(Copy)]
pub struct Position {
    pub side: Color,
    pub capture: Piece,
    pub en_passant: Square,
    pub castling_rights: [[bool; 2]; 2]
}

impl Position {
    pub fn new() -> Position {
        Position {
            side: WHITE,
            capture: EMPTY,
            en_passant: OUT,
            castling_rights: [[false; 2]; 2]
        }
    }
}

pub struct Positions {
    stack: [Position; MAX_PLY],
    ply: uint
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
    pub fn len(&self) -> uint {
        self.ply
    }
}

impl Index<uint> for Positions {
    type Output = Position;
    fn index(&self, _index: &uint) -> &Position {
        &self.stack[*_index]
    }
}
