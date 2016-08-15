use common::*;
use game::Game;

pub const PAWN_VALUE:   Score = 100;
pub const KNIGHT_VALUE: Score = 325;
pub const BISHOP_VALUE: Score = 325;
pub const ROOK_VALUE:   Score = 500;
pub const QUEEN_VALUE:  Score = 965; // Rook + Minor + Pawn + Bishop Pair
pub const KING_VALUE:   Score = 10000;

pub trait Eval {
    fn eval_pieces(&self, c: Color) -> Score;
    fn eval(&self) -> Score;
}

impl Eval for Game {
    fn eval_pieces(&self, c: Color) -> Score {
        let mut score = 0;

        score += PAWN_VALUE * self.bitboards[(c | PAWN) as usize].count_ones() as Score;
        score += KNIGHT_VALUE * self.bitboards[(c | KNIGHT) as usize].count_ones() as Score;
        score += BISHOP_VALUE * self.bitboards[(c | BISHOP) as usize].count_ones() as Score;
        score += ROOK_VALUE * self.bitboards[(c | ROOK) as usize].count_ones() as Score;
        score += QUEEN_VALUE * self.bitboards[(c | QUEEN) as usize].count_ones() as Score;
        score += KING_VALUE * self.bitboards[(c | KING) as usize].count_ones() as Score;

        score
    }

    fn eval(&self) -> Score {
        let mut score = 0;
        let side = self.positions.top().side;

        score += self.eval_pieces(side);
        score -= self.eval_pieces(side ^ 1);

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
        let game: Game = FEN::from_fen(DEFAULT_FEN);
        b.iter(|| {
            game.eval()
        })
    }
}
*/
