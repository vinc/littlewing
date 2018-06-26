use common::*;
use square::*;
use bitboard::{Bitboard, BitboardExt};

// Flood fill algorithm
#[allow(dead_code)]
pub fn dumb7fill(mut fill: Bitboard, empty: Bitboard, dir: Shift) -> Bitboard {
    let mut flood: Bitboard = 0;

    while fill > 0 {
        flood |= fill;
        fill = fill.shift(dir) & empty;
    }

    flood
}

#[allow(dead_code)]
pub fn bishop_attacks(from: Square, occupied: Bitboard) -> Bitboard {
    let fill = 1 << from;
    let mut targets = 0;

    let occluded = dumb7fill(fill, !occupied & 0x7F7F7F7F7F7F7F7F, UP + LEFT);
    targets |= 0x7F7F7F7F7F7F7F7F & occluded.shift(UP + LEFT);
    let occluded = dumb7fill(fill, !occupied & 0x7F7F7F7F7F7F7F7F, DOWN + LEFT);
    targets |= 0x7F7F7F7F7F7F7F7F & occluded.shift(DOWN + LEFT);
    let occluded = dumb7fill(fill, !occupied & 0xFEFEFEFEFEFEFEFE, DOWN + RIGHT);
    targets |= 0xFEFEFEFEFEFEFEFE & occluded.shift(DOWN + RIGHT);
    let occluded = dumb7fill(fill, !occupied & 0xFEFEFEFEFEFEFEFE, UP + RIGHT);
    targets |= 0xFEFEFEFEFEFEFEFE & occluded.shift(UP + RIGHT);

    targets
}

#[allow(dead_code)]
pub fn rook_attacks(from: Square, occupied: Bitboard) -> Bitboard {
    let fill = 1 << from;
    let mut targets = 0;

    let occluded = dumb7fill(fill, !occupied & 0xFFFFFFFFFFFFFFFF, UP);
    targets |= 0xFFFFFFFFFFFFFFFF & occluded.shift(UP);
    let occluded = dumb7fill(fill, !occupied & 0xFFFFFFFFFFFFFFFF, DOWN);
    targets |= 0xFFFFFFFFFFFFFFFF & occluded.shift(DOWN);
    let occluded = dumb7fill(fill, !occupied & 0x7F7F7F7F7F7F7F7F, LEFT);
    targets |= 0x7F7F7F7F7F7F7F7F & occluded.shift(LEFT);
    let occluded = dumb7fill(fill, !occupied & 0xFEFEFEFEFEFEFEFE, RIGHT);
    targets |= 0xFEFEFEFEFEFEFEFE & occluded.shift(RIGHT);

    targets
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
}
