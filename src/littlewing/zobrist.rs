extern crate rand;

use self::rand::Rng;
use self::rand::weak_rng;

pub struct Zobrist {
    pub positions: [[u64; 64]; 14],
    pub en_passant: [u64; 64],
    pub castling_rights: [[u64; 2]; 2],
    pub side: u64
}

impl Zobrist {
    pub fn new() -> Zobrist {
        let mut zobrist = Zobrist {
            positions: [[0; 64]; 14],
            en_passant: [0; 64],
            castling_rights: [[0; 2]; 2],
            side: 0
        };

        let mut rng = weak_rng();
        for i in 0..14 {
            for j in 0..64 {
                zobrist.positions[i][j] = rng.next_u64();
            }
        }
        for i in 0..64 {
            zobrist.en_passant[i] = rng.next_u64();
        }
        for i in 0..2 {
            for j in 0..2 {
                zobrist.castling_rights[i][j] = rng.next_u64();
            }
        }
        zobrist.side = rng.next_u64();

        zobrist
    }
}

#[cfg(test)]
mod tests {
    use littlewing::zobrist::Zobrist;

    #[test]
    fn test_new() {
        let zobrist = Zobrist::new();
        assert!(zobrist.positions[0][0] != zobrist.positions[7][42]);
    }
}
