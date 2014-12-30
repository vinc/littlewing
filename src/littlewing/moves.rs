use littlewing::common::*;
use littlewing::bitboard::BitwiseOperations;

const MAX_PLY: uint = 256;
const MAX_MOVES: uint = 256;

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

pub struct Moves {
    knight_mask: [Bitboard, ..64],
    king_mask: [Bitboard, ..64],
    lists: Vec<Vec<Move>>,
    pub ply: uint
}

impl Moves {
    pub fn new() -> Moves {
        Moves {
            knight_mask: [0, ..64],
            king_mask: [0, ..64],
            lists: Vec::with_capacity(MAX_PLY),
            ply: 0
        }
    }
    pub fn init(&mut self) {
        self.init_masks();
        for _ in range(0u, MAX_PLY) {
            self.lists.push(Vec::with_capacity(MAX_MOVES));
        }
    }
    pub fn inc(&mut self) {
        self.ply += 1;
    }
    pub fn dec(&mut self) {
        self.ply -= 1;
    }
    pub fn clear(&mut self) {
        self.lists[self.ply].clear()
    }
    pub fn len(&self) -> uint {
        self.lists[self.ply].len()
    }
    pub fn get(&self, i: uint) -> Move {
        self.lists[self.ply][i]
    }
    pub fn add_moves(&mut self, mut targets: Bitboard, dir: uint, mt: MoveType) {
        while targets != 0 {
            let to = targets.ffs();
            let from = to - dir;
            let m = Move::new(from, to, mt);

            self.lists[self.ply].push(m);
            targets.reset(to);
        }
    }
    pub fn add_moves_from(&mut self, mut targets: Bitboard, from: Direction, mt: MoveType) {
        while targets != 0 {
            let to = targets.ffs();
            let m = Move::new(from, to, mt);

            self.lists[self.ply].push(m);
            targets.reset(to);
        }
    }
    pub fn add_pawns_moves(&mut self, bitboards: &[Bitboard], side: Color) {
        const XDIRS: [[Direction, ..2], ..2] = [[LEFT, RIGHT], [RIGHT, LEFT]];
        const YDIRS: [Direction, ..2] = [UP, DOWN];
        const FILES: [Bitboard, ..2] = [FILE_A, FILE_H];
        const RANKS: [Bitboard, ..2] = [RANK_3, RANK_6];

        let ydir = YDIRS[side];

        let occupied = bitboards[WHITE] | bitboards[BLACK];

        let pushes = bitboards[side | PAWN].shift(ydir) & !occupied;
        self.add_moves(pushes, ydir, QUIET_MOVE);

        let double_pushes = (pushes & RANKS[side]).shift(ydir) & !occupied;
        self.add_moves(double_pushes, 2 * ydir, DOUBLE_PAWN_PUSH);

        for i in range(0, 2) { // LEFT and RIGHT attacks
            let dir = ydir + XDIRS[side][i];
            let attacks =
                (bitboards[side | PAWN] & !FILES[i]).shift(dir)
                & bitboards[side ^ 1] & !bitboards[side];
            self.add_moves(attacks, dir, CAPTURE);
        }
    }
    pub fn add_knights_moves(&mut self, bitboards: &[Bitboard], side: Color) {
        let mut knights = bitboards[side | KNIGHT];

        while knights > 0 {
            let from = knights.ffs();
            let targets = self.knight_mask[from] & !bitboards[side];
            self.add_moves_from(targets, from, QUIET_MOVE);
            let targets = self.knight_mask[from] & bitboards[side ^ 1];
            self.add_moves_from(targets, from, CAPTURE);
            knights.reset(from);
        }
    }
    pub fn add_king_moves(&mut self, bitboards: &[Bitboard], side: Color) {
        let mut kings = bitboards[side | KING];

        while kings > 0 {
            let from = kings.ffs();
            let targets = self.king_mask[from] & !bitboards[side];
            self.add_moves_from(targets, from, QUIET_MOVE);
            let targets = self.king_mask[from] & bitboards[side ^ 1];
            self.add_moves_from(targets, from, CAPTURE);
            kings.reset(from);
        }
    }
    fn init_masks(&mut self) {
        let ydirs = [UP, DOWN];
        let xdirs = [LEFT, RIGHT];

        for sq in range(0, 64) {
            // Use 0x88 board representation to do off the board tests
            let sq88 = sq + (sq & !7);
            for i in range(0, 2) {
                // A 0x88 board contains 128 squares, that is two boards
                // of 64 squares side by side. So UP and DOWN values must
                // be doubled.

                // Example: UP
                if (sq88 + 2 * ydirs[i]) & 0x88 == 0 {
                    self.king_mask[sq].set(sq + ydirs[i]);
                }
                // Example: LEFT
                if (sq88 + xdirs[i]) & 0x88 == 0 {
                    self.king_mask[sq].set(sq + xdirs[i]);
                }
                for j in range(0, 2) {
                    // Example: UP + LEFT
                    if (sq88 + 2 * ydirs[i] + xdirs[j]) & 0x88 == 0 {
                        self.king_mask[sq].set(sq + ydirs[i] + xdirs[j]);
                    }
                    // Example: UP + UP + LEFT
                    if (sq88 + 4 * ydirs[i] + xdirs[j]) & 0x88 == 0 {
                        self.knight_mask[sq].set(sq + 2 * ydirs[i] + xdirs[j]);
                    }
                    // Example: UP + LEFT + LEFT
                    if (sq88 + 2 * ydirs[i] + 2 * xdirs[j]) & 0x88 == 0 {
                        self.knight_mask[sq].set(sq + ydirs[i] + 2 * xdirs[j]);
                    }
                }
            }
        }
    }
}
