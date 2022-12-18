use crate::std::prelude::v1::*;

use crate::common::*;
use crate::piece_move::PieceMove;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Debug)]
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

    // Total: 16 bytes
    //
    // NOTE: `depth` will never go above MAX_PLY, which is 128 so we can store
    // it as `i8`.
    //
    // TODO: we don't need to store the whole hash as the first part is the
    // index of the entry: `entries[hash % size]`
}

impl Transposition {
    pub fn new(hash: u64, depth: Depth, score: Score, best_move: PieceMove, bound: Bound, age: u8) -> Transposition {
        Transposition {
            hash: hash,
            depth: depth,
            score: score,
            best_move: best_move,
            bound: bound,
            age: age
        }
    }

    pub fn new_null() -> Transposition {
        Transposition::new(0, 0, 0, PieceMove::new_null(), Bound::Exact, 0)
    }

    pub fn hash(&self) -> u64 {
        self.hash
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
