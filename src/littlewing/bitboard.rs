use littlewing::common::*;

pub trait BitboardExt {
    fn shift(&self, x: Direction) -> Bitboard;
    fn toggle(&mut self, i: Square); // FIXME: Return instead of update?
    fn set(&mut self, i: Square);
    fn reset(&mut self, i: Square);
    fn get(&self, i: Square) -> bool;
    fn debug(&self);
}

impl BitboardExt for Bitboard {
    fn shift(&self, x: Direction) -> Bitboard {
        let v = *self;
        if x > 0 {
            v << x
        } else {
            v >> -x
        }
    }

    fn toggle(&mut self, i: Square) {
        *self ^= 1 << i
    }
    fn set(&mut self, i: Square) {
        *self |= 1 << i
    }
    fn reset(&mut self, i: Square) {
        *self &= !(1 << i)
    }
    fn get(&self, i: Square) -> bool {
        *self & (1 << i) > 0
    }
    fn debug(&self) {
        println!("DEBUG(bitboard): 0x{:016X}", *self);
        for i in 0..8 {
            for j in 0..8 {
                print!("{:b}", self.get(8 * i + j) as usize);
            }
            println!("");
        }
        println!("");
    }
}

pub fn dumb7fill(mut sliders: Bitboard, empty: Bitboard, dir: Direction) -> Bitboard {
    let mut flood: Bitboard = 0;
    while sliders > 0 {
        flood |= sliders;
        sliders = sliders.shift(dir) & empty;
    }
    flood
}

#[cfg(test)]
mod tests {
    use littlewing::common::*;
    use super::BitboardExt;
    use super::dumb7fill;

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
