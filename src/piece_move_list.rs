use crate::std::prelude::v1::*;
use crate::std::ops::{Index, IndexMut};

use crate::color::*;
use crate::piece::*;
use crate::square::*;
use crate::common::*;
use crate::attack::*;
use crate::piece_move::*;
use crate::square::SquareExt;
use crate::bitboard::{Bitboard, BitboardExt, BitboardIterator};
use crate::hyperbola::bishop_attacks;
use crate::hyperbola::rook_attacks;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Scored<T, S> {
    pub item: T,
    pub score: S
}

impl<T, S> Scored<T, S> {
    pub fn new(item: T, score: S) -> Self {
        Scored { item, score }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Eq, PartialEq, PartialOrd, Debug)]
pub enum PieceMoveListStage { // If we don't care about `PartialOrd` we could do:
    BestPieceMove   = 0,      // = 16 (BEST_MOVE)
    Capture         = 1,      // =  4 (CAPTURE)
    KillerPieceMove = 2,      // = 17 (KILLER_MOVE)
    QuietPieceMove  = 3,      // =  0 (QUIET_MOVE < CAPTURE)
    Done            = 4,
}

// Convert `PieceMoveListStage::Capture` into `CAPTURE`
// and `PieceMoveListStage::QuietPieceMove` into `QUIET_MOVE`
// but does not work with other values of `PieceMoveListStage`
impl From<PieceMoveListStage> for PieceMoveType {
    fn from(stage: PieceMoveListStage) -> Self {
        // Without `PartialOrd` we could do: stage as PieceMoveType
        CAPTURE * ((stage == PieceMoveListStage::Capture) as PieceMoveType)
    }
}

#[derive(Clone)]
pub struct PieceMoveList {
    killers: [[PieceMove; MAX_KILLERS]; MAX_PLY],

    // We store the generated moves for each ply in a two dimensional array
    // used by the recursive search function. It must be able to store any
    // ply up to `MAX_PLY`, the theoretical maximum number of plies in a chess
    // game. And likewise it must be able to store the generated moves up to
    // the maximum of any chess position `MAX_MOVES`.
    lists: [[Scored<PieceMove, u8>; MAX_MOVES]; MAX_PLY],

    // Number of moves at a given ply.
    sizes: [usize; MAX_PLY],

    // Index of the current move being searched at a given ply.
    indexes: [usize; MAX_PLY],

    stages: [PieceMoveListStage; MAX_PLY],

    pub skip_ordering: bool,
    pub skip_killers: bool,

    // Index of the ply currently searched.
    ply: usize,
}

impl PieceMoveList {
    pub fn new() -> PieceMoveList {
        PieceMoveList {
            killers: [[PieceMove::new_null(); MAX_KILLERS]; MAX_PLY],
            lists: [[Scored::new(PieceMove::new_null(), 0); MAX_MOVES]; MAX_PLY],
            sizes: [0; MAX_PLY],
            indexes: [0; MAX_PLY],
            stages: [PieceMoveListStage::BestPieceMove; MAX_PLY],
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
        self.stages[self.ply] = PieceMoveListStage::BestPieceMove;
    }

    pub fn clear_all(&mut self) {
        self.killers = [[PieceMove::new_null(); MAX_KILLERS]; MAX_PLY];
        self.sizes = [0; MAX_PLY];
        self.indexes = [0; MAX_PLY];
        self.stages = [PieceMoveListStage::BestPieceMove; MAX_PLY];
        self.ply = 0;
    }

    pub fn len(&self) -> usize {
        self.sizes[self.ply]
    }

    pub fn index(&self) -> usize {
        self.indexes[self.ply]
    }

    pub fn stage(&self) -> PieceMoveListStage {
        self.stages[self.ply]
    }

    pub fn next_stage(&mut self) {
        self.stages[self.ply] = match self.stages[self.ply] {
            PieceMoveListStage::BestPieceMove   => PieceMoveListStage::Capture,
            PieceMoveListStage::Capture    => PieceMoveListStage::KillerPieceMove,
            PieceMoveListStage::KillerPieceMove => PieceMoveListStage::QuietPieceMove,
            PieceMoveListStage::QuietPieceMove  => PieceMoveListStage::Done,
            PieceMoveListStage::Done       => panic!("no next stage")
        }
    }

    pub fn is_last_stage(&self) -> bool {
        // debug_assert(self.stages[self.ply] != PieceMoveListStage::Done);
        // self.stages[self.ply] == PieceMoveListStage::QuietPieceMove
        self.stages[self.ply] >= PieceMoveListStage::QuietPieceMove
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.sizes[self.ply] == 0
    }

    pub fn add_move(&mut self, m: PieceMove) {
        // Avoid adding again a best move
        // NOTE: The best move is always first in the list, but the list
        // could contains previous entries so we need to check its current
        // length.
        if self.len() > 0 && self.lists[self.ply][0].item == m {
            return;
        }

        // Avoid adding again a killer move
        if self.stage() == PieceMoveListStage::QuietPieceMove && !self.skip_killers {
            for &killer in &self.killers[self.ply] {
                if killer == m {
                    return;
                }
            }
        }

        // NOTE: we cannot use MVV/LVA or SEE to assign a score to captures
        // here because we don't have access to the board from `PieceMoveList`.
        let score = match self.stage() {
            PieceMoveListStage::BestPieceMove   => BEST_MOVE_SCORE,
            PieceMoveListStage::Capture    => QUIET_MOVE_SCORE,
            PieceMoveListStage::KillerPieceMove => KILLER_MOVE_SCORE,
            PieceMoveListStage::QuietPieceMove  => QUIET_MOVE_SCORE,
            PieceMoveListStage::Done       => panic!("last stage")
        };

        self.lists[self.ply][self.sizes[self.ply]] = Scored::new(m, score);
        self.sizes[self.ply] += 1;
    }

    pub fn add_moves(&mut self, mut targets: Bitboard, dir: Shift, mt: PieceMoveType) {
        while targets != 0 {
            let to = targets.scan() as Square;
            debug_assert!((to as Shift) - dir >= 0);
            debug_assert!((to as Shift) - dir < 64);
            let from = ((to as Shift) - dir) as Square;
            let m = PieceMove::new(from, to, mt);
            self.add_move(m);
            targets.reset(to);
        }
    }

    pub fn add_moves_from(&mut self, mut targets: Bitboard, from: Square, mt: PieceMoveType) {
        while targets != 0 {
            let to = targets.scan() as Square;
            let m = PieceMove::new(from, to, mt);
            self.add_move(m);
            targets.reset(to);
        }
    }

    pub fn add_pawns_moves(&mut self, bitboards: &[Bitboard], side: Color, ep: Square) {
        let ydir = YSHIFTS[side as usize];
        let end_rank = END_RANKS[side as usize];

        match self.stage() {
            PieceMoveListStage::QuietPieceMove => {
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
            PieceMoveListStage::Capture => {
                for i in 0..2 { // LEFT and RIGHT attacks
                    let dir = ydir + XSHIFTS[i as usize];
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
        let mt = PieceMoveType::from(self.stage());
        let targets = match self.stage() {
            PieceMoveListStage::QuietPieceMove => !occupied,
            PieceMoveListStage::Capture        => bitboards[(side ^ 1) as usize],
            _                                  => panic!("wrong generation stage")
        };
        while let Some(from) = pieces.next() {
            let mask = piece_attacks(p, from, occupied);
            self.add_moves_from(targets & mask, from, mt);
        }
    }

    pub fn add_knights_moves(&mut self, bitboards: &[Bitboard], side: Color) {
        let occupied = bitboards[WHITE as usize] | bitboards[BLACK as usize];
        let mut knights = bitboards[(side | KNIGHT) as usize];
        let mt = PieceMoveType::from(self.stage());
        let dests = match self.stage() {
            PieceMoveListStage::QuietPieceMove => !occupied,
            PieceMoveListStage::Capture        => bitboards[(side ^ 1) as usize],
            _                                  => panic!("wrong generation stage")
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
        let mt = PieceMoveType::from(self.stage());
        let dests = match self.stage() {
            PieceMoveListStage::QuietPieceMove => !occupied,
            PieceMoveListStage::Capture        => bitboards[(side ^ 1) as usize],
            _                                  => panic!("wrong generation stage")
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
        let mt = PieceMoveType::from(self.stage());
        let dests = match self.stage() {
            PieceMoveListStage::QuietPieceMove => !occupied,
            PieceMoveListStage::Capture        => bitboards[(side ^ 1) as usize],
            _                                  => panic!("wrong generation stage")
        };
        while let Some(from) = bishops.next() {
            let targets = bishop_attacks(from, occupied);
            self.add_moves_from(targets & dests, from, mt);
        }
    }

    pub fn add_rooks_moves(&mut self, bitboards: &[Bitboard], side: Color) {
        let occupied = bitboards[WHITE as usize] | bitboards[BLACK as usize];
        let mut rooks = bitboards[(side | ROOK) as usize];
        let mt = PieceMoveType::from(self.stage());
        let dests = match self.stage() {
            PieceMoveListStage::QuietPieceMove => !occupied,
            PieceMoveListStage::Capture        => bitboards[(side ^ 1) as usize],
            _                                  => panic!("wrong generation stage")
        };
        while let Some(from) = rooks.next() {
            let targets = rook_attacks(from, occupied);
            self.add_moves_from(targets & dests, from, mt);
        }
    }

    pub fn add_queens_moves(&mut self, bitboards: &[Bitboard], side: Color) {
        let occupied = bitboards[WHITE as usize] | bitboards[BLACK as usize];
        let mut queens = bitboards[(side | QUEEN) as usize];
        let mt = PieceMoveType::from(self.stage());
        let dests = match self.stage() {
            PieceMoveListStage::QuietPieceMove => !occupied,
            PieceMoveListStage::Capture        => bitboards[(side ^ 1) as usize],
            _                                  => panic!("wrong generation stage")
        };
        while let Some(from) = queens.next() {
            let targets = bishop_attacks(from, occupied) | rook_attacks(from, occupied);
            self.add_moves_from(targets & dests, from, mt);
        }
    }

    pub fn add_king_castle(&mut self, side: Color) {
        let m = PieceMove::new(E1.flip(side), G1.flip(side), KING_CASTLE);
        self.add_move(m);
    }

    pub fn add_queen_castle(&mut self, side: Color) {
        let m = PieceMove::new(E1.flip(side), C1.flip(side), QUEEN_CASTLE);
        self.add_move(m);
    }

    pub fn swap(&mut self, i: usize, j: usize) {
        self.lists[self.ply].swap(i, j);
    }

    pub fn get_killer_move(&mut self, i: usize) -> PieceMove {
        self.killers[self.ply][i]
    }

    pub fn add_killer_move(&mut self, killer_move: PieceMove) {
        debug_assert_eq!(MAX_KILLERS, 2);
        if killer_move != self.killers[self.ply][0] {
            self.killers[self.ply][1] = self.killers[self.ply][0];
            self.killers[self.ply][0] = killer_move;
        }
    }
}

impl Iterator for PieceMoveList {
    type Item = PieceMove;

    fn next(&mut self) -> Option<PieceMove> {
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

impl Index<usize> for PieceMoveList {
    type Output = Scored<PieceMove, u8>;

    fn index(&self, index: usize) -> &Scored<PieceMove, u8> {
        &self.lists[self.ply][index]
    }
}

impl IndexMut<usize> for PieceMoveList {
    fn index_mut(&mut self, index: usize) -> &mut Scored<PieceMove, u8> {
        &mut self.lists[self.ply][index]
    }
}

#[cfg(test)]
mod tests {
    use crate::common::*;
    use super::*;

    #[test]
    fn test_next_stage() {
        let mut moves = PieceMoveList::new();
        assert_eq!(moves.stage(), PieceMoveListStage::BestPieceMove);
        moves.next_stage();
        assert_eq!(moves.stage(), PieceMoveListStage::Capture);
        moves.next_stage();
        assert_eq!(moves.stage(), PieceMoveListStage::KillerPieceMove);
        moves.next_stage();
        assert_eq!(moves.stage(), PieceMoveListStage::QuietPieceMove);
    }

    #[test]
    fn test_moves_ordering() {
        // TODO: rewrite this test

        // NOTE: move ordering is now done outside of PieceMoveList to access the board
        let m1 = PieceMove::new(D2, C1, QUIET_MOVE);
        let m2 = PieceMove::new(D2, C2, CAPTURE);
        //let m3 = PieceMove::new(D2, C3, CAPTURE);

        let mut moves = PieceMoveList::new();
        moves.add_move(m1);
        moves.next_stage(); // From BestPieceMove to Capture
        moves.add_move(m2);
        moves.next_stage(); // From Capture to QuietPieceMove
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
