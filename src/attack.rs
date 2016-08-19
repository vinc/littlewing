use common::*;
use bitboard::BitboardExt;
use bitboard::dumb7fill;
use game::Game;
use piece::PieceAttr;

pub trait Attack {
    fn is_check(&self, side: Color) -> bool;
    fn is_attacked(&self, square: Square, side: Color) -> bool;
}

impl Attack for Game {
    fn is_check(&self, side: Color) -> bool {
        let king = self.bitboards[(side | KING) as usize];
        if king == 0 {
            return true; // FIXME: Obviously...
        }
        let square = king.trailing_zeros() as Square;
        self.is_attacked(square, side)
    }
    fn is_attacked(&self, square: Square, side: Color) -> bool {
        let occupied = self.bitboards[WHITE as usize] | self.bitboards[BLACK as usize];

        let pawns = self.bitboards[(side ^ 1 | PAWN) as usize];
        let attacks = PAWN_ATTACKS[side as usize][square as usize];
        if attacks & pawns > 0 {
            return true;
        }

        let knights = self.bitboards[(side ^ 1 | KNIGHT) as usize];
        let attacks = PIECE_MASKS[KNIGHT as usize][square as usize];
        if attacks & knights > 0 {
            return true;
        }

        let king = self.bitboards[(side ^ 1 | KING) as usize];
        let attacks = PIECE_MASKS[KING as usize][square as usize];
        if attacks & king > 0 {
            return true;
        }

        let queens = self.bitboards[(side ^ 1 | QUEEN) as usize];

        let bishops = self.bitboards[(side ^ 1 | BISHOP) as usize];
        let attacks = bishop_attacks(square, occupied);
        if attacks & (bishops | queens) > 0 {
            return true;
        }

        let rooks = self.bitboards[(side ^ 1 | ROOK) as usize];
        let attacks = rook_attacks(square, occupied);
        if attacks & (rooks | queens) > 0 {
            return true;
        }

        false
    }
}

pub fn attacks(piece: Piece, square: Square, occupied: Bitboard) -> Bitboard {
    match piece.kind() {
        PAWN => PAWN_ATTACKS[piece.color() as usize][square as usize],
        KNIGHT => PIECE_MASKS[KNIGHT as usize][square as usize],
        KING => PIECE_MASKS[KING as usize][square as usize],
        BISHOP => bishop_attacks(square, occupied),
        ROOK => rook_attacks(square, occupied),
        QUEEN => bishop_attacks(square, occupied) | rook_attacks(square, occupied),
        _ => panic!("wrong kind of piece") // FIXME
    }
}

pub fn bishop_attacks(from: Square, occupied: Bitboard) -> Bitboard {
    let mut targets = 0;

    let occluded = dumb7fill(1 << from, !occupied & 0x7F7F7F7F7F7F7F7F, UP + LEFT);
    targets |= 0x7F7F7F7F7F7F7F7F & occluded.shift(UP + LEFT);
    let occluded = dumb7fill(1 << from, !occupied & 0x7F7F7F7F7F7F7F7F, DOWN + LEFT);
    targets |= 0x7F7F7F7F7F7F7F7F & occluded.shift(DOWN + LEFT);
    let occluded = dumb7fill(1 << from, !occupied & 0xFEFEFEFEFEFEFEFE, DOWN + RIGHT);
    targets |= 0xFEFEFEFEFEFEFEFE & occluded.shift(DOWN + RIGHT);
    let occluded = dumb7fill(1 << from, !occupied & 0xFEFEFEFEFEFEFEFE, UP + RIGHT);
    targets |= 0xFEFEFEFEFEFEFEFE & occluded.shift(UP + RIGHT);

    targets
}

pub fn rook_attacks(from: Square, occupied: Bitboard) -> Bitboard {
    let mut targets = 0;

    let occluded = dumb7fill(1 << from, !occupied & 0xFFFFFFFFFFFFFFFF, UP);
    targets |= 0xFFFFFFFFFFFFFFFF & occluded.shift(UP);
    let occluded = dumb7fill(1 << from, !occupied & 0xFFFFFFFFFFFFFFFF, DOWN);
    targets |= 0xFFFFFFFFFFFFFFFF & occluded.shift(DOWN);
    let occluded = dumb7fill(1 << from, !occupied & 0x7F7F7F7F7F7F7F7F, LEFT);
    targets |= 0x7F7F7F7F7F7F7F7F & occluded.shift(LEFT);
    let occluded = dumb7fill(1 << from, !occupied & 0xFEFEFEFEFEFEFEFE, RIGHT);
    targets |= 0xFEFEFEFEFEFEFEFE & occluded.shift(RIGHT);

    targets
}

lazy_static! {
    pub static ref PAWN_ATTACKS: [[Bitboard; 64]; 2] = {
        let xdirs = [LEFT, RIGHT];
        let ydirs = [DOWN, UP];
        let files = [FILE_H, FILE_A];
        let mut attacks = [[0; 64]; 2];
        for side in 0..2 {
            for square in 0..64 {
                for i in 0..2 {
                    let dir = ydirs[side ^ 1] + xdirs[i];
                    attacks[side][square] |= (1 << square).shift(dir) & !files[i];
                }
            }
        }
        attacks
    };
}

/*
#[cfg(test)]
mod tests {
    extern crate test;

    //use self::test::Bencher;
    use common::*;
    use attack::{bishop_attacks, rook_attacks};

    #[bench]
    fn bench_bishop_attacks(b: &mut Bencher) {
        b.iter(|| {
            bishop_attacks(E4, 0u64)
        })
    }

    #[bench]
    fn bench_rook_attacks(b: &mut Bencher) {
        b.iter(|| {
            rook_attacks(E4, 0u64)
        })
    }
}
*/
