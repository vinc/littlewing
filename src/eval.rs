use common::*;
use attack::piece_attacks;
use bitboard::{BitboardExt, BitboardIterator};
use game::Game;

pub const PAWN_VALUE:   Score = 100;
pub const KNIGHT_VALUE: Score = 325;
pub const BISHOP_VALUE: Score = 325;
pub const ROOK_VALUE:   Score = 500;
pub const QUEEN_VALUE:  Score = 965; // Rook + Minor + Pawn + Bishop Pair
pub const KING_VALUE:   Score = 10000;

lazy_static! {
    pub static ref PIECE_VALUES: [Score; 14] = {
        let mut piece_values = [0; 14];

        piece_values[PAWN   as usize] = PAWN_VALUE;
        piece_values[KNIGHT as usize] = KNIGHT_VALUE;
        piece_values[BISHOP as usize] = BISHOP_VALUE;
        piece_values[ROOK   as usize] = ROOK_VALUE;
        piece_values[QUEEN  as usize] = QUEEN_VALUE;
        piece_values[KING   as usize] = KING_VALUE;

        for i in 0..7 {
            let j = i * 2;
            piece_values[j + 1] = piece_values[j];
        }

        piece_values
    };
}

pub trait Eval {
    fn eval_pieces(&self, piece: Piece) -> Score;
    fn eval_side(&self, c: Color) -> Score;
    fn eval(&self) -> Score;
}

impl Eval for Game {
    fn eval_pieces(&self, piece: Piece) -> Score {
        let mut score = 0;

        // Material score
        let n = self.bitboards[piece as usize].count() as Score;
        score += n * PIECE_VALUES[piece as usize];

        // Mobility score
        let occupied = self.bitboards[WHITE as usize] | self.bitboards[BLACK as usize];
        let mut pieces = self.bitboards[piece as usize];
        while let Some(from) = pieces.next() {
            let targets = piece_attacks(piece, from, occupied);
            score += targets.count() as Score;
        }

        score
    }

    fn eval_side(&self, c: Color) -> Score {
        let mut score = 0;

        score += self.eval_pieces(c | PAWN);
        score += self.eval_pieces(c | KNIGHT);
        score += self.eval_pieces(c | BISHOP);
        score += self.eval_pieces(c | ROOK);
        score += self.eval_pieces(c | QUEEN);
        score += self.eval_pieces(c | KING);

        score
    }

    fn eval(&self) -> Score {
        let mut score = 0;
        let side = self.positions.top().side;

        score += self.eval_side(side);
        score -= self.eval_side(side ^ 1);

        if score > KING_VALUE {
            return INF; // Win
        } else if score < -KING_VALUE {
            return -INF; // Loss
        }

        score
    }
}

/*
#[cfg(test)]
mod tests {
    extern crate test;
    use self::test::Bencher;
    use common::*;
    use game::Game;
    use eval::Eval;
    use fen::FEN;

    #[bench]
    fn bench_eval(b: &mut Bencher) {
        let game = Game::from_fen(DEFAULT_FEN);
        b.iter(|| {
            game.eval()
        })
    }
}
*/
