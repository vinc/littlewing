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
    bishop_mask: [Bitboard, ..64],
    rook_mask:   [Bitboard, ..64],
    queen_mask:  [Bitboard, ..64],
    king_mask:   [Bitboard, ..64],
    lists: Vec<Vec<Move>>,
    pub ply: uint
}

impl Moves {
    pub fn new() -> Moves {
        Moves {
            knight_mask: [0, ..64],
            bishop_mask: [0, ..64],
            rook_mask:   [0, ..64],
            queen_mask:  [0, ..64],
            king_mask:   [0, ..64],
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
        let deltas = [-2u, -1u, 0u, 1u, 2u];

        for x in range(0u, 8) {
            for y in range(0u, 8) {
                let from = 8 * x + y;
                for &i in deltas.iter() {
                    for &j in deltas.iter() {
                        for k in range(1u, 7) {
                            let dx = x + i * k;
                            let dy = y + j * k;
                            let to = 8 * dx + dy;
                            if to == from {
                                break;
                            }
                            if dx < 0 || dx >= 8 || dy < 0 || dy >= 8 {
                                break; // Out of board
                            }
                            if i == -2u || j == -2u || i == 2u || j == 2u {
                                if i == -1u || j == -1u || i == 1u || j == 1u {
                                    self.knight_mask[from].set(to);
                                }
                                break;
                            }
                            if k == 1 {
                                self.king_mask[from].set(to);
                            }
                            if dx + i < 0 || dx + i >= 8 || dy + j < 0 || dy + j >= 8 {
                                break; // Edge of the board
                            }
                            if i == 0 || j == 0 {
                                self.rook_mask[from].set(to);
                            } else {
                                self.bishop_mask[from].set(to);
                            }
                            self.queen_mask[from].set(to);
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use littlewing::common::*;
    use littlewing::bitboard::BitwiseOperations;
    use super::Moves;

    #[test]
    fn test_init_masks() {
        let mut moves = Moves::new();
        moves.init_masks();

        moves.king_mask[A1].debug();
        assert_eq!(moves.king_mask[A1], 0x0000000000000302);

        moves.king_mask[E3].debug();
        assert_eq!(moves.king_mask[E3], 0x0000000038283800);

        moves.knight_mask[B1].debug();
        assert_eq!(moves.knight_mask[B1], 0x0000000000050800);

        moves.bishop_mask[A1].debug();
        assert_eq!(moves.bishop_mask[A1], 0x0040201008040200);

        moves.bishop_mask[E3].debug();
        assert_eq!(moves.bishop_mask[E3], 0x0000024428002800);

        moves.rook_mask[E3].debug();
        assert_eq!(moves.rook_mask[E3], 0x00101010106E1000);

        moves.rook_mask[A1].debug();
        assert_eq!(moves.rook_mask[A1], 0x000101010101017E);
    }
}
