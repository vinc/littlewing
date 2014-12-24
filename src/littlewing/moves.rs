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
