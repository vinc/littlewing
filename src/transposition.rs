use crate::std::prelude::v1::*;

use crate::common::*;
use crate::piece_move::PieceMove;

#[repr(u8)]
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum Bound {
    Exact,
    Lower,
    Upper
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Transposition {
    hash: u64,            // 64 bits => 8 bytes
    best_move: PieceMove, // 16 bits => 2 bytes
    score: Score,         // 16 bits => 2 bytes
    depth: Depth,         //  8 bits => 1 bytes
    bound: Bound,         //  8 bits => 1 bytes
    age: u8,              //  8 bits => 1 bytes
}

impl Transposition {
    pub fn new(hash: u64, depth: Depth, score: Score, best_move: PieceMove, bound: Bound, age: u8) -> Transposition {
        Transposition { hash, depth, score, best_move, bound, age }
    }

    pub fn new_null() -> Transposition {
        Transposition::new(0, 0, 0, PieceMove::new_null(), Bound::Exact, 0)
    }

    pub fn hash(&self) -> u64 {
        self.hash
    }

    pub fn data(&self) -> u64 {
        let m = self.best_move.to_u16() as u64;
        let s = self.score as u64;
        let d = self.depth as u64;
        let b = self.bound as u64;
        let a = self.age as u64;
        // m 0x1111111111111111000000000000000000000000000000000000000000000000
        // s 0x0000000000000000111111111111111100000000000000000000000000000000
        // d 0x0000000000000000000000000000000011111111000000000000000000000000
        // b 0x0000000000000000000000000000000000000000111111110000000000000000
        // a 0x0000000000000000000000000000000000000000000000001111111100000000
        m << 48 | s << 32 | d << 24 | b << 16 | a << 8
    }

    pub fn xored(&mut self) {
        self.hash = self.hash ^ self.data();
    }

    pub fn depth(&self) -> Depth {
        self.depth
    }

    pub fn score(&self) -> Score {
        self.score
    }

    pub fn best_move(&self) -> PieceMove {
        self.best_move
    }

    pub fn bound(&self) -> Bound {
        self.bound
    }

    pub fn age(&self) -> u8 {
        self.age
    }
}

#[cfg(test)]
mod tests {
    use crate::std::mem;

    use super::*;
    use crate::piece_move::PieceMove;

    #[test]
    fn test_size_of_transposition() {
        assert_eq!(mem::size_of::<u64>(),       8); // Hash
        assert_eq!(mem::size_of::<Score>(),     2); // Score
        assert_eq!(mem::size_of::<PieceMove>(), 2); // PieceMove
        assert_eq!(mem::size_of::<u8>(),        1); // Depth

        assert_eq!(mem::size_of::<Transposition>(), 16);
    }
}
