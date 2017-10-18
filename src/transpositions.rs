use std::mem;
use std::cell::UnsafeCell;
use std::sync::Arc;

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
    depth: Depth,    //  8 bits => 1 bytes
    bound: Bound,    //  8 bits => 1 bytes

    // Total: 15 bytes, which will use 16 bytes including alignment padding.

    // NOTE: `depth` will never go above MAX_PLY, which is 128 so we can store
    // it as `i8`.
    //
    // TODO: we don't need to store the whole hash as the first part is the
    // index of the entry: `entries[hash % size]`
    //
    // TODO: add age counter incremented at the begining of each root search
    // to be used to replace old entries from previous search without having
    // to clear the whole table.
}

impl Transposition {
    pub fn new(hash: u64, depth: Depth, score: Score, best_move: Move, bound: Bound) -> Transposition {
        Transposition {
            hash: hash,
            depth: depth,
            score: score,
            best_move: best_move,
            bound: bound
        }
    }

    pub fn new_null() -> Transposition {
        Transposition::new(0, 0, 0, Move::new_null(), Bound::Exact)
    }

    pub fn depth(&self) -> Depth {
        self.depth as Depth
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
    entries: Arc<SharedTable>,
    stats_lookups: u64,
    stats_inserts: u64,
    stats_hits : u64,
    stats_collisions: u64
}

impl Transpositions {
    pub fn with_capacity(capacity: usize) -> Transpositions {
        let n = if capacity.is_power_of_two() {
            capacity
        } else {
            capacity.next_power_of_two()
        };

        Transpositions {
            entries: Arc::new(SharedTable::with_capacity(n)),
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
        let entries = self.entries.get();
        self.stats_lookups += 1;

        let n = self.len() as u64;
        let k = (hash % n) as usize; // TODO: hash & (n - 1)
        let t = &entries[k]; // TODO: use get_unchecked?

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

    pub fn set(&mut self, hash: u64, depth: Depth, score: Score, best_move: Move, bound: Bound) {
        let entries = self.entries.get();
        let n = self.len() as u64;
        let k = (hash % n) as usize;

        // NOTE: replacement strategies:
        // 1. Always replace
        // 2. Depth prefered
        if depth >= entries[k].depth() { // Using "depth prefered"
            let t = Transposition::new(hash, depth, score, best_move, bound);
            entries[k] = t;
            self.stats_inserts += 1;
        }
    }

    pub fn clear(&mut self) {
        let capacity = self.len();
        self.entries = Arc::new(SharedTable::with_capacity(capacity));

        self.stats_lookups = 0;
        self.stats_inserts = 0;
        self.stats_hits = 0;
        self.stats_collisions = 0;
    }

    pub fn len(&self) -> usize {
        self.entries.get().len()
    }

    pub fn print_stats(&mut self) {
        println!("# {:15} {}", "tt size:", self.len());
        println!("# {:15} {}", "tt inserts:", self.stats_inserts);
        println!("# {:15} {}", "tt lookups:", self.stats_lookups);
        println!("# {:15} {}", "tt hits:", self.stats_hits);
        println!("# {:15} {}", "tt collisions:", self.stats_collisions);
    }
}

pub struct SharedTable {
    inner: UnsafeCell<Box<[Transposition]>>
}

unsafe impl Sync for SharedTable {}

impl SharedTable {
    pub fn with_capacity(capacity: usize) -> SharedTable {
        SharedTable {
            inner: UnsafeCell::new(vec![Transposition::new_null(); capacity].into_boxed_slice())
        }
    }

    pub fn get(&self) -> &mut Box<[Transposition]> {
        unsafe { &mut *self.inner.get() }
    }
}

#[cfg(test)]
mod tests {
    use std::mem;
    use std::sync::{Arc, Barrier};
    use std::thread;

    use super::*;
    use square::*;
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
        assert_eq!(Transpositions::with_memory(512).len(), 32); // 32 == 512 / 16
        assert_eq!(Transpositions::with_capacity(32).len(), 32);

        // Size should be a power of two for efficient lookups
        assert_eq!(Transpositions::with_capacity(24).len(), 32);

        // Large table of 1 M entries using 16 MB of memory
        assert_eq!(Transpositions::with_memory(16 << 20).len(), 1048576);
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

    #[test]
    fn test_transpositions_in_threads() {
        // Transposition content
        let h = 42;
        let m = Move::new(E2, E4, DOUBLE_PAWN_PUSH);
        let s = 100;
        let d = 8;
        let b = Bound::Exact;

        let n = 4;
        let mut children = Vec::with_capacity(n);
        let shared_tt = Transpositions::with_memory(1 << 20);
        let barrier = Arc::new(Barrier::new(n));
        for i in 0..n {
            let mut tt = shared_tt.clone();
            let c = barrier.clone();

            children.push(thread::spawn(move || {
                if i == 0 {
                    tt.set(h, d, s, m, b); // First thread set a value in TT
                }
                c.wait(); // Synchronize all threads
                tt.get(&h).unwrap().best_move() // All threads should get it
            }));
        }

        for child in children {
            assert_eq!(child.join().unwrap(), m);
        }
    }
}
