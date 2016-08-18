use std::ops::Index;

use common::*;

#[derive(Copy, Clone)]
pub struct Position {
    pub halfmoves_count: u8,
    pub hash: u64,
    pub side: Color,
    pub capture: Piece,
    pub en_passant: Square,
    pub castling_rights: [[bool; 2]; 2]
}

impl Position {
    pub fn new() -> Position {
        Position {
            halfmoves_count: 0,
            hash: 0, // FIXME
            side: WHITE,
            capture: EMPTY,
            en_passant: OUT,
            castling_rights: [[false; 2]; 2]
        }
    }
}

pub struct Positions {
    stack: [Position; MAX_POSITIONS],
    ply: usize
}

impl Positions {
    pub fn new() -> Positions {
        Positions {
            stack: [Position::new(); MAX_POSITIONS],
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

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.ply
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
}

impl Index<usize> for Positions {
    type Output = Position;

    fn index(&self, _index: usize) -> &Position {
        &self.stack[_index]
    }
}
