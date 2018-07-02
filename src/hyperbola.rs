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

fn rank_attacks(occupied: Bitboard, sq: Square) -> Bitboard {
    debug_assert!(sq < OUT);
    let file = sq.file() as Bitboard;
    let rankx8 = (sq.rank() * 8) as Bitboard;
    let occupied = (occupied >> rankx8) & 2 * 63;

    RANK_ATTACKS[(4 * occupied + file) as usize] << rankx8
}

// Initialization of HYPERBOLA_MASKS and RANK_ATTACKS

lazy_static! {
    static ref HYPERBOLA_MASKS: [[Bitboard; 4]; 64] = {
        let mut hyperbola_masks = [[0; 4]; 64];
        for sq in 0..64 {
            hyperbola_masks[sq as usize][HyperbolaMask::File as usize] = genmask(NORTH,     sq) | genmask(SOUTH,     sq);
            hyperbola_masks[sq as usize][HyperbolaMask::Rank as usize] = genmask(EAST,      sq) | genmask(WEST,      sq);
            hyperbola_masks[sq as usize][HyperbolaMask::Diag as usize] = genmask(NORTHEAST, sq) | genmask(SOUTHWEST, sq);
            hyperbola_masks[sq as usize][HyperbolaMask::Anti as usize] = genmask(NORTHWEST, sq) | genmask(SOUTHEAST, sq);
        }
        hyperbola_masks
    };

    static ref RANK_ATTACKS: [Bitboard; 64 * 8] = {
        let mut rank_attacks = [0; 64 * 8];
        for occ in 0..64 {
            for file in 0..8 {
                let i = occ * 8 + file;

                rank_attacks[i] = 0;

                for dest in (file + 1)..8 {
                    rank_attacks[i] |= 1 << dest;
                    if (1 << dest) & (occ << 1) > 0 {
                        break;
                    }
                }
                for dest in (0..file).rev() {
                    rank_attacks[i] |= 1 << dest;
                    if (1 << dest) & (occ << 1) > 0 {
                        break;
                    }
                }
                println!("occ={} file={} i={}", occ, file, i);
                rank_attacks[i].debug()
            }
        }
        rank_attacks
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

fn genmask(dir: Direction, sq: Square) -> Bitboard {
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
