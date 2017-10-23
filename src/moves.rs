use std::fmt;
use std::ops::{Index, IndexMut};

use color::*;
use piece::*;
use square::*;
use common::*;
use attack::*;
use piece::PieceChar;
use square::SquareExt;
use bitboard::{Bitboard, BitboardExt, BitboardIterator};

pub const BEST_MOVE_SCORE:    u8 = 255;
pub const KILLER_MOVE_SCORE:  u8 = 254;
pub const GOOD_CAPTURE_SCORE: u8 = 64;
pub const QUIET_MOVE_SCORE:   u8 = 0;

#[derive(Copy, Clone, PartialEq)]
pub struct Move(u16);

impl Move {
    pub fn new(from: Square, to: Square, mt: MoveType) -> Move {
        // from: 6 bits
        // to:   6 bits
        // mt:   4 bits
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
pub struct Scored<T, S> {
    pub item: T,
    pub score: S
}

impl<T, S> Scored<T, S> {
    pub fn new(item: T, score: S) -> Self {
        Scored { item: item, score: score }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
pub enum MovesStage { // If we don't care about `PartialOrd` we could do:
    BestMove   = 0,   // = 16 (BEST_MOVE)
    Capture    = 1,   // =  4 (CAPTURE)
    KillerMove = 2,   // = 17 (KILLER_MOVE)
    QuietMove  = 3,   // =  0 (QUIET_MOVE < CAPTURE)
    Done       = 4,
}

// Convert `MovesStage::Capture` into `CAPTURE`
// and `MovesStage::QuietMove` into `QUIET_MOVE`
// but does not work with other values of `MovesStage`
impl From<MovesStage> for MoveType {
    fn from(stage: MovesStage) -> Self {
        // Without `PartialOrd` we could do: stage as MoveType
        CAPTURE * ((stage == MovesStage::Capture) as MoveType)
    }
}

#[derive(Clone)]
pub struct Moves {
    killers: [[Move; MAX_KILLERS]; MAX_PLY],

    // We store the generated moves for each ply in a two dimensional array
    // used by the recursive search function. It must be able to store any
    // ply up to `MAX_PLY`, the theoretical maximum number of plies in a chess
    // game. And likewise it must be able to store the generated moves up to
    // the maximum of any chess position `MAX_MOVES`.
    lists: [[Scored<Move, u8>; MAX_MOVES]; MAX_PLY],

    // Number of moves at a given ply.
    sizes: [usize; MAX_PLY],

    // Index of the current move being searched at a given ply.
    indexes: [usize; MAX_PLY],

    stages: [MovesStage; MAX_PLY],

    pub skip_ordering: bool,
    pub skip_killers: bool,

    // Index of the ply currently searched.
    ply: usize,
}

impl Moves {
    pub fn new() -> Moves {
        Moves {
            killers: [[Move::new_null(); MAX_KILLERS]; MAX_PLY],
            lists: [[Scored::new(Move::new_null(), 0); MAX_MOVES]; MAX_PLY],
            sizes: [0; MAX_PLY],
            indexes: [0; MAX_PLY],
            stages: [MovesStage::BestMove; MAX_PLY],
            skip_ordering: false,
            skip_killers: false,
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
        self.stages[self.ply] = MovesStage::BestMove;
    }

    pub fn clear_all(&mut self) {
        self.killers = [[Move::new_null(); MAX_KILLERS]; MAX_PLY];
        self.sizes = [0; MAX_PLY];
        self.indexes = [0; MAX_PLY];
        self.stages = [MovesStage::BestMove; MAX_PLY];
        self.ply = 0;
    }

    pub fn len(&self) -> usize {
        self.sizes[self.ply]
    }

    pub fn index(&self) -> usize {
        self.indexes[self.ply]
    }

    pub fn stage(&self) -> MovesStage {
        self.stages[self.ply]
    }

    pub fn next_stage(&mut self) {
        self.stages[self.ply] = match self.stages[self.ply] {
            MovesStage::BestMove   => MovesStage::Capture,
            MovesStage::Capture    => MovesStage::KillerMove,
            MovesStage::KillerMove => MovesStage::QuietMove,
            MovesStage::QuietMove  => MovesStage::Done,
            MovesStage::Done       => panic!("no next stage")
        }
    }

    pub fn is_last_stage(&self) -> bool {
        // debug_assert(self.stages[self.ply] != MovesStage::Done);
        // self.stages[self.ply] == MovesStage::QuietMove
        self.stages[self.ply] >= MovesStage::QuietMove
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.sizes[self.ply] == 0
    }

    pub fn add_move(&mut self, m: Move) {
        // Avoid adding again a best move
        // NOTE: The best move is always first in the list, but the list
        // could contains previous entries so we need to check its current
        // length.
        if self.len() > 0 && self.lists[self.ply][0].item == m {
            return;
        }

        // Avoid adding again a killer move
        if self.stage() == MovesStage::QuietMove && !self.skip_killers {
            for &killer in &self.killers[self.ply] {
                if killer == m {
                    return;
                }
            }
        }

        // NOTE: we cannot use MVV/LVA or SEE to assign a score to captures
        // here because we don't have access to the board from `Moves`.
        let score = match self.stage() {
            MovesStage::BestMove   => BEST_MOVE_SCORE,
            MovesStage::Capture    => QUIET_MOVE_SCORE,
            MovesStage::KillerMove => KILLER_MOVE_SCORE,
            MovesStage::QuietMove  => QUIET_MOVE_SCORE,
            MovesStage::Done       => panic!("last stage")
        };

        self.lists[self.ply][self.sizes[self.ply]] = Scored::new(m, score);
        self.sizes[self.ply] += 1;
    }

    pub fn add_moves(&mut self, mut targets: Bitboard, dir: Direction, mt: MoveType) {
        while targets != 0 {
            let to = targets.trailing_zeros() as Square;
            debug_assert!((to as Direction) - dir >= 0);
            debug_assert!((to as Direction) - dir < 64);
            let from = ((to as Direction) - dir) as Square;
            let m = Move::new(from, to, mt);
            self.add_move(m);
            targets.reset(to);
        }
    }

    pub fn add_moves_from(&mut self, mut targets: Bitboard, from: Square, mt: MoveType) {
        while targets != 0 {
            let to = targets.trailing_zeros() as Square;
            let m = Move::new(from, to, mt);
            self.add_move(m);
            targets.reset(to);
        }
    }

    pub fn add_pawns_moves(&mut self, bitboards: &[Bitboard], side: Color, ep: Square) {
        let ydir = YDIRS[side as usize];
        let end_rank = END_RANKS[side as usize];

        match self.stage() {
            MovesStage::QuietMove => {
                let occupied = bitboards[WHITE as usize] | bitboards[BLACK as usize];

                let pushes = bitboards[(side | PAWN) as usize].shift(ydir) & !occupied;

                self.add_moves(pushes & !end_rank, ydir, QUIET_MOVE);
                self.add_moves(pushes & end_rank, ydir, KNIGHT_PROMOTION);
                self.add_moves(pushes & end_rank, ydir, BISHOP_PROMOTION);
                self.add_moves(pushes & end_rank, ydir, ROOK_PROMOTION);
                self.add_moves(pushes & end_rank, ydir, QUEEN_PROMOTION);

                let double_pushes = (pushes & SEC_RANKS[side as usize]).shift(ydir) & !occupied;
                self.add_moves(double_pushes, 2 * ydir, DOUBLE_PAWN_PUSH);
            },
            MovesStage::Capture => {
                for i in 0..2 { // LEFT and RIGHT attacks
                    let dir = ydir + XDIRS[i as usize];
                    let attackers = bitboards[(side | PAWN) as usize] & !END_FILES[i];

                    let targets = attackers.shift(dir);
                    //let epb = 1 << ep; // FIXME: 1 << 64 == 0
                    let epb = ((ep as u64 >> 6) ^ 1) << (ep % 64);
                    self.add_moves(targets & epb, dir, EN_PASSANT);

                    let attacks = targets & bitboards[(side ^ 1) as usize];

                    self.add_moves(attacks & !end_rank, dir, CAPTURE);
                    self.add_moves(attacks & end_rank, dir, KNIGHT_PROMOTION_CAPTURE);
                    self.add_moves(attacks & end_rank, dir, BISHOP_PROMOTION_CAPTURE);
                    self.add_moves(attacks & end_rank, dir, ROOK_PROMOTION_CAPTURE);
                    self.add_moves(attacks & end_rank, dir, QUEEN_PROMOTION_CAPTURE);
                }
            },
            _ => panic!("wrong generation stage")
        }
    }

    // NOTE: this method is the generic version of the next methods
    // but it's slower due to the `match` in `attacks` called in the loop.
    #[allow(dead_code)]
    pub fn add_moves_for(&mut self, p: Piece, bitboards: &[Bitboard], side: Color) {
        let occupied = bitboards[WHITE as usize] | bitboards[BLACK as usize];
        let mut pieces = bitboards[(side | p) as usize];
        let mt = MoveType::from(self.stage());
        let targets = match self.stage() {
            MovesStage::QuietMove => !occupied,
            MovesStage::Capture   => bitboards[(side ^ 1) as usize],
            _                     => panic!("wrong generation stage")
        };
        while let Some(from) = pieces.next() {
            let mask = piece_attacks(p, from, occupied);
            self.add_moves_from(targets & mask, from, mt);
        }
    }

    pub fn add_knights_moves(&mut self, bitboards: &[Bitboard], side: Color) {
        let occupied = bitboards[WHITE as usize] | bitboards[BLACK as usize];
        let mut knights = bitboards[(side | KNIGHT) as usize];
        let mt = MoveType::from(self.stage());
        let dests = match self.stage() {
            MovesStage::QuietMove => !occupied,
            MovesStage::Capture   => bitboards[(side ^ 1) as usize],
            _                     => panic!("wrong generation stage")
        };
        while let Some(from) = knights.next() {
            let mask = PIECE_MASKS[KNIGHT as usize][from as usize];
            let targets = dests & mask;
            self.add_moves_from(targets, from, mt);
        }
    }

    pub fn add_king_moves(&mut self, bitboards: &[Bitboard], side: Color) {
        let occupied = bitboards[WHITE as usize] | bitboards[BLACK as usize];
        let mut kings = bitboards[(side | KING) as usize];
        let mt = MoveType::from(self.stage());
        let dests = match self.stage() {
            MovesStage::QuietMove => !occupied,
            MovesStage::Capture   => bitboards[(side ^ 1) as usize],
            _                     => panic!("wrong generation stage")
        };
        while let Some(from) = kings.next() {
            let mask = PIECE_MASKS[KING as usize][from as usize];
            let targets = dests & mask;
            self.add_moves_from(targets, from, mt);
        }
    }

    pub fn add_bishops_moves(&mut self, bitboards: &[Bitboard], side: Color) {
        let occupied = bitboards[WHITE as usize] | bitboards[BLACK as usize];
        let mut bishops = bitboards[(side | BISHOP) as usize];
        let mt = MoveType::from(self.stage());
        let dests = match self.stage() {
            MovesStage::QuietMove => !occupied,
            MovesStage::Capture   => bitboards[(side ^ 1) as usize],
            _                     => panic!("wrong generation stage")
        };
        while let Some(from) = bishops.next() {
            let targets = bishop_attacks(from, occupied);
            self.add_moves_from(targets & dests, from, mt);
        }
    }

    pub fn add_rooks_moves(&mut self, bitboards: &[Bitboard], side: Color) {
        let occupied = bitboards[WHITE as usize] | bitboards[BLACK as usize];
        let mut rooks = bitboards[(side | ROOK) as usize];
        let mt = MoveType::from(self.stage());
        let dests = match self.stage() {
            MovesStage::QuietMove => !occupied,
            MovesStage::Capture   => bitboards[(side ^ 1) as usize],
            _                     => panic!("wrong generation stage")
        };
        while let Some(from) = rooks.next() {
            let targets = rook_attacks(from, occupied);
            self.add_moves_from(targets & dests, from, mt);
        }
    }

    pub fn add_queens_moves(&mut self, bitboards: &[Bitboard], side: Color) {
        let occupied = bitboards[WHITE as usize] | bitboards[BLACK as usize];
        let mut queens = bitboards[(side | QUEEN) as usize];
        let mt = MoveType::from(self.stage());
        let dests = match self.stage() {
            MovesStage::QuietMove => !occupied,
            MovesStage::Capture   => bitboards[(side ^ 1) as usize],
            _                     => panic!("wrong generation stage")
        };
        while let Some(from) = queens.next() {
            let targets = bishop_attacks(from, occupied) | rook_attacks(from, occupied);
            self.add_moves_from(targets & dests, from, mt);
        }
    }

    pub fn add_king_castle(&mut self, side: Color) {
        let m = Move::new(E1.flip(side), G1.flip(side), KING_CASTLE);
        self.add_move(m);
    }

    pub fn add_queen_castle(&mut self, side: Color) {
        let m = Move::new(E1.flip(side), C1.flip(side), QUEEN_CASTLE);
        self.add_move(m);
    }

    pub fn swap(&mut self, i: usize, j: usize) {
        self.lists[self.ply].swap(i, j);
    }

    pub fn get_killer_move(&mut self, i: usize) -> Move {
        self.killers[self.ply][i]
    }

    pub fn add_killer_move(&mut self, killer_move: Move) {
        debug_assert_eq!(MAX_KILLERS, 2);
        if killer_move != self.killers[self.ply][0] {
            self.killers[self.ply][1] = self.killers[self.ply][0];
            self.killers[self.ply][0] = killer_move;
        }
    }
}

impl Iterator for Moves {
    type Item = Move;

    fn next(&mut self) -> Option<Move> {
        let i = self.indexes[self.ply];
        let n = self.sizes[self.ply];

        if i < n {
            self.indexes[self.ply] += 1;
            /*
            if !self.skip_ordering {
                // Find the next best move by selection sort
                let mut j = i;
                for k in (i + 1)..n {
                    if self.lists[self.ply][j].score < self.lists[self.ply][k].score {
                        j = k
                    }
                }

                // Swap it with the current next move
                if i != j {
                    self.lists[self.ply].swap(i, j);
                }
            }
            */

            Some(self.lists[self.ply][i].item)
        } else {
            None
        }
    }
}

impl Index<usize> for Moves {
    type Output = Scored<Move, u8>;

    fn index(&self, index: usize) -> &Scored<Move, u8> {
        &self.lists[self.ply][index]
    }
}

impl IndexMut<usize> for Moves {
    fn index_mut(&mut self, index: usize) -> &mut Scored<Move, u8> {
        &mut self.lists[self.ply][index]
    }
}

#[cfg(test)]
mod tests {
    use piece::*;
    use common::*;
    use super::*;

    #[test]
    fn test_next_stage() {
        let mut moves = Moves::new();
        assert_eq!(moves.stage(), MovesStage::BestMove);
        moves.next_stage();
        assert_eq!(moves.stage(), MovesStage::Capture);
        moves.next_stage();
        assert_eq!(moves.stage(), MovesStage::KillerMove);
        moves.next_stage();
        assert_eq!(moves.stage(), MovesStage::QuietMove);
    }

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
        // TODO: rewrite this test

        // NOTE: move ordering is now done outside of Moves to access the board
        let m1 = Move::new(D2, C1, QUIET_MOVE);
        let m2 = Move::new(D2, C2, CAPTURE);
        //let m3 = Move::new(D2, C3, CAPTURE);

        let mut moves = Moves::new();
        moves.add_move(m1);
        moves.next_stage(); // From BestMove to Capture
        moves.add_move(m2);
        moves.next_stage(); // From Capture to QuietMove
        //moves.add_move(m3);

        println!("m1 = {}, {}", moves[0].item, moves[0].score);
        println!("m2 = {}, {}", moves[1].item, moves[1].score);
        //println!("m3 = {}, {}", moves[2].m, moves[2].s);

        assert_eq!(moves.next(), Some(m1));
        //assert_eq!(moves.next(), Some(m3));
        assert_eq!(moves.next(), Some(m2));
        assert_eq!(moves.next(), None);
    }
}
