use common::*;
use square::*;

pub type Bitboard = u64;

pub trait BitboardExt {
    /// Population count using LLVM `ctpop`
    fn count(&self) -> u32;

    /// Bitscan using LLVM `cttz`
    fn scan(&self) -> u32;

    /// Left shift positive values or right shift negative values
    fn shift(&self, x: Direction) -> Bitboard;

    /// Toggle occupancy bit at the given square
    fn toggle(&mut self, i: Square); // FIXME: Return instead of update?

    /// Set occupancy bit at the given square
    fn set(&mut self, i: Square);

    /// Reset occupancy bit at the given square
    fn reset(&mut self, i: Square);

    /// Get occupancy at the given square
    fn get(&self, i: Square) -> bool;

    fn debug(&self);
    fn to_debug_string(&self) -> String;
}

impl BitboardExt for Bitboard {
    #[inline]
    fn count(&self) -> u32 {
        self.count_ones()
    }

    #[inline]
    fn scan(&self) -> u32 {
        self.trailing_zeros()
    }

    #[inline]
    fn shift(&self, x: Direction) -> Bitboard {
        if x > 0 {
            *self << x
        } else {
            *self >> -x
        }
    }

    #[inline]
    fn toggle(&mut self, i: Square) {
        *self ^= 1 << i
    }

    #[inline]
    fn set(&mut self, i: Square) {
        *self |= 1 << i
    }

    #[inline]
    fn reset(&mut self, i: Square) {
        *self &= !(1 << i)
    }

    #[inline]
    fn get(&self, i: Square) -> bool {
        *self & (1 << i) > 0
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

// Flood fill algorithm
pub fn dumb7fill(mut fill: Bitboard, empty: Bitboard, dir: Direction) -> Bitboard {
    let mut flood: Bitboard = 0;

    while fill > 0 {
        flood |= fill;
        fill = fill.shift(dir) & empty;
    }

    flood
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

#[cfg(test)]
mod tests {
    use common::*;
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
    fn test_dumb7fill() {
        let rooks: Bitboard = 0x0000000000100000;

        let empty: Bitboard = !rooks;
        let targets = dumb7fill(rooks, empty, UP);
        targets.debug();
        let attacks = targets.shift(UP);
        attacks.debug();
        assert_eq!(targets, 0x1010101010100000);

        let empty: Bitboard = !rooks;
        let targets = dumb7fill(rooks, empty, DOWN);
        targets.debug();
        let attacks = targets.shift(DOWN);
        attacks.debug();
        assert_eq!(targets, 0x0000000000101010);

        let empty: Bitboard = !rooks & 0x7F7F7F7F7F7F7F7F;
        let targets = dumb7fill(rooks, empty, RIGHT);
        targets.debug();
        let attacks = targets.shift(RIGHT);
        attacks.debug();
        assert_eq!(targets, 0x0000000000700000);

        let empty: Bitboard = !(rooks | rooks << 16); // With blocker
        let targets = dumb7fill(rooks, empty, UP);
        targets.debug();
        let attacks = targets.shift(UP);
        attacks.debug();
        assert_eq!(targets, 0x0000000010100000);

        let bishop: Bitboard = 0x0000000000100000;
        let empty: Bitboard = !bishop & 0x7F7F7F7F7F7F7F7F;
        let targets = dumb7fill(bishop, empty, UP + RIGHT);
        targets.debug();
        let attacks = targets.shift(UP + RIGHT);
        attacks.debug();
        assert_eq!(targets, 0x0000004020100000);
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
