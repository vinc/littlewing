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

pub trait Stack<T> {
    fn top(&self) -> &T;
}

impl Stack<Position> for Vec<Position> {
    fn top(&self) -> &Position {
        &self[self.len() - 1]
    }
}
