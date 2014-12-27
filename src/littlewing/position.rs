use littlewing::common::*;

#[deriving(Copy)]
pub struct Position {
    pub side: Color
}

impl Position {
    pub fn new() -> Position {
        Position {
            side: WHITE
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
