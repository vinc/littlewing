use std::prelude::v1::*;
use std::cell::UnsafeCell;
use std::mem;
use std::sync::Arc;

use common::*;
use piece_move::PieceMove;
use transposition::{Transposition, Bound};

#[derive(Clone)]
pub struct TranspositionTable {
    entries: Arc<SharedTable>,
    age: u8,
    stats_lookups: u64,
    stats_inserts: u64,
    stats_hits : u64,
    stats_collisions: u64
}

impl TranspositionTable {
    pub fn with_capacity(capacity: usize) -> TranspositionTable {
        let n = if capacity.is_power_of_two() {
            capacity
        } else {
            capacity.next_power_of_two()
        };

        TranspositionTable {
            entries: Arc::new(SharedTable::with_capacity(n)),
            age: 0,
            stats_lookups: 0,
            stats_inserts: 0,
            stats_hits: 0,
            stats_collisions: 0
        }
    }

    pub fn with_memory(memory: usize) -> TranspositionTable {
        let capacity = memory / mem::size_of::<Transposition>();

        TranspositionTable::with_capacity(capacity)
    }

    pub fn get(&mut self, hash: u64) -> Option<&Transposition> {
        self.stats_lookups += 1;

        let h = self.entries.get();
        let n = self.len() as u64;
        let k = (hash % n) as usize; // TODO: hash & (n - 1)
        let t = &h[k]; // TODO: use get_unchecked?

        // TODO: how faster would it be to just also return null move?
        if t.best_move().is_null() {
            None
        } else if t.hash() != hash {
            self.stats_collisions += 1;
            None
        } else {
            debug_assert_eq!(t.hash(), hash);
            self.stats_hits += 1;
            Some(t)
        }
    }

    pub fn set(&mut self, hash: u64, depth: Depth, score: Score, best_move: PieceMove, bound: Bound) {
        let age = self.age;
        let h = self.entries.get();
        let n = self.len() as u64;
        let k = (hash % n) as usize;

        // Always replace entries from previous searches (entry.age < age)
        // but use depth preferred replacement strategy for the current search.
        if age > h[k].age() || (age == 0 && h[k].age() > 0) || depth >= h[k].depth() {
            let t = Transposition::new(hash, depth, score, best_move, bound, age);
            h[k] = t;
            self.stats_inserts += 1;
        }
    }

    pub fn reset(&mut self) {
        self.age = (self.age + 1) % u8::max_value();
        self.clear_stats();
    }

    pub fn clear(&mut self) {
        let n = self.len();
        self.entries = Arc::new(SharedTable::with_capacity(n));
        self.clear_stats();
    }

    fn clear_stats(&mut self) {
        self.stats_lookups = 0;
        self.stats_inserts = 0;
        self.stats_hits = 0;
        self.stats_collisions = 0;
    }

    pub fn len(&self) -> usize {
        self.entries.get().len()
    }

    pub fn memory(&self) -> usize {
        self.len() * mem::size_of::<Transposition>()
    }

    /// Print transposition table stats
    #[cfg(feature = "std")]
    pub fn print_stats(&mut self) {
        // Memory size
        let v = self.memory() as u64;
        let z = (63 - v.leading_zeros()) / 10;
        let size = v / (1 << (10 * z));
        let unit = "KMG".chars().nth((z - 1) as usize).unwrap();

        // Occupacy
        let mut exact_count = 0;
        let mut upper_count = 0;
        let mut lower_count = 0;
        for t in self.entries.get() {
            if t.best_move().is_null() {
                continue;
            }
            match t.bound() {
                Bound::Exact => exact_count += 1,
                Bound::Upper => upper_count += 1,
                Bound::Lower => lower_count += 1,
            }
        }

        // Usage
        let collisions = self.stats_collisions;
        let hits = self.stats_hits;
        let miss = self.stats_lookups - hits - collisions;

        println!("# {:15} {:>8} ({} {}B)", "tt size:", self.len(), size, unit);

        let percent = (lower_count as f64) * 100.0 / (self.len() as f64);
        println!("# {:15} {:>8} ({:.2} %)", " - lower:", lower_count, percent);
        let percent = (upper_count as f64) * 100.0 / (self.len() as f64);
        println!("# {:15} {:>8} ({:.2} %)", " - upper:", upper_count, percent);
        let percent = (exact_count as f64) * 100.0 / (self.len() as f64);
        println!("# {:15} {:>8} ({:.2} %)", " - exact:", exact_count, percent);

        println!("# {:15} {:>8}", "tt inserts:", self.stats_inserts);
        println!("# {:15} {:>8}", "tt lookups:", self.stats_lookups);

        let percent = (miss as f64) * 100.0 / (self.stats_lookups as f64);
        println!("# {:15} {:>8} ({:.2} %)", " - miss:", miss, percent);
        let percent = (hits as f64) * 100.0 / (self.stats_lookups as f64);
        println!("# {:15} {:>8} ({:.2} %)", " - hits:", hits, percent);
        let percent = (collisions as f64) * 100.0 / (self.stats_lookups as f64);
        println!("# {:15} {:>8} ({:.2} %)", " - collisions:", collisions, percent);
    }
}

pub struct SharedTable {
    inner: UnsafeCell<Box<[Transposition]>>
}

// Tell the compiler than the transposition table can be shared between
// threads inside an `Arc`, even if it's not really safe at all in reality :)
unsafe impl Sync for SharedTable {}

impl SharedTable {
    pub fn with_capacity(capacity: usize) -> SharedTable {
        SharedTable {
            // NOTE: Transmuting a boxed slice of zeroed 128 bit integers into
            // empty transpositions is much faster than creating a boxed slice
            // of transitions directly.
            // inner: UnsafeCell::new(vec![Transposition::new_null(); capacity].into_boxed_slice())
            inner: UnsafeCell::new(unsafe {
                mem::transmute::<Box<[u128]>, Box<[Transposition]>>(
                    vec![0u128; capacity].into_boxed_slice()
                )
            })
        }
    }

    // FIXME: mutable borrow from immutable input
    pub fn get(&self) -> &mut [Transposition] {
        unsafe { &mut *self.inner.get() }
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "std")]
    use std::sync::{Arc, Barrier};
    #[cfg(feature = "std")]
    use std::thread;

    use super::*;
    use square::*;
    use piece_move::PieceMove;

    #[test]
    fn test_transposition_table_size() {
        assert_eq!(TranspositionTable::with_memory(512).len(), 32); // 512 / 16 == 32
        assert_eq!(TranspositionTable::with_capacity(32).len(), 32);

        // Size should be a power of two for efficient lookups
        assert_eq!(TranspositionTable::with_capacity(24).len(), 32);

        // Large table of 1 M entries using 16 MB of memory
        assert_eq!(TranspositionTable::with_memory(16 << 20).len(), 1048576);
    }

    #[test]
    fn test_transposition_table() {
        let mut tt = TranspositionTable::with_capacity(1 << 20); // 1 M entries

        let h = 42;
        let m = PieceMove::new(E2, E4, DOUBLE_PAWN_PUSH);
        let s = 100;
        let d = 8;
        let b = Bound::Exact;
        let a = 0;
        let t = Transposition::new(h, d, s, m, b, a);

        assert_eq!(t.best_move(), m);
        assert_eq!(t.score(), s);
        assert_eq!(t.depth(), d);

        tt.set(h, d, s, m, b);

        assert_eq!(tt.get(h).unwrap().best_move(), m);

        let h = 1337;
        assert_eq!(tt.get(h), None);
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_transposition_table_in_threads() {
        // Transposition content
        let h = 42;
        let m = PieceMove::new(E2, E4, DOUBLE_PAWN_PUSH);
        let s = 100;
        let d = 8;
        let b = Bound::Exact;

        let n = 4;
        let mut children = Vec::with_capacity(n);
        let shared_tt = TranspositionTable::with_memory(1 << 20);
        let barrier = Arc::new(Barrier::new(n));
        for i in 0..n {
            let mut tt = shared_tt.clone();
            let c = barrier.clone();

            children.push(thread::spawn(move || {
                if i == 0 {
                    tt.set(h, d, s, m, b); // First thread set a value in TT
                }
                c.wait(); // Synchronize all threads
                tt.get(h).unwrap().best_move() // All threads should get it
            }));
        }

        for child in children {
            assert_eq!(child.join().unwrap(), m);
        }
    }
}
