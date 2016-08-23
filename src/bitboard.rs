use common::*;

pub trait BitboardExt {
    fn count(&self) -> u32;
    fn shift(&self, x: Direction) -> Bitboard;
    fn toggle(&mut self, i: Square); // FIXME: Return instead of update?
    fn set(&mut self, i: Square);
    fn reset(&mut self, i: Square);
    fn get(&self, i: Square) -> bool;
    fn debug(&self);
    fn to_debug_string(&self) -> String;
}

impl BitboardExt for Bitboard {
    fn count(&self) -> u32 {
        self.count_ones()
    }

    fn shift(&self, x: Direction) -> Bitboard {
        if x > 0 {
            *self << x
        } else {
            *self >> -x
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

    //FIXME: remove this method
    //#[deprecated(since="0.2.0", note="please use `to_debug_string` instead")]
    fn debug(&self) {
        println!("{}", self.to_debug_string());
    }

    fn to_debug_string(&self) -> String {
        let mut out = String::new();

        out.push_str(&format!("DEBUG(bitboard): 0x{:016X}", *self));

        for i in 0..8 {
            for j in 0..8 {
                out.push_str(&format!("{:b}", self.get(8 * i + j) as usize));
            }
            out.push('\n');
        }
        out.push('\n');

        out
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


pub trait BitboardIterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
}

impl BitboardIterator for Bitboard {
    type Item = Square;

    fn next(&mut self) -> Option<Square> {
        if *self > 0 {
            let sq = self.trailing_zeros() as Square;

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

    /*
    #[bench]
    fn bench_iterator(b: &mut Bencher) {
        let mut bb: Bitboard = 0; // TODO
        b.iter(|| {
            // Old way
            while bb > 0 {
                let sq = bb.trailing_zeros() as Square;
                bb.reset(sq);
                // TODO
            }
        })

        let mut bb: Bitboard = 0; // TODO
        b.iter(|| {
            // New way
            while let Some(sq) = bb.next() {
                // TODO
            }
        })
    }
    */
}
