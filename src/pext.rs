use std::arch::x86_64::_pext_u64;

use crate::square::*;
use crate::bitboard::Bitboard;
use crate::hyperbola;

pub fn bishop_attacks(from: Square, occupied: Bitboard) -> Bitboard {
    let i = 64 + from as usize;
    let j = pext(occupied, BISHOP_MASKS[from as usize]);
    ATTACKS[OFFSETS[i] + j]
}

pub fn rook_attacks(from: Square, occupied: Bitboard) -> Bitboard {
    let i = from as usize;
    let j = pext(occupied, ROOK_MASKS[from as usize]);
    ATTACKS[OFFSETS[i] + j]
}

fn pext(occupied: Bitboard, mask: u64) -> usize {
    unsafe {
        _pext_u64(occupied as u64, mask) as usize
    }
}

const SIZE: usize = 173568;

lazy_static! {
    static ref ROOK_MASKS: [u64; 64] = {
        let mut mask = [0; 64];
        for sq in 0..64 {
            mask[sq] = hyperbola::rook_mask(sq as Square);
        }
        mask
    };

    static ref BISHOP_MASKS: [u64; 64] = {
        let mut mask = [0; 64];
        for sq in 0..64 {
            mask[sq] = hyperbola::bishop_mask(sq as Square);
        }
        mask
    };

    static ref OFFSETS: [usize; 128] = {
        let mut offsets = [0; 128];
        let mut offset = 0;
        for i in 0..64 {
            offsets[i] = offset;
            offset += 1 << ROOK_MASKS[i].count_ones();
        }
        for i in 64..128 {
            offsets[i] = offset;
            offset += 1 << BISHOP_MASKS[i - 64].count_ones();
        }
        assert_eq!(SIZE, offset);
        offsets
    };

    //static ref ATTACKS: Vec<Bitboard> = {
    //    let mut table = vec![0; SIZE];
    static ref ATTACKS: [Bitboard; SIZE] = {
        let mut table = [0; SIZE];
        for sq in 0..64 {
            // Carry-Rippler: enumerate all subsets of mask
            let mask = ROOK_MASKS[sq];
            let mut occupied = 0u64;
            loop {
                let index = OFFSETS[sq] + pext(occupied, mask);
                let attacks = hyperbola::rook_attacks(sq as Square, occupied);
                table[index] = attacks;

                occupied = occupied.wrapping_sub(mask) & mask;
                if occupied == 0 {
                    break;
                }
            }

            let mask = BISHOP_MASKS[sq];
            let mut occupied = 0u64;
            loop {
                let index = OFFSETS[sq + 64] + pext(occupied, mask);
                let attacks = hyperbola::bishop_attacks(sq as Square, occupied);
                table[index] = attacks;

                occupied = occupied.wrapping_sub(mask) & mask;
                if occupied == 0 {
                    break;
                }
            }
        }
        table
    };
}
