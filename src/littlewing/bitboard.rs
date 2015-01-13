use littlewing::common::*;
use std::num::Int;

pub trait BitboardExt {
    fn shift(&self, x: usize) -> Bitboard;
    fn toggle(&mut self, i: usize); // FIXME: Return instead of update?
    fn set(&mut self, i: usize);
    fn reset(&mut self, i: usize);
    fn get(&self, i: usize) -> bool;
    fn debug(&self);
}

impl BitboardExt for Bitboard {
    fn shift(&self, x: usize) -> Bitboard {
        let v = *self;
        if x < 64 {
            v << x
        } else {
            v >> -x
        }
    }

    fn toggle(&mut self, i: usize) {
        *self ^= 1 << i
    }
    fn set(&mut self, i: usize) {
        *self |= 1 << i
    }
    fn reset(&mut self, i: usize) {
        *self &= !(1 << i)
    }
    fn get(&self, i: usize) -> bool {
        *self & (1 << i) > 0
    }
    fn debug(&self) {
        println!("DEBUG(bitboard): 0x{:016X}", *self);
        for i in range(0, 8) {
            for j in range(0, 8) {
                print!("{:b}", self.get(8 * i + j) as usize);
            }
            println!("");
        }
        println!("");
    }
}

pub fn dumb7fill(mut sliders: Bitboard, empty: Bitboard, dir: usize) -> Bitboard {
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
    use super::BitboardExt;
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
