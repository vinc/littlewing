use common::*;
use moves::Move;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Transposition {
    hash: u64,       // 64 bits => 8 bytes
    best_move: Move, // 16 bits => 2 bytes
    score: Score,    // 16 bits => 2 bytes
    depth: u8        //  8 bits => 1 bytes

    // Total: 13 bytes, which will use 16 bytes including alignment padding.

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
    pub fn new(hash: u64, best_move: Move, score: Score, depth: usize) -> Transposition {
        Transposition {
            hash: hash,
            best_move: best_move,
            score: score,
            depth: depth as u8
        }
    }

    pub fn new_null() -> Transposition {
        Transposition::new(0, Move::new_null(), 0, 0)
    }

    pub fn depth(&self) -> usize {
        self.depth as usize
    }
    pub fn best_move(&self) -> Move {
        self.best_move
    }
    pub fn score(&self) -> Score {
        self.score
    }
}

pub struct Transpositions {
    pub entries: Box<[Transposition]>,
    pub size: usize,
    pub stats_lookups: u64,
    pub stats_inserts: u64,
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
            stats_collisions: 0
        }
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
            Some(t)
        }
    }

    pub fn set(&mut self, hash: u64, best_move: Move, score: Score, depth: usize) {
        self.stats_inserts += 1;

        let t = Transposition::new(hash, best_move, score, depth);
        let n = self.size as u64;
        let k = (hash % n) as usize;

        // NOTE: replacement strategies:
        // 1. Always replace
        // 2. Depth prefered
        if self.entries[k].depth() < depth { // Using "depth prefered"
            self.entries[k] = t;
        }
    }

    pub fn clear(&mut self) {
        let capacity = self.size;
        self.entries = vec![Transposition::new_null(); capacity].into_boxed_slice();

        self.stats_lookups = 0;
        self.stats_inserts = 0;
        self.stats_collisions = 0;
    }

    pub fn print_stats(&mut self) {
        println!("# tt size:       {}", self.entries.len());
        println!("# tt lookups:    {}", self.stats_lookups);
        println!("# tt inserts:    {}", self.stats_inserts);
        println!("# tt collisions: {}", self.stats_collisions);
    }
}

#[cfg(test)]
mod tests {
    use std::mem;

    use super::*;
    use common::*;
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
        // Size should be a power of two for efficient lookups
        assert_eq!(Transpositions::with_capacity(32).size, 32);
        assert_eq!(Transpositions::with_capacity(24).size, 32);
    }

    #[test]
    fn test_transpositions() {
        let mut tt = Transpositions::with_capacity(1 << 20); // 1 Mb
        
        let h = 42;
        let m = Move::new(E2, E4, DOUBLE_PAWN_PUSH);
        let s = 100;
        let d = 8;
        let t = Transposition::new(h, m, s, d);

        assert_eq!(t.best_move(), m);
        assert_eq!(t.score(), s);
        assert_eq!(t.depth(), d);

        tt.set(h, m, s, d);

        assert_eq!(tt.get(&h).unwrap().best_move(), m);

        let h = 1337;
        assert_eq!(tt.get(&h), None);
    }
}
