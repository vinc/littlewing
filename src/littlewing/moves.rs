use littlewing::bitboard::Bitboard;
use littlewing::bitboard::BitwiseOperations;
use littlewing::piece::Piece;
use littlewing::square::Square;

#[deriving(Copy, PartialEq)]
pub enum MoveCategory {
    QuietMove,
    DoublePawnPush,
    KingCastle,
    QueenCastle,
    Capture,
    EnPassant,
    NullMove,
    KnightPromotion = 8,
    BishopPromotion,
    RookPromotion,
    QueenPromotion,
    KnightPromotionCapture,
    BishopPromotionCapture,
    RookPromotionCapture,
    QueenPromotionCapture
}

pub struct Move {
    pub from: Square,
    pub to: Square,
    pub category: MoveCategory
}

impl Move {
    pub fn new(f: Square, t: Square, mc: MoveCategory) -> Move {
        Move {
            from: f,
            to: t,
            category: mc
        }
    }
}

pub type Moves = Vec<Move>;

pub trait MovesOperations {
    fn add_moves(&mut self, mut targets: Bitboard, dir: uint, mc: MoveCategory);
}

impl MovesOperations for Moves {
    fn add_moves(&mut self, mut targets: Bitboard, dir: uint, mc: MoveCategory) {
        while targets != 0 {
            let to = targets.ffs();
            let from = to - dir;
            let m = Move::new(from, to, mc);
            self.push(m);
            targets.reset(to);
        }
    }
}
