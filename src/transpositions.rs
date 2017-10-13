use std::mem;
use std::cell::UnsafeCell;

use common::*;
use moves::Move;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Bound {
    Exact,
    Lower,
    Upper
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Transposition {
    hash: u64,       // 64 bits => 8 bytes
    best_move: Move, // 16 bits => 2 bytes
    score: Score,    // 16 bits => 2 bytes
    depth: u8,       //  8 bits => 1 bytes
    bound: Bound,    //  8 bits => 1 bytes

    // Total: 14 bytes, which will use 16 bytes including alignment padding.

    // NOTE: `depth` will never go above MAX_PLY, which is 128 so we can store
    // it as `u8`.
    //
    // TODO: we don't need to store the whole hash as the first part is the
    // index of the entry: `entries[hash % size]`
    //
    // TODO: add age counter incremented at the begining of each root search
    // to be used to replace old entries from previous search without having
    // to clear the whole table.
}

impl Transposition {
    pub fn new(hash: u64, depth: usize, score: Score, best_move: Move, bound: Bound) -> Transposition {
        Transposition {
            hash: hash,
            depth: depth as u8,
            score: score,
            best_move: best_move,
            bound: bound
        }
    }

    pub fn new_null() -> Transposition {
        Transposition::new(0, 0, 0, Move::new_null(), Bound::Exact)
    }

    pub fn depth(&self) -> usize {
        self.depth as usize
    }

    pub fn score(&self) -> Score {
        self.score
    }

    pub fn best_move(&self) -> Move {
        self.best_move
    }

    pub fn bound(&self) -> Bound {
        self.bound
    }
}

#[derive(Clone)]
pub struct Transpositions {
    pub entries: Box<[Transposition]>,
    pub size: usize,
    pub stats_lookups: u64,
    pub stats_inserts: u64,
    pub stats_hits : u64,
    pub stats_collisions: u64
}

impl Transpositions {
    pub fn with_capacity(capacity: usize) -> Transpositions {
        let size = if capacity.is_power_of_two() {
            capacity
        } else {
            capacity.next_power_of_two()
        };

        Transpositions {
            entries: vec![Transposition::new_null(); size].into_boxed_slice(),
            size: size,
            stats_lookups: 0,
            stats_inserts: 0,
            stats_hits: 0,
            stats_collisions: 0
        }
    }

    pub fn with_memory(memory: usize) -> Transpositions {
        let capacity = memory / mem::size_of::<Transposition>();

        Transpositions::with_capacity(capacity)
    }

    pub fn get(&mut self, hash: &u64) -> Option<&Transposition> {
        self.stats_lookups += 1;

        let n = self.size as u64;
        let k = (hash % n) as usize; // TODO: hash & (n - 1)
        let t = &self.entries[k]; // TODO: use get_unchecked?

        // TODO: how faster would it be to just also return null move?
        if t.best_move().is_null() {
            None
        } else if &t.hash != hash {
            self.stats_collisions += 1;
            None
        } else {
            debug_assert_eq!(&t.hash, hash);
            self.stats_hits += 1;
            Some(t)
        }
    }

    pub fn set(&mut self, hash: u64, depth: usize, score: Score, best_move: Move, bound: Bound) {
        self.stats_inserts += 1;

        let t = Transposition::new(hash, depth, score, best_move, bound);
        let n = self.size as u64;
        let k = (hash % n) as usize;

        // NOTE: replacement strategies:
        // 1. Always replace
        // 2. Depth prefered
        if self.entries[k].depth() <= depth { // Using "depth prefered"
            self.entries[k] = t;
        }
    }

    pub fn clear(&mut self) {
        let capacity = self.size;
        self.entries = vec![Transposition::new_null(); capacity].into_boxed_slice();

        self.stats_lookups = 0;
        self.stats_inserts = 0;
        self.stats_hits = 0;
        self.stats_collisions = 0;
    }

    pub fn print_stats(&mut self) {
        println!("# {:15} {}", "tt size:", self.entries.len());
        println!("# {:15} {}", "tt inserts:", self.stats_inserts);
        println!("# {:15} {}", "tt lookups:", self.stats_lookups);
        println!("# {:15} {}", "tt hits:", self.stats_hits);
        println!("# {:15} {}", "tt collisions:", self.stats_collisions);
    }
}

pub struct SharedTranspositions {
    inner: UnsafeCell<Transpositions>,
}

unsafe impl Sync for SharedTranspositions {}

impl SharedTranspositions {
    fn new(tt: Transpositions) -> SharedTranspositions {
        SharedTranspositions {
            inner: UnsafeCell::new(tt)
        }
    }

    pub fn with_memory(memory: usize) -> SharedTranspositions {
        SharedTranspositions::new(Transpositions::with_memory(memory))
    }

    pub fn get(&self) -> &mut Transpositions {
        unsafe { &mut *self.inner.get() }
    }
}

#[cfg(test)]
mod tests {
    use std::mem;

    use super::*;
    use moves::Move;

    #[test]
    fn test_size_of_transposition() {
        assert_eq!(mem::size_of::<u64>(),   8); // Hash
        assert_eq!(mem::size_of::<Score>(), 2); // Score
        assert_eq!(mem::size_of::<Move>(),  2); // Move
        assert_eq!(mem::size_of::<u8>(),    1); // Depth

        assert_eq!(mem::size_of::<Transposition>(), 16);
    }

    #[test]
    fn test_transpositions_size() {
        assert_eq!(Transpositions::with_memory(512).size, 32); // 32 == 512 / 16
        assert_eq!(Transpositions::with_capacity(32).size, 32);

        // Size should be a power of two for efficient lookups
        assert_eq!(Transpositions::with_capacity(24).size, 32);

        // Large table of 1 M entries using 16 MB of memory
        assert_eq!(Transpositions::with_memory(16 << 20).size, 1048576);
    }

    #[test]
    fn test_transpositions() {
        let mut tt = Transpositions::with_capacity(1 << 20); // 1 M entries
        
        let h = 42;
        let m = Move::new(E2, E4, DOUBLE_PAWN_PUSH);
        let s = 100;
        let d = 8;
        let b = Bound::Exact;
        let t = Transposition::new(h, d, s, m, b);

        assert_eq!(t.best_move(), m);
        assert_eq!(t.score(), s);
        assert_eq!(t.depth(), d);

        tt.set(h, d, s, m, b);

        assert_eq!(tt.get(&h).unwrap().best_move(), m);

        let h = 1337;
        assert_eq!(tt.get(&h), None);
    }
}
