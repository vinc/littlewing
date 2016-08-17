use std::fmt;
use std::ops::{Index, IndexMut};

use common::*;
use attack::{bishop_attacks, rook_attacks};
use piece::PieceChar;
use square::SquareString;
use bitboard::BitboardExt;

const BEST_MOVE_SCORE:  u8 = 255;
const QUIET_MOVE_SCORE: u8 = 0;

#[derive(Copy, Clone, PartialEq)]
pub struct Move(u16);

impl Move {
    pub fn new(from: Square, to: Square, mt: MoveType) -> Move {
        Move(((from as u16) << 10) | ((to as u16) << 4) | mt as u16)
    }

    pub fn new_null() -> Move {
        Move(0)
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

    pub fn is_null(&self) -> bool {
        let Move(v) = *self;
        v == 0
    }

    pub fn is_capture(&self) -> bool {
        self.kind() == CAPTURE || self.kind() & 0b1100 == 0b1100
    }

    pub fn is_en_passant(&self) -> bool {
        self.kind() == EN_PASSANT
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
        PROMOTION_KINDS[(self.kind() & PROMOTION_KIND_MASK >> 2) as usize]
    }

    pub fn to_can(&self) -> String {
        self.to_string()
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut out = String::new();
        out.push_str(self.from().to_coord().as_str());
        out.push_str(self.to().to_coord().as_str());
        if self.is_promotion() {
            out.push((BLACK | self.promotion_kind()).to_char());        
        }
        write!(f, "{}", out)
    }
}

impl fmt::Debug for Move {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

#[derive(Copy, Clone, PartialEq)]
pub struct MoveExt {
    pub m: Move,
    pub s: u8
}

impl MoveExt {
    pub fn new(m: Move, s: u8) -> MoveExt {
        MoveExt { m: m, s: s }
    }
}

#[repr(usize)]
#[derive(Clone, Copy, PartialEq)]
pub enum MovesState {
    BestMove,
    //GoodCapture,
    //KillerMove,
    //BadCapture,
    QuietMove
}

pub struct Moves {
    // We store the generated moves for each ply in a two dimensional array
    // used by the recursive search function. It must be able to store any
    // ply up to `MAX_PLY`, the theoretical maximum number of plies in a chess
    // game. And likewise it must be able to store the generated moves up to
    // the maximum of any chess position `MAX_MOVES`.
    lists: [[MoveExt; MAX_MOVES]; MAX_PLY],

    // Number of best moves at a given ply.
    best_moves_counts: [usize; MAX_PLY],

    // Number of moves at a given ply.
    sizes: [usize; MAX_PLY],

    // Index of the current move being searched at a given ply.
    indexes: [usize; MAX_PLY],

    pub skip_moves_ordering: bool,

    // Index of the ply currently searched.
    ply: usize,
}

impl Moves {
    pub fn new() -> Moves {
        Moves {
            lists: [[MoveExt::new(Move::new_null(), 0); MAX_MOVES]; MAX_PLY],
            sizes: [0; MAX_PLY],
            indexes: [0; MAX_PLY],
            best_moves_counts: [0; MAX_PLY],
            skip_moves_ordering: false,
            ply: 0,
        }
    }

    pub fn inc(&mut self) {
        self.ply += 1;
    }

    pub fn dec(&mut self) {
        // NOTE: the condition is only required if we use `moves.clear_all()`
        // in `root()` instead of `moves.clear()` that is used in `search()`
        // and `quiescence()`.
        if self.ply > 0 {
            self.ply -= 1;
        }
    }

    pub fn clear(&mut self) {
        self.sizes[self.ply] = 0;
        self.indexes[self.ply] = 0;
        self.best_moves_counts[self.ply] = 0;
    }

    pub fn clear_all(&mut self) {
        self.sizes = [0; MAX_PLY];
        self.indexes = [0; MAX_PLY];
        self.best_moves_counts = [0; MAX_PLY];
        self.ply = 0;
    }

    pub fn len_best_moves(&self) -> usize {
        self.best_moves_counts[self.ply]
    }

    pub fn len(&self) -> usize {
        self.sizes[self.ply]
    }

    pub fn index(&self) -> usize {
        self.indexes[self.ply]
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.sizes[self.ply] == 0
    }

    pub fn add_move(&mut self, m: Move, state: MovesState) {
        if !self.skip_moves_ordering {
            // Avoid adding again a best move
            let n = self.best_moves_counts[self.ply];
            for i in 0..n {
                if self.lists[self.ply][i].m == m {
                    return;
                }
            }
        }

        let score = match state {
            MovesState::BestMove => {
                self.best_moves_counts[self.ply] += 1;

                BEST_MOVE_SCORE
            },
            MovesState::QuietMove => {
                // NOTE: we cannot use MVV/LVA or SEE to assign a score to
                // captures here because we don't have access to the board
                // from `Moves`.
                QUIET_MOVE_SCORE
            }
        };

        self.lists[self.ply][self.sizes[self.ply]] = MoveExt::new(m, score);
        self.sizes[self.ply] += 1;
    }

    pub fn add_moves(&mut self, mut targets: Bitboard, dir: Direction, mt: MoveType) {
        while targets != 0 {
            let to = targets.trailing_zeros() as Square;
            debug_assert!((to as Direction) - dir >= 0);
            debug_assert!((to as Direction) - dir < 64);
            let from = ((to as Direction) - dir) as Square;
            let m = Move::new(from, to, mt);
            self.add_move(m, MovesState::QuietMove);
            targets.reset(to);
        }
    }

    pub fn add_moves_from(&mut self, mut targets: Bitboard, from: Square, mt: MoveType) {
        while targets != 0 {
            let to = targets.trailing_zeros() as Square;
            let m = Move::new(from, to, mt);
            self.add_move(m, MovesState::QuietMove);
            targets.reset(to);
        }
    }

    pub fn add_pawns_moves(&mut self, bitboards: &[Bitboard], side: Color, ep: Square) {
        const XDIRS: [Direction; 2] = [LEFT, RIGHT];
        const YDIRS: [Direction; 2] = [UP, DOWN];
        const FILES: [Bitboard; 2] = [FILE_A, FILE_H];
        const SEC_RANKS: [Bitboard; 2] = [RANK_3, RANK_6];
        const END_RANKS: [Bitboard; 2] = [RANK_8, RANK_1];

        let ydir = YDIRS[side as usize];

        let occupied = bitboards[WHITE as usize] | bitboards[BLACK as usize];

        let pushes = bitboards[(side | PAWN) as usize].shift(ydir) & !occupied;
        self.add_moves(pushes & !END_RANKS[side as usize], ydir, QUIET_MOVE);
        self.add_moves(pushes & END_RANKS[side as usize], ydir, KNIGHT_PROMOTION);
        self.add_moves(pushes & END_RANKS[side as usize], ydir, BISHOP_PROMOTION);
        self.add_moves(pushes & END_RANKS[side as usize], ydir, ROOK_PROMOTION);
        self.add_moves(pushes & END_RANKS[side as usize], ydir, QUEEN_PROMOTION);

        let double_pushes = (pushes & SEC_RANKS[side as usize]).shift(ydir) & !occupied;
        self.add_moves(double_pushes, 2 * ydir, DOUBLE_PAWN_PUSH);

        for i in 0..2 { // LEFT and RIGHT attacks
            let dir = ydir + XDIRS[i as usize];
            let attackers = bitboards[(side | PAWN) as usize] & !FILES[i];

            let targets = attackers.shift(dir);
            //let epb = 1 << ep; // FIXME: 1 << 64 == 0
            let epb = ((ep as u64 >> 6) ^ 1) << (ep % 64);
            self.add_moves(targets & epb, dir, EN_PASSANT);

            let attacks = targets & bitboards[(side ^ 1) as usize];

            self.add_moves(attacks & !END_RANKS[side as usize], dir, CAPTURE);
            self.add_moves(attacks & END_RANKS[side as usize], dir, KNIGHT_PROMOTION_CAPTURE);
            self.add_moves(attacks & END_RANKS[side as usize], dir, BISHOP_PROMOTION_CAPTURE);
            self.add_moves(attacks & END_RANKS[side as usize], dir, ROOK_PROMOTION_CAPTURE);
            self.add_moves(attacks & END_RANKS[side as usize], dir, QUEEN_PROMOTION_CAPTURE);
        }
    }

    pub fn add_knights_moves(&mut self, bitboards: &[Bitboard], side: Color) {
        let occupied = bitboards[WHITE as usize] | bitboards[BLACK as usize];
        let mut knights = bitboards[(side | KNIGHT) as usize];

        while knights > 0 {
            let from = knights.trailing_zeros() as Square;
            let targets = PIECE_MASKS[KNIGHT as usize][from as usize] & !occupied;
            self.add_moves_from(targets, from, QUIET_MOVE);
            let targets = PIECE_MASKS[KNIGHT as usize][from as usize] & bitboards[(side ^ 1) as usize];
            self.add_moves_from(targets, from, CAPTURE);
            knights.reset(from);
        }
    }

    pub fn add_king_moves(&mut self, bitboards: &[Bitboard], side: Color) {
        let occupied = bitboards[WHITE as usize] | bitboards[BLACK as usize];
        let mut kings = bitboards[(side | KING) as usize];

        while kings > 0 {
            let from = kings.trailing_zeros() as Square;
            let targets = PIECE_MASKS[KING as usize][from as usize] & !occupied;
            self.add_moves_from(targets, from, QUIET_MOVE);
            let targets = PIECE_MASKS[KING as usize][from as usize] & bitboards[(side ^ 1) as usize];
            self.add_moves_from(targets, from, CAPTURE);
            kings.reset(from);
        }
    }

    pub fn add_bishops_moves(&mut self, bitboards: &[Bitboard], side: Color) {
        let occupied = bitboards[WHITE as usize] | bitboards[BLACK as usize];
        let mut bishops = bitboards[(side | BISHOP) as usize];
        while bishops > 0 {
            let from = bishops.trailing_zeros() as Square;
            let targets = bishop_attacks(from, occupied);
            self.add_moves_from(targets & !occupied, from, QUIET_MOVE);
            self.add_moves_from(targets & bitboards[(side ^ 1) as usize], from, CAPTURE);
            bishops.reset(from);
        }
    }

    pub fn add_rooks_moves(&mut self, bitboards: &[Bitboard], side: Color) {
        let occupied = bitboards[WHITE as usize] | bitboards[BLACK as usize];
        let mut rooks = bitboards[(side | ROOK) as usize];
        while rooks > 0 {
            let from = rooks.trailing_zeros() as Square;
            let targets = rook_attacks(from, occupied);
            self.add_moves_from(targets & !occupied, from, QUIET_MOVE);
            self.add_moves_from(targets & bitboards[(side ^ 1) as usize], from, CAPTURE);
            rooks.reset(from);
        }
    }

    pub fn add_queens_moves(&mut self, bitboards: &[Bitboard], side: Color) {
        let occupied = bitboards[WHITE as usize] | bitboards[BLACK as usize];
        let mut queens = bitboards[(side | QUEEN) as usize];
        while queens > 0 {
            let from = queens.trailing_zeros() as Square;
            let targets = bishop_attacks(from, occupied) | rook_attacks(from, occupied);
            self.add_moves_from(targets & !occupied, from, QUIET_MOVE);
            self.add_moves_from(targets & bitboards[(side ^ 1) as usize], from, CAPTURE);

            queens.reset(from);
        }
    }

    pub fn add_king_castle(&mut self, side: Color) {
        let m = Move::new(E1 ^ 56 * side, G1 ^ 56 * side, KING_CASTLE);
        self.add_move(m, MovesState::QuietMove);
    }

    pub fn add_queen_castle(&mut self, side: Color) {
        let m = Move::new(E1 ^ 56 * side, C1 ^ 56 * side, QUEEN_CASTLE);
        self.add_move(m, MovesState::QuietMove);
    }

    pub fn swap(&mut self, i: usize, j: usize) {
        self.lists[self.ply].swap(i, j);
    }
}

impl Iterator for Moves {
    type Item = Move;

    fn next(&mut self) -> Option<Move> {
        let i = self.indexes[self.ply];
        let n = self.sizes[self.ply];

        self.indexes[self.ply] += 1;

        if i < n {
            if !self.skip_moves_ordering {
                /*
                // Find the next best move by selection sort
                let mut j = i;
                for k in (i + 1)..n {
                    if self.lists[self.ply][j].s < self.lists[self.ply][k].s {
                        j = k
                    }
                }

                // Swap it with the current next move
                if i != j {
                    self.lists[self.ply].swap(i, j);
                }
                */
            }

            Some(self.lists[self.ply][i].m)
        } else {
            None
        }
    }
}

impl Index<usize> for Moves {
    type Output = MoveExt;

    fn index(&self, index: usize) -> &MoveExt {
        &self.lists[self.ply][index]
    }
}

impl IndexMut<usize> for Moves {
    fn index_mut(&mut self, index: usize) -> &mut MoveExt {
        &mut self.lists[self.ply][index]
    }
}

#[cfg(test)]
mod tests {
    use common::*;
    use super::*;

    #[test]
    fn test_move_is_castle() {
        assert_eq!(Move::new(E2, E3, QUIET_MOVE).is_castle(), false);
        assert_eq!(Move::new(E1, G1, KING_CASTLE).is_castle(), true);
        assert_eq!(Move::new(E1, C1, QUEEN_CASTLE).is_castle(), true);
    }

    #[test]
    fn test_move_castle_kind() {
        assert_eq!(Move::new(E1, G1, KING_CASTLE).castle_kind(), KING);
        assert_eq!(Move::new(E1, C1, QUEEN_CASTLE).castle_kind(), QUEEN);
    }

    #[test]
    fn test_move_is_promotion() {
        assert_eq!(Move::new(E2, E3, QUIET_MOVE).is_promotion(), false);
        assert_eq!(Move::new(E7, E8, QUEEN_PROMOTION).is_promotion(), true);
        assert_eq!(Move::new(E7, D8, QUEEN_PROMOTION_CAPTURE).is_promotion(), true);
    }

    #[test]
    fn test_move_promotion_kind() {
        assert_eq!(Move::new(E7, E8, QUEEN_PROMOTION).promotion_kind(), QUEEN);
        assert_eq!(Move::new(E7, D8, ROOK_PROMOTION_CAPTURE).promotion_kind(), ROOK);
    }

    #[test]
    fn test_moves_ordering() {
        // NOTE: move ordering is now done outside of Moves to access the board
        let m1 = Move::new(D2, C2, CAPTURE);
        let m2 = Move::new(D2, C1, QUIET_MOVE);
        //let m3 = Move::new(D2, C3, CAPTURE);

        let mut moves = Moves::new();
        moves.add_move(m1, MovesState::BestMove);
        moves.add_move(m2, MovesState::QuietMove);
        //moves.add_move(m3, MovesState::QuietMove);

        println!("m1 = {}, {}", moves[0].m, moves[0].s);
        println!("m2 = {}, {}", moves[1].m, moves[1].s);
        //println!("m3 = {}, {}", moves[2].m, moves[2].s);

        assert_eq!(moves.next(), Some(m1));
        //assert_eq!(moves.next(), Some(m3));
        assert_eq!(moves.next(), Some(m2));
        assert_eq!(moves.next(), None);
    }
}
