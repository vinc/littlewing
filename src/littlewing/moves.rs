use littlewing::common::*;
use littlewing::attack::{bishop_attacks, rook_attacks};
use littlewing::piece::PieceChar;
use littlewing::square::SquareString;
use littlewing::bitboard::BitwiseOperations;

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
    lists: Vec<Vec<Move>>,
    pub ply: uint
}

impl Moves {
    pub fn new() -> Moves {
        let mut moves = Moves {
            lists: Vec::with_capacity(MAX_PLY),
            ply: 0
        };
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
    pub fn add_moves(&mut self, mut targets: Bitboard, dir: Direction, mt: MoveType) {
        while targets != 0 {
            let to = targets.ffs();
            let from = to - dir;
            let m = Move::new(from, to, mt);

            self.lists[self.ply].push(m);
            targets.reset(to);
        }
    }
    pub fn add_moves_from(&mut self, mut targets: Bitboard, from: Square, mt: MoveType) {
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
            let targets = PIECE_MASKS[KNIGHT][from] & !bitboards[side];
            self.add_moves_from(targets, from, QUIET_MOVE);
            let targets = PIECE_MASKS[KNIGHT][from] & bitboards[side ^ 1];
            self.add_moves_from(targets, from, CAPTURE);
            knights.reset(from);
        }
    }
    pub fn add_king_moves(&mut self, bitboards: &[Bitboard], side: Color) {
        let mut kings = bitboards[side | KING];

        while kings > 0 {
            let from = kings.ffs();
            let targets = PIECE_MASKS[KING][from] & !bitboards[side];
            self.add_moves_from(targets, from, QUIET_MOVE);
            let targets = PIECE_MASKS[KING][from] & bitboards[side ^ 1];
            self.add_moves_from(targets, from, CAPTURE);
            kings.reset(from);
        }
    }
    pub fn add_bishops_moves(&mut self, bitboards: &[Bitboard], side: Color) {
        let occupied = bitboards[WHITE] | bitboards[BLACK];
        let mut bishops = bitboards[side | BISHOP];
        while bishops > 0 {
            let from = bishops.ffs();
            let targets = bishop_attacks(from, occupied);
            self.add_moves_from(targets & !occupied, from, QUIET_MOVE);
            self.add_moves_from(targets & bitboards[side ^ 1], from, CAPTURE);
            bishops.reset(from);
        }
    }
    pub fn add_rooks_moves(&mut self, bitboards: &[Bitboard], side: Color) {
        let occupied = bitboards[WHITE] | bitboards[BLACK];
        let mut rooks = bitboards[side | ROOK];
        while rooks > 0 {
            let from = rooks.ffs();
            let targets = rook_attacks(from, occupied);
            self.add_moves_from(targets & !occupied, from, QUIET_MOVE);
            self.add_moves_from(targets & bitboards[side ^ 1], from, CAPTURE);
            rooks.reset(from);
        }
    }
    pub fn add_queens_moves(&mut self, bitboards: &[Bitboard], side: Color) {
        let occupied = bitboards[WHITE] | bitboards[BLACK];
        let mut queens = bitboards[side | QUEEN];
        while queens > 0 {
            let from = queens.ffs();
            let targets = bishop_attacks(from, occupied) | rook_attacks(from, occupied);
            self.add_moves_from(targets & !occupied, from, QUIET_MOVE);
            self.add_moves_from(targets & bitboards[side ^ 1], from, CAPTURE);

            queens.reset(from);
        }
    }
}
