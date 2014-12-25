use littlewing::common::*;

const INDEX64: [uint, ..64] = [
     0,  1, 48,  2, 57, 49, 28,  3,
    61, 58, 50, 42, 38, 29, 17,  4,
    62, 55, 59, 36, 53, 51, 43, 22,
    45, 39, 33, 30, 24, 18, 12,  5,
    63, 47, 56, 27, 60, 41, 37, 16,
    54, 35, 52, 21, 44, 32, 23, 11,
    46, 26, 40, 15, 34, 20, 31, 10,
    25, 14, 19,  9, 13,  8,  7,  6
];

pub trait BitwiseOperations {
    fn set(&mut self, i: uint);
    fn reset(&mut self, i: uint);
    fn get(&self, i: uint) -> bool;
    fn ffs(&self) -> uint;
    fn debug(&self);
}

impl BitwiseOperations for Bitboard {
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
        let bb = *self;
        let debruijn64 = 0x03f79d71b4cb0a89u64;
        let i = ((bb & -bb) * debruijn64) >> 58;
        INDEX64[i as uint]
    }
    fn debug(&self) {
        //println!("{:016X}", self);
        //println!("{:064b}", self);
        println!("DEBUG(bitboard)");
        for i in range(0, 8) {
            for j in range(0, 8) {
                print!("{:b}", self.get(8 * i + j) as uint);
            }
            println!("");
        }
        println!("");
    }
}
