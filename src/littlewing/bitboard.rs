pub type Bitboard = u64;

pub trait Bitwise {
    fn set(&mut self, i: uint);
    fn get(&self, i: uint) -> bool;
}

impl Bitwise for Bitboard {
    fn set(&mut self, i: uint) {
        *self |= 1 << i
    }
    fn get(&self, i: uint) -> bool {
        *self & (1 << i) > 0
    }
}
