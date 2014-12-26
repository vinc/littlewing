use littlewing::common::*;
use littlewing::bitboard::BitwiseOperations;

#[deriving(Copy)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    _type: MoveType // FIXME
}

impl Move {
    pub fn new(f: Square, t: Square, mt: MoveType) -> Move {
        Move {
            from: f,
            to: t,
            _type: mt
        }
    }
}

pub type Moves = Vec<Move>;

pub trait MovesOperations {
    fn add_moves(&mut self, mut targets: Bitboard, dir: uint, mt: MoveType);
    fn add_pawn_moves(&mut self, bitboards: &[Bitboard], side: Color);
}

impl MovesOperations for Moves {
    fn add_moves(&mut self, mut targets: Bitboard, dir: uint, mt: MoveType) {
        while targets != 0 {
            let to = targets.ffs();
            let from = to - dir;
            let m = Move::new(from, to, mt);
            self.push(m);
            targets.reset(to);
        }
    }
    fn add_pawn_moves(&mut self, bitboards: &[Bitboard], side: Color) {
        const YDIRS: [Direction, ..2] = [UP, DOWN];
        const XDIRS: [[Direction, ..2], ..2] = [[LEFT, RIGHT], [RIGHT, LEFT]];
        const FILES: [Bitboard, ..2] = [FILE_A, FILE_H];
        const RANKS: [Bitboard, ..2] = [RANK_3, RANK_6];

        let ydir = YDIRS[side];

        let occupied = bitboards[WHITE] | bitboards[BLACK];

        let pushes = bitboards[side | PAWN].shift(ydir) & !occupied;
        self.add_moves(pushes, ydir, QUIET_MOVE);

        let double_pushes = (pushes & RANKS[side]).shift(ydir) & !occupied;
        self.add_moves(double_pushes, 2 * ydir, DOUBLE_PAWN_PUSH);

        for i in range(0, 2) {
            let dir = ydir + XDIRS[side][i];
            let attacks =
                (bitboards[side | PAWN] & !FILES[i]).shift(dir)
                & bitboards[side ^ 1] & !bitboards[side];
            self.add_moves(attacks, dir, CAPTURE);
        }
    }
}
