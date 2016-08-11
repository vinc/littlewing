use std::collections::HashMap;

use moves::Move;

pub struct Transposition {
    pub hash: u64,
    pub best_move: Move,
    pub score: i32,
    pub depth: usize
}

impl Transposition {
    pub fn new(hash: u64, best_move: Move, score: i32, depth: usize) -> Transposition {
        Transposition {
            hash: hash,
            best_move: best_move,
            score: score,
            depth: depth
        }
    }
}

pub struct Transpositions {
    pub entries: HashMap<u64, Transposition>,
    pub stats_lookup: u64,
    pub stats_insert: u64
}

impl Transpositions {
    pub fn with_capacity(capacity: usize) -> Transpositions {
        Transpositions {
            entries: HashMap::with_capacity(capacity),
            stats_lookup: 0,
            stats_insert: 0
        }
    }

    pub fn get(&mut self, hash: &u64) -> Option<&Transposition> {
        self.stats_lookup += 1;
        self.entries.get(&hash)
    }

    pub fn set(&mut self, hash: u64, best_move: Move, score: i32, depth: usize) {
        let t = Transposition::new(hash, best_move, score, depth);

        self.stats_insert += 1;
        self.entries.insert(hash, t);
    }

    pub fn clear_stats(&mut self) {
        self.stats_lookup = 0;
        self.stats_insert = 0;
    }

    pub fn print_stats(&mut self) {
        println!("# tt size:    {}", self.entries.len());
        println!("# tt lookups: {}", self.stats_lookup);
        println!("# tt inserts: {}", self.stats_insert);
    }
}
