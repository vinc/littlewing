use littlewing::common::*;
use littlewing::piece::PieceChar;
use littlewing::square::SquareString;
use littlewing::bitboard::BitwiseOperations;

const MAX_PLY: uint = 256;
const MAX_MOVES: uint = 256;

#[deriving(Copy)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub kind: MoveType // FIXME
}

impl Move {
    pub fn new(f: Square, t: Square, mt: MoveType) -> Move {
        Move {
            from: f,
            to: t,
            kind: mt
        }
    }
    pub fn to_string(&self, board: &[Piece]) -> String {
        let mut out = String::new();
        let piece = board[self.from] & !BLACK;
        let capture = board[self.to];
        if piece != PAWN {
            out.push(piece.to_char());
        }
        out.push_str(self.from.to_square_string().as_slice());
        if capture == EMPTY {
            out.push('-');
        } else {
            out.push('x');
        }
        out.push_str(self.to.to_square_string().as_slice());
        out
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
        let mut moves = Moves {
            knight_mask: [0, ..64],
            bishop_mask: [0, ..64],
            rook_mask:   [0, ..64],
            queen_mask:  [0, ..64],
            king_mask:   [0, ..64],
            lists: Vec::with_capacity(MAX_PLY),
            ply: 0
        };

        moves.init_masks();
        for _ in range(0u, MAX_PLY) {
            moves.lists.push(Vec::with_capacity(MAX_MOVES));
        }

        moves
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
    pub fn add_bishops_moves(&mut self, bitboards: &[Bitboard], side: Color) {
        let mut bishops = bitboards[side | BISHOP];
        while bishops > 0 {
            let from = bishops.ffs();
            self.add_bishop_moves(from, bitboards, side);
            bishops.reset(from);
        }
    }
    fn add_bishop_moves(&mut self, from: Square, bitboards: &[Bitboard], side: Color) {
        let occupied = bitboards[WHITE] | bitboards[BLACK];

        const DIRS: [Square, ..4 ] = [
            UP + LEFT,
            DOWN + LEFT,
            DOWN + RIGHT,
            UP + RIGHT
        ];
        const WRAPS: [Bitboard, ..4] = [
            0xFEFEFEFEFEFEFEFE,
            0xFEFEFEFEFEFEFEFE,
            0x7F7F7F7F7F7F7F7F,
            0x7F7F7F7F7F7F7F7F
        ];
        for i in range(0u, 4) {
            let targets = Moves::dumb7fill(1 << from, !occupied & WRAPS[i], DIRS[i]).shift(DIRS[i]);
            self.add_moves_from(targets & !occupied, from, QUIET_MOVE);
            self.add_moves_from(targets & bitboards[side ^ 1], from, CAPTURE);
        }
    }
    pub fn add_rooks_moves(&mut self, bitboards: &[Bitboard], side: Color) {
        let mut rooks = bitboards[side | ROOK];
        while rooks > 0 {
            let from = rooks.ffs();
            self.add_rook_moves(from, bitboards, side);
            rooks.reset(from);
        }
    }
    fn add_rook_moves(&mut self, from: Square, bitboards: &[Bitboard], side: Color) {
        let occupied = bitboards[WHITE] | bitboards[BLACK];

        const DIRS: [Square, ..4 ] = [
            UP,
            DOWN,
            LEFT,
            RIGHT
        ];
        const WRAPS: [Bitboard, ..4] = [
            0xFFFFFFFFFFFFFFFF,
            0xFFFFFFFFFFFFFFFF,
            0xFEFEFEFEFEFEFEFE,
            0x7F7F7F7F7F7F7F7F
        ];
        for i in range(0u, 4) {
            let targets = Moves::dumb7fill(1 << from, !occupied & WRAPS[i], DIRS[i]).shift(DIRS[i]);
            self.add_moves_from(targets & !occupied, from, QUIET_MOVE);
            self.add_moves_from(targets & bitboards[side ^ 1], from, CAPTURE);
        }
    }
    pub fn add_queens_moves(&mut self, bitboards: &[Bitboard], side: Color) {
        let mut queens = bitboards[side | QUEEN];
        while queens > 0 {
            let from = queens.ffs();
            self.add_bishop_moves(from, bitboards, side);
            self.add_rook_moves(from, bitboards, side);
            queens.reset(from);
        }
    }

    fn dumb7fill(mut sliders: Bitboard, empty: Bitboard, dir: uint) -> Bitboard {
        let mut flood: Bitboard = 0;
        while sliders > 0 {
            flood |= sliders;
            sliders = sliders.shift(dir) & empty;
        }
        flood
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
                            if dx >= 8 || dy >= 8 {
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
                            if dx + i >= 8 || dy + j >= 8 {
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
    fn test_dumb7fill() {
        let rooks: Bitboard = 0x0000000000100000;

        let empty: Bitboard = !rooks;
        let targets = Moves::dumb7fill(rooks, empty, UP);
        targets.debug();
        let attacks = targets.shift(UP);
        attacks.debug();
        assert_eq!(targets, 0x1010101010100000);

        let empty: Bitboard = !rooks;
        let targets = Moves::dumb7fill(rooks, empty, DOWN);
        targets.debug();
        let attacks = targets.shift(DOWN);
        attacks.debug();
        assert_eq!(targets, 0x0000000000101010);

        let empty: Bitboard = !rooks & 0x7F7F7F7F7F7F7F7F;
        let targets = Moves::dumb7fill(rooks, empty, RIGHT);
        targets.debug();
        let attacks = targets.shift(RIGHT);
        attacks.debug();
        assert_eq!(targets, 0x0000000000700000);

        let empty: Bitboard = !(rooks | rooks << 16); // With blocker
        let targets = Moves::dumb7fill(rooks, empty, UP);
        targets.debug();
        let attacks = targets.shift(UP);
        attacks.debug();
        assert_eq!(targets, 0x0000000010100000);

        let bishop: Bitboard = 0x0000000000100000;
        let empty: Bitboard = !bishop & 0x7F7F7F7F7F7F7F7F;
        let targets = Moves::dumb7fill(bishop, empty, UP + RIGHT);
        targets.debug();
        let attacks = targets.shift(UP + RIGHT);
        attacks.debug();
        assert_eq!(targets, 0x0000004020100000);
    }

    #[test]
    fn test_init_masks() {
        let mut moves = Moves::new();
        moves.init_masks();
        assert_eq!(moves.king_mask[A1],   0x0000000000000302);
        assert_eq!(moves.king_mask[E3],   0x0000000038283800);
        assert_eq!(moves.knight_mask[B1], 0x0000000000050800);
        assert_eq!(moves.bishop_mask[A1], 0x0040201008040200);
        assert_eq!(moves.bishop_mask[E3], 0x0000024428002800);
        assert_eq!(moves.rook_mask[E3],   0x00101010106E1000);
        assert_eq!(moves.rook_mask[A1],   0x000101010101017E);
    }
}
