use std::prelude::v1::*;

use common::*;
use color::*;
use square::*;
use bitboard::{Bitboard, BitboardExt};

pub fn bishop_attacks(from: Square, occupied: Bitboard) -> Bitboard {
    hyperbola(occupied, from, HyperbolaMask::Diag) |
    hyperbola(occupied, from, HyperbolaMask::Anti)
}

pub fn rook_attacks(from: Square, occupied: Bitboard) -> Bitboard {
    hyperbola(occupied, from, HyperbolaMask::File) |
    rank_attacks(occupied, from)
}

#[repr(usize)]
enum HyperbolaMask { File, Rank, Diag, Anti }

// Hyperbola Quintessence
fn hyperbola(occupied: Bitboard, sq: Square, t: HyperbolaMask) -> Bitboard {
    debug_assert!(sq < OUT);
    let mask = HYPERBOLA_MASKS[sq as usize][t as usize];
    let mut forward = occupied & mask;
    let mut reverse = forward.swap_bytes();
    //forward -= 1 << sq;
    //reverse -= 1 << sq.flip(BLACK);
    forward = forward.wrapping_sub(Bitboard::from_square(sq));
    reverse = reverse.wrapping_sub(Bitboard::from_square(sq.flip(BLACK)));
    forward ^= reverse.swap_bytes();
    forward & mask
}

// First Rank Attacks
fn rank_attacks(occupied: Bitboard, sq: Square) -> Bitboard {
    debug_assert!(sq < OUT);
    let f = sq & 7; // sq.file() as Bitboard;
    let r = sq & !7; // (sq.rank() * 8) as Bitboard;
    let o = (occupied >> (r + 1)) & 63;
    FIRST_RANK_ATTACKS[o as usize][f as usize] << r
}

lazy_static! {
    static ref HYPERBOLA_MASKS: [[Bitboard; 4]; 64] = {
        let mut hyperbola_masks = [[0; 4]; 64];
        for sq in 0..64 {
            let mask = &mut hyperbola_masks[sq as usize];
            mask[HyperbolaMask::File as usize] = generate_mask(NORTH,     sq) | generate_mask(SOUTH,     sq);
            mask[HyperbolaMask::Rank as usize] = generate_mask(EAST,      sq) | generate_mask(WEST,      sq);
            mask[HyperbolaMask::Diag as usize] = generate_mask(NORTHEAST, sq) | generate_mask(SOUTHWEST, sq);
            mask[HyperbolaMask::Anti as usize] = generate_mask(NORTHWEST, sq) | generate_mask(SOUTHEAST, sq);
        }
        hyperbola_masks
    };

    static ref FIRST_RANK_ATTACKS: [[Bitboard; 8]; 64] = {
        let mut first_rank_attacks = [[0; 8]; 64];
        for o in 0..64 {
            for f in 0..8 {
                first_rank_attacks[o][f] = 0;

                for i in (f + 1)..8 {
                    first_rank_attacks[o][f] |= 1 << i;
                    if (o << 1) & (1 << i) > 0 {
                        break;
                    }
                }
                for i in (0..f).rev() {
                    first_rank_attacks[o][f] |= 1 << i;
                    if (o << 1) & (1 << i) > 0 {
                        break;
                    }
                }
            }
        }

        first_rank_attacks
    };
}

fn is_out_rank(dir: Direction, sq: Square) -> bool {
    let crossed_north = dir.is_north() && sq.rank() == 7;
    let crossed_south = dir.is_south() && sq.rank() == 0;
    crossed_north || crossed_south
}

fn is_out_file(dir: Direction, sq: Square) -> bool {
    let crossed_west = dir.is_west() && sq.file() == 0;
    let crossed_east = dir.is_east() && sq.file() == 7;
    crossed_west || crossed_east
}

fn generate_mask(dir: Direction, sq: Square) -> Bitboard {
    debug_assert!(sq < OUT);
    let shift = DIRECTION_SHIFTS[dir];
    let mut bb = 0;
    let mut dest = ((sq as Shift) + shift) as Square;
    loop {
        if is_out_file(dir, dest) || dest >= OUT {
            break;
        }
        bb |= Bitboard::from_square(dest);
        if is_out_rank(dir, dest) {
            break;
        }
        dest = ((dest as Shift) + shift) as Square;
    }
    bb
}
