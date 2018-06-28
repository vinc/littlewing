use common::*;
use square::*;

pub type Bitboard = u64;

pub trait BitboardExt {
    fn from_square(sq: Square) -> Bitboard;

    /// Population count using LLVM `ctpop`
    fn count(self) -> u32;

    /// Bitscan using LLVM `cttz`
    fn scan(self) -> u32;

    /// Generalized shift (left shift positive values and right shift negative values)
    fn shift(self, s: Shift) -> Bitboard;

    /// Get occupancy at the given square
    fn get(self, sq: Square) -> bool;

    /// Set occupancy bit at the given square
    fn set(&mut self, sq: Square);

    /// Toggle occupancy bit at the given square
    fn toggle(&mut self, sq: Square); // FIXME: Return instead of update?

    /// Reset occupancy bit at the given square
    fn reset(&mut self, sq: Square);

    fn debug(&self);
    fn to_debug_string(&self) -> String;
}

impl BitboardExt for Bitboard {
    fn from_square(sq: Square) -> Bitboard {
        //BB_SQUARES[sq as usize]
        //unsafe { *BB_SQUARES.get_unchecked(sq as usize) }
        1 << sq
    }

    #[inline]
    fn count(self) -> u32 {
        self.count_ones()
    }

    #[inline]
    fn scan(self) -> u32 {
        self.trailing_zeros()
    }

    #[inline]
    fn shift(self, s: Shift) -> Bitboard {
        if s > 0 {
            self << s
        } else {
            self >> -s
        }
    }

    #[inline]
    fn get(self, sq: Square) -> bool {
        (self & Bitboard::from_square(sq)) > 0
    }

    #[inline]
    fn set(&mut self, sq: Square) {
        *self |= Bitboard::from_square(sq)
    }

    #[inline]
    fn toggle(&mut self, sq: Square) {
        *self ^= Bitboard::from_square(sq)
    }

    #[inline]
    fn reset(&mut self, sq: Square) {
        *self &= !Bitboard::from_square(sq)
    }

    //FIXME: remove this method
    //#[deprecated(since="0.2.0", note="please use `to_debug_string` instead")]
    fn debug(&self) {
        println!("{}", self.to_debug_string());
    }

    fn to_debug_string(&self) -> String {
        let mut out = String::new();

        out.push_str(&format!("DEBUG(bitboard): 0x{:016X}\n", *self));

        out.push_str("+--------+\n");
        for i in 0..8 {
            out.push_str("|");
            for j in 0..8 {
                let s = 8 * i + j;
                out.push_str(&format!("{:b}", self.get(s) as usize));
            }
            out.push_str(&format!("|{}\n", i + 1));
        }
        out.push_str("+--------+\n abcdefgh\n");

        out
    }
}

pub fn upfill(mut pieces: Bitboard) -> Bitboard {
    pieces |= pieces << 8;
    pieces |= pieces << 16;
    pieces |= pieces << 32;
    pieces
}

pub fn downfill(mut pieces: Bitboard) -> Bitboard {
    pieces |= pieces >> 8;
    pieces |= pieces >> 16;
    pieces |= pieces >> 32;
    pieces
}

pub fn filefill(pieces: Bitboard) -> Bitboard {
    upfill(pieces) | downfill(pieces)
}

pub trait BitboardIterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
}

impl BitboardIterator for Bitboard {
    type Item = Square;

    fn next(&mut self) -> Option<Square> {
        if *self > 0 {
            let sq = self.scan() as Square;

            self.reset(sq);

            Some(sq)
        } else {
            None
        }
    }
}

/*
lazy_static! {
    static ref BB_SQUARES: [Bitboard; 64] = {
        let mut bb_squares = [0; 64];
        for sq in 0..64 {
            bb_squares[sq] = 1 << sq;
        }
        bb_squares
    };
}
*/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shift() {
        let bb1: Bitboard = 0b0011000000000000000000000000000000110000000000000000000000000000;
        let bb2: Bitboard = 0b1100000000000000000000000000000011000000000000000000000000000011;
        let bb3: Bitboard = 0b0000000000000000000000000000001100000000000000000000000000001100;

        assert_eq!(bb2.shift(0),  bb2, "{:b} == {:b}", bb2, bb2);
        assert_eq!(bb2 << 2,      bb3, "{:b} == {:b}", bb2, bb3);
        assert_eq!(bb2.shift(2),  bb3, "{:b} == {:b}", bb2, bb3);
        assert_eq!(bb2 >> 2,      bb1, "{:b} == {:b}", bb2, bb1);
        assert_eq!(bb2.shift(-2), bb1, "{:b} == {:b}", bb2, bb1);
    }

    #[test]
    fn test_iterator() {
        let mut bb: Bitboard = 0;

        bb.set(A2);
        bb.set(B2);
        bb.set(C2);
        bb.set(D2);

        assert_eq!(bb.next(), Some(A2));
        assert_eq!(bb.next(), Some(B2));
        assert_eq!(bb.next(), Some(C2));
        assert_eq!(bb.next(), Some(D2));
        assert_eq!(bb.next(), None);
    }
}
