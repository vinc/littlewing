use littlewing::common::*;
use std::num::Int;

/*
const INDEX64: [uint; 64] = [
     0,  1, 48,  2, 57, 49, 28,  3,
    61, 58, 50, 42, 38, 29, 17,  4,
    62, 55, 59, 36, 53, 51, 43, 22,
    45, 39, 33, 30, 24, 18, 12,  5,
    63, 47, 56, 27, 60, 41, 37, 16,
    54, 35, 52, 21, 44, 32, 23, 11,
    46, 26, 40, 15, 34, 20, 31, 10,
    25, 14, 19,  9, 13,  8,  7,  6
];
*/

pub trait BitwiseOperations {
    fn shift(&self, x: uint) -> Bitboard;
    fn toggle(&mut self, i: uint); // FIXME: Return instead of update?
    fn set(&mut self, i: uint);
    fn reset(&mut self, i: uint);
    fn get(&self, i: uint) -> bool;
    fn ffs(&self) -> uint;
    fn debug(&self);
}

impl BitwiseOperations for Bitboard {
    fn shift(&self, x: uint) -> Bitboard { // FIXME: Use int instead of uint
        let v = *self;
        // (v << x) & (-1u64 >> x) | (v >> -x) & (-1u64 << -x)
        if x as int > 0 {
            v << x
        } else {
            v >> -x
        }
    }

    fn toggle(&mut self, i: uint) {
        *self ^= 1 << i
    }
    fn set(&mut self, i: uint) {
        *self |= 1 << i
    }
    fn reset(&mut self, i: uint) {
        *self &= !(1 << i)
    }
    fn get(&self, i: uint) -> bool {
        *self & (1 << i) > 0
    }
    fn ffs(&self) -> uint {
        /*
        let bb = *self;
        let debruijn64 = 0x03f79d71b4cb0a89u64;
        let i = ((bb & -bb) * debruijn64) >> 58; // Intentional unsigned negation
        INDEX64[i as uint]
        */
        self.trailing_zeros()
    }
    fn debug(&self) {
        //println!("{:016X}", *self);
        //println!("{:064b}", *self);
        println!("DEBUG(bitboard): 0x{:016X}", *self);
        for i in range(0, 8) {
            for j in range(0, 8) {
                print!("{:b}", self.get(8 * i + j) as uint);
            }
            println!("");
        }
        println!("");
    }
}

pub fn dumb7fill(mut sliders: Bitboard, empty: Bitboard, dir: uint) -> Bitboard {
    let mut flood: Bitboard = 0;
    while sliders > 0 {
        flood |= sliders;
        sliders = sliders.shift(dir) & empty;
    }
    flood
}

#[cfg(test)]
mod tests {
    extern crate test;
    use littlewing::common::*;
    use std::num::Int;
    use super::BitwiseOperations;
    use super::dumb7fill;
    use self::test::Bencher;

    #[test]
    fn test_ffs() {
        let bb: Bitboard = 0x0000000000FF0000;
        assert_eq!(bb.ffs(), 16);
    }

    #[test]
    fn test_trailing_zeros() {
        let bb: Bitboard = 0x0000000000FF0000;
        assert_eq!(bb.trailing_zeros(), 16);
    }

    #[bench]
    fn bench_ffs(b: &mut Bencher) {
        let bb: Bitboard = 0x0000000000FF0000;

        b.iter(|| {
            bb.ffs();
        })
    }

    #[bench]
    fn bench_trailing_zeros(b: &mut Bencher) {
        let bb: Bitboard = 0x0000000000FF0000;

        b.iter(|| {
            bb.trailing_zeros();
        })
    }

    /*
    #[test]
    fn test_toggle() {
        let c = WHITE;
        assert!(c.toggle() == BLACK);

        let c = BLACK;
        assert!(c.toggle() == WHITE);
    }
    */

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
}
