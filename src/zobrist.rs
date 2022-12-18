use std::prelude::v1::*;

use rand::{RngCore, SeedableRng};
use rand_xorshift::XorShiftRng;

use crate::color::Color;
use crate::piece::Piece;

#[derive(Clone)]
pub struct Zobrist {
    pub pieces: [[u64; 64]; 14],
    pub en_passant: [u64; 64],
    pub castling_rights: [[u64; 2]; 2],
    pub side: u64
}

const SEED: [u8; 16] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];

impl Zobrist {
    pub fn new() -> Zobrist {
        let mut zobrist = Zobrist {
            pieces: [[0; 64]; 14],
            en_passant: [0; 64],
            castling_rights: [[0; 2]; 2],
            side: 0
        };

        let mut rng = XorShiftRng::from_seed(SEED);

        for i in 0..14 {
            for j in 0..64 {
                zobrist.pieces[i][j] = rng.next_u64();
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

    pub fn castling_right(&self, side: Color, wing: Piece) -> u64 {
        self.castling_rights[side as usize][(wing >> 3) as usize]
    }
}

#[cfg(test)]
mod tests {
    use crate::zobrist::Zobrist;

    #[test]
    fn test_new() {
        let zobrist = Zobrist::new();
        assert!(zobrist.pieces[0][0] != zobrist.pieces[7][42]);
    }
}
