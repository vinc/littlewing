use std::prelude::v1::*;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::common::*;
use crate::piece_move::PieceMove;

#[repr(u8)]
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum Bound {
    Exact,
    Lower,
    Upper
}

impl Bound {
    fn from_u8(n: u8) -> Bound {
        match n {
            0 => Bound::Exact,
            1 => Bound::Lower,
            2 => Bound::Upper,
            _ => Bound::Exact
        }
    }
}

#[derive(Debug)]
pub struct Transposition {
    hash: AtomicU64,
    data: AtomicU64
}

impl Transposition {
    pub const fn new_null() -> Transposition {
        Transposition {
            hash: AtomicU64::new(0),
            data: AtomicU64::new(0)
        }
    }

    pub fn new(hash: u64, depth: Depth, score: Score, best_move: PieceMove, bound: Bound, age: u8) -> Transposition {
        let entry = Transposition::new_null();
        entry.store(hash, depth, score, best_move, bound, age);
        entry
    }

    pub fn store(&self, hash: u64, depth: Depth, score: Score, best_move: PieceMove, bound: Bound, age: u8) {
        let a = ((age as u64) & 0xFF) << 8;
        let b = ((bound as u8) as u64 & 0xFF) << 16;
        let d = ((depth as u8) as u64 & 0xFF) << 24;
        let s = ((score as u16) as u64 & 0xFFFF) << 32;
        let m = (best_move.to_u16() as u64 & 0xFFFF) << 48;
        let data = a | b | d | s | m;
        self.data.store(data, Ordering::Release);
        self.hash.store(hash ^ data, Ordering::Release);
    }

    pub fn hash(&self) -> u64 {
        self.hash.load(Ordering::Acquire)
    }

    pub fn data(&self) -> u64 {
        self.data.load(Ordering::Acquire)
    }

    pub fn hash_and_data(&self) -> (u64, u64) {
        loop {
            let hash_before = self.hash.load(Ordering::Acquire);
            let data = self.data.load(Ordering::Acquire);
            let hash_after = self.hash.load(Ordering::Acquire);
            if hash_before == hash_after {
                return (hash_after, data);
            }
        }
    }

    pub fn age(&self) -> u8 {
        ((self.data() >> 8) & 0xFF) as u8
    }

    pub fn bound(&self) -> Bound {
        Bound::from_u8(((self.data() >> 16) & 0xFF) as u8)
    }

    pub fn depth(&self) -> Depth {
        (((self.data() >> 24) & 0xFF) as u8) as Depth
    }

    pub fn score(&self) -> Score {
        (((self.data() >> 32) & 0xFFFF) as u16) as Score
    }

    pub fn best_move(&self) -> PieceMove {
        PieceMove::from_u16(((self.data() >> 48) & 0xFFFF) as u16)
    }

    pub fn is_null(&self) -> bool {
        self.best_move().is_null()
    }
}

impl Clone for Transposition {
    fn clone(&self) -> Self {
        Transposition {
            hash: AtomicU64::new(self.hash()),
            data: AtomicU64::new(self.data()),
        }
    }
}

impl PartialEq for Transposition {
    fn eq(&self, other: &Self) -> bool {
        let h1 = self.hash.load(Ordering::Relaxed);
        let h2 = other.hash.load(Ordering::Relaxed);
        let d1 = self.data.load(Ordering::Relaxed);
        let d2 = other.data.load(Ordering::Relaxed);
        h1 == h2 && d1 == d2
    }
}

impl Eq for Transposition {}

impl Default for Transposition {
    fn default() -> Self {
        Transposition::new_null()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::piece_move::PieceMove;
    use crate::square::*;

    #[test]
    fn test_size_of_transposition() {
        assert_eq!(core::mem::size_of::<Transposition>(), 16);
    }

    #[test]
    fn test_new() {
        let hash = 0x1234_5678_9ABC_DEF0;
        let depth: Depth = 8;
        let score: Score = -231;
        let best_move = PieceMove::new(E2, E4, DOUBLE_PAWN_PUSH);
        let bound = Bound::Lower;
        let age = 17;

        let entry = Transposition::new(hash, depth, score, best_move, bound, age);

        let (hash_word, data_word) = entry.hash_and_data();
        assert_eq!(hash_word ^ data_word, hash);
        assert_eq!(entry.depth(), depth);
        assert_eq!(entry.score(), score);
        assert_eq!(entry.best_move(), best_move);
        assert_eq!(entry.bound(), bound);
        assert_eq!(entry.age(), age);
    }

    #[test]
    fn test_store() {
        let hash = 0x1234_5678_9ABC_DEF0;
        let depth: Depth = 8;
        let score: Score = -231;
        let best_move = PieceMove::new(E2, E4, DOUBLE_PAWN_PUSH);
        let bound = Bound::Lower;
        let age = 17;

        let entry = Transposition::new_null();
        entry.store(hash, depth, score, best_move, bound, age);

        let (hash_word, data_word) = entry.hash_and_data();
        assert_eq!(hash_word ^ data_word, hash);
        assert_eq!(entry.depth(), depth);
        assert_eq!(entry.score(), score);
        assert_eq!(entry.best_move(), best_move);
        assert_eq!(entry.bound(), bound);
        assert_eq!(entry.age(), age);
    }
}
