use littlewing::common::*;
use littlewing::bitboard::BitwiseOperations;

const KNIGHT_SQBB: [Bitboard, ..64] = [
    0x0000000000020400,
    0x0000000000050800,
    0x00000000000A1100,
    0x0000000000142200,
    0x0000000000284400,
    0x0000000000508800,
    0x0000000000A01000,
    0x0000000000402000,
    0x0000000002040004,
    0x0000000005080008,
    0x000000000A110011,
    0x0000000014220022,
    0x0000000028440044,
    0x0000000050880088,
    0x00000000A0100010,
    0x0000000040200020,
    0x0000000204000402,
    0x0000000508000805,
    0x0000000A1100110A,
    0x0000001422002214,
    0x0000002844004428,
    0x0000005088008850,
    0x000000A0100010A0,
    0x0000004020002040,
    0x0000020400040200,
    0x0000050800080500,
    0x00000A1100110A00,
    0x0000142200221400,
    0x0000284400442800,
    0x0000508800885000,
    0x0000A0100010A000,
    0x0000402000204000,
    0x0002040004020000,
    0x0005080008050000,
    0x000A1100110A0000,
    0x0014220022140000,
    0x0028440044280000,
    0x0050880088500000,
    0x00A0100010A00000,
    0x0040200020400000,
    0x0204000402000000,
    0x0508000805000000,
    0x0A1100110A000000,
    0x1422002214000000,
    0x2844004428000000,
    0x5088008850000000,
    0xA0100010A0000000,
    0x4020002040000000,
    0x0400040200000000,
    0x0800080500000000,
    0x1100110A00000000,
    0x2200221400000000,
    0x4400442800000000,
    0x8800885000000000,
    0x100010A000000000,
    0x2000204000000000,
    0x0004020000000000,
    0x0008050000000000,
    0x00110A0000000000,
    0x0022140000000000,
    0x0044280000000000,
    0x0088500000000000,
    0x0010A00000000000,
    0x0020400000000000
];

pub struct Init;

impl Init {
    pub fn knight_sqbb() {
        println!("const KNIGHT_SQBB: [Bitboard, ..64] = [");
        let ydirs = [UP, DOWN];
        let xdirs = [LEFT, RIGHT];

        let mut knight_sqbb: [Bitboard, ..64] = [0, ..64];
        for sq in range(0, 64) {
            // Use 0x88 board representation to do off the board tests
            let sq88 = sq + (sq & !7);
            for i in range(0, 2) {
                for j in range(0, 2) {
                    // A 0x88 board contains 128 squares, that is two boards
                    // of 64 squares side by side. So UP and DOWN values must
                    // be doubled.
                    if (sq88 + 4 * ydirs[i] + xdirs[j]) & 0x88 == 0 {
                        knight_sqbb[sq].set(sq + 2 * ydirs[i] + xdirs[j]);
                    }
                    if (sq88 + 2 * ydirs[i] + 2 * xdirs[j]) & 0x88 == 0 {
                        knight_sqbb[sq].set(sq + ydirs[i] + 2 * xdirs[j]);
                    }
                }
            }
            println!("    0x{:016X},", knight_sqbb[sq]);
        }
        println!("];");
    }
}

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
    fn add_moves_from(&mut self, mut targets: Bitboard, from: Direction, mt: MoveType);
    fn add_pawns_moves(&mut self, bitboards: &[Bitboard], side: Color);
    fn add_knights_moves(&mut self, bitboards: &[Bitboard], side: Color);
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
    fn add_moves_from(&mut self, mut targets: Bitboard, from: Direction, mt: MoveType) {
        while targets != 0 {
            let to = targets.ffs();
            let m = Move::new(from, to, mt);
            self.push(m);
            targets.reset(to);
        }
    }
    fn add_pawns_moves(&mut self, bitboards: &[Bitboard], side: Color) {
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

    fn add_knights_moves(&mut self, bitboards: &[Bitboard], side: Color) {
        let mut knights = bitboards[side | KNIGHT];

        while knights > 0 {
            let from = knights.ffs();
            let targets = KNIGHT_SQBB[from] & !bitboards[side];
            self.add_moves_from(targets, from, QUIET_MOVE);
            let targets = KNIGHT_SQBB[from] & bitboards[side ^ 1];
            self.add_moves_from(targets, from, CAPTURE);
            knights.reset(from);
        }
    }
}
