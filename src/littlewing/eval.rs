use littlewing::common::*;
use littlewing::game::Game;

pub const PAWN_VALUE:   i32 = 100;
pub const KNIGHT_VALUE: i32 = 325;
pub const BISHOP_VALUE: i32 = 325;
pub const ROOK_VALUE:   i32 = 500;
pub const QUEEN_VALUE:  i32 = 965; // Rook + Minor + Pawn + Bishop Pair
pub const KING_VALUE:   i32 = 10000;

pub trait Eval {
    fn eval_pieces(&self, c: Color) -> i32;
    fn eval(&self) -> i32;
}

impl Eval for Game {
    fn eval_pieces(&self, c: Color) -> i32 {
        let mut score = 0;

        score += PAWN_VALUE * self.bitboards[(c | PAWN) as usize].count_ones() as i32;
        score += KNIGHT_VALUE * self.bitboards[(c | KNIGHT) as usize].count_ones() as i32;
        score += BISHOP_VALUE * self.bitboards[(c | BISHOP) as usize].count_ones() as i32;
        score += ROOK_VALUE * self.bitboards[(c | ROOK) as usize].count_ones() as i32;
        score += QUEEN_VALUE * self.bitboards[(c | QUEEN) as usize].count_ones() as i32;
        score += KING_VALUE * self.bitboards[(c | KING) as usize].count_ones() as i32;

        score
    }

    fn eval(&self) -> i32 {
        let mut score = 0;
        let side = self.positions.top().side;

        score += self.eval_pieces(side);
        score -= self.eval_pieces(side ^ 1);

        score
    }
}

/*
#[cfg(test)]
mod tests {
    extern crate test;
    use self::test::Bencher;
    use littlewing::common::*;
    use littlewing::game::Game;
    use littlewing::eval::Eval;
    use littlewing::fen::FEN;

    #[bench]
    fn bench_eval(b: &mut Bencher) {
        let game: Game = FEN::from_fen(DEFAULT_FEN);
        b.iter(|| {
            game.eval()
        })
    }
}
*/
