use std::ops::Index;

use littlewing::common::*;
use littlewing::attack::{bishop_attacks, rook_attacks};
use littlewing::piece::PieceChar;
use littlewing::square::SquareString;
use littlewing::bitboard::BitwiseOperations;

#[derive(Copy)]
pub struct Move(u16);

impl Move {
    pub fn new(f: Square, t: Square, mt: MoveType) -> Move {
        Move(((f << 10) | (t << 4) | mt) as u16)
    }
    pub fn from(&self) -> Square {
        let Move(v) = *self;
        (v >> 10) as Square
    }
    pub fn to(&self) -> Square {
        let Move(v) = *self;
        ((v >> 4) & 0b111111) as Square
    }
    pub fn kind(&self) -> MoveType {
        let Move(v) = *self;
        (v & 0b1111) as MoveType
    }
    pub fn is_castle(&self) -> bool {
        self.kind() == KING_CASTLE || self.kind() == QUEEN_CASTLE
    }
    pub fn castle_kind(&self) -> Piece {
        QUEEN_CASTLE << self.kind() - 1
    }
    pub fn is_promotion(&self) -> bool {
        self.kind() & PROMOTION_MASK > 0
    }
    pub fn promotion_kind(&self) -> Piece {
        PROMOTION_KINDS[self.kind() & PROMOTION_KIND_MASK >> 2]
    }
    pub fn to_can(&self) -> String {
        let mut out = String::new();
        out.push_str(self.from().to_square_string().as_slice());
        out.push_str(self.to().to_square_string().as_slice());
        if self.is_promotion() {
            out.push((BLACK | self.promotion_kind()).to_char());        
        }
        out
    }
    pub fn to_san(&self, board: &[Piece]) -> String {
        let mut out = String::new();
        let piece = board[self.from()] & !BLACK;
        let capture = board[self.to()];
        if piece != PAWN {
            out.push(piece.to_char());
        }
        out.push_str(self.from().to_square_string().as_slice());
        if capture == EMPTY {
            out.push('-');
        } else {
            out.push('x');
        }
        out.push_str(self.to().to_square_string().as_slice());
        out
    }
}

pub struct Moves {
    lists: [[Move; MAX_MOVES]; MAX_PLY],
    lens: [usize; MAX_PLY],
    pub ply: usize
}

impl Moves {
    pub fn new() -> Moves {
        Moves {
            lists: [[Move::new(A1, A1, QUIET_MOVE); MAX_MOVES]; MAX_PLY],
            lens: [0; MAX_PLY],
            ply: 0
        }
    }
    pub fn inc(&mut self) {
        self.ply += 1;
    }
    pub fn dec(&mut self) {
        self.ply -= 1;
    }
    pub fn clear(&mut self) {
        self.lens[self.ply] = 0;
    }
    pub fn len(&self) -> usize {
        self.lens[self.ply]
    }
    pub fn add_move(&mut self, from: Square, to: Square, mt: MoveType) {
        self.lists[self.ply][self.lens[self.ply]] = Move::new(from, to, mt);
        self.lens[self.ply] += 1;
    }
    pub fn add_moves(&mut self, mut targets: Bitboard, dir: Direction, mt: MoveType) {
        while targets != 0 {
            let to = targets.ffs();
            let from = to - dir;
            self.add_move(from, to, mt);
            targets.reset(to);
        }
    }
    pub fn add_moves_from(&mut self, mut targets: Bitboard, from: Square, mt: MoveType) {
        while targets != 0 {
            let to = targets.ffs();
            self.add_move(from, to, mt);
            targets.reset(to);
        }
    }
    pub fn add_pawns_moves(&mut self, bitboards: &[Bitboard], side: Color, ep: Square) {
        const XDIRS: [Direction; 2] = [LEFT, RIGHT];
        const YDIRS: [Direction; 2] = [UP, DOWN];
        const FILES: [Bitboard; 2] = [FILE_A, FILE_H];
        const SEC_RANKS: [Bitboard; 2] = [RANK_3, RANK_6];
        const END_RANKS: [Bitboard; 2] = [RANK_8, RANK_1];

        let ydir = YDIRS[side];

        let occupied = bitboards[WHITE] | bitboards[BLACK];

        let pushes = bitboards[side | PAWN].shift(ydir) & !occupied;
        self.add_moves(pushes & !END_RANKS[side], ydir, QUIET_MOVE);
        self.add_moves(pushes & END_RANKS[side], ydir, KNIGHT_PROMOTION);
        self.add_moves(pushes & END_RANKS[side], ydir, BISHOP_PROMOTION);
        self.add_moves(pushes & END_RANKS[side], ydir, ROOK_PROMOTION);
        self.add_moves(pushes & END_RANKS[side], ydir, QUEEN_PROMOTION);

        let double_pushes = (pushes & SEC_RANKS[side]).shift(ydir) & !occupied;
        self.add_moves(double_pushes, 2 * ydir, DOUBLE_PAWN_PUSH);

        for i in range(0, 2) { // LEFT and RIGHT attacks
            let dir = ydir + XDIRS[i];
            let attackers = bitboards[side | PAWN] & !FILES[i];

            let targets = attackers.shift(dir);
            //let epb = 1 << ep; // FIXME: 1 << 64 == 0
            let epb = ((ep as u64 >> 6) ^ 1) << (ep % 64);
            self.add_moves(targets & epb, dir, EN_PASSANT);

            let attacks = targets & bitboards[side ^ 1];
            self.add_moves(attacks & !END_RANKS[side], dir, CAPTURE);
            self.add_moves(attacks & END_RANKS[side], dir, KNIGHT_PROMOTION_CAPTURE);
            self.add_moves(attacks & END_RANKS[side], dir, BISHOP_PROMOTION_CAPTURE);
            self.add_moves(attacks & END_RANKS[side], dir, ROOK_PROMOTION_CAPTURE);
            self.add_moves(attacks & END_RANKS[side], dir, QUEEN_PROMOTION_CAPTURE);
        }
    }
    pub fn add_knights_moves(&mut self, bitboards: &[Bitboard], side: Color) {
        let occupied = bitboards[WHITE] | bitboards[BLACK];
        let mut knights = bitboards[side | KNIGHT];

        while knights > 0 {
            let from = knights.ffs();
            let targets = PIECE_MASKS[KNIGHT][from] & !occupied;
            self.add_moves_from(targets, from, QUIET_MOVE);
            let targets = PIECE_MASKS[KNIGHT][from] & bitboards[side ^ 1];
            self.add_moves_from(targets, from, CAPTURE);
            knights.reset(from);
        }
    }
    pub fn add_king_moves(&mut self, bitboards: &[Bitboard], side: Color) {
        let occupied = bitboards[WHITE] | bitboards[BLACK];
        let mut kings = bitboards[side | KING];

        while kings > 0 {
            let from = kings.ffs();
            let targets = PIECE_MASKS[KING][from] & !occupied;
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
    pub fn add_king_castle(&mut self, side: Color) {
        self.add_move(E1 ^ 56 * side, G1 ^ 56 * side, KING_CASTLE);
    }
    pub fn add_queen_castle(&mut self, side: Color) {
        self.add_move(E1 ^ 56 * side, C1 ^ 56 * side, QUEEN_CASTLE);
    }
}

impl Index<usize> for Moves {
    type Output = Move;
    fn index(&self, _index: &usize) -> &Move {
        &self.lists[self.ply][*_index]
    }
}

#[cfg(test)]
mod tests {
    use littlewing::common::*;
    use super::Move;

    #[test]
    fn test_move_is_castle() {
        assert_eq!(Move::new(E2, E3, QUIET_MOVE).is_castle(), false);
        assert_eq!(Move::new(E1, G1, KING_CASTLE).is_castle(), true);
        assert_eq!(Move::new(E1, C1, QUEEN_CASTLE).is_castle(), true);
    }
    fn test_move_castle_kind() {
        assert_eq!(Move::new(E1, G1, KING_CASTLE).castle_kind(), KING);
        assert_eq!(Move::new(E1, C1, QUEEN_CASTLE).castle_kind(), QUEEN);
    }
    fn test_move_is_promotion() {
        assert_eq!(Move::new(E2, E3, QUIET_MOVE).is_promotion(), false);
        assert_eq!(Move::new(E7, E8, QUEEN_PROMOTION).is_promotion(), true);
        assert_eq!(Move::new(E7, D8, QUEEN_PROMOTION_CAPTURE).is_promotion(), true);
    }
    fn test_move_promotion_kind() {
        assert_eq!(Move::new(E7, E8, QUEEN_PROMOTION).promotion_kind(), QUEEN);
        assert_eq!(Move::new(E7, D8, ROOK_PROMOTION_CAPTURE).promotion_kind(), ROOK);
    }
}
