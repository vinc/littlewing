use std::cmp;

use common::*;
use attack::Attack;
use attack::piece_attacks;
use bitboard::{BitboardExt, BitboardIterator};
use game::Game;
use moves::Move;

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
    fn see(&self, capture: Move) -> Score;
    fn lvp(&self, side: Color, attacks: Bitboard, occupied: Bitboard) -> Square;
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

    // Static Exchange Evaluation
    fn see(&self, capture: Move) -> Score {
        let mut occupied = self.bitboards[WHITE as usize] | self.bitboards[BLACK as usize];
        let mut sq = capture.from();
        let mut side = self.positions.top().side;
        let mut gains = [0; 32];
        let mut d = 0;

        let piece = self.board[capture.to() as usize];
        let value = PIECE_VALUES[piece as usize];
        gains[d] = value;

        while sq != OUT {
            d += 1;
            side ^= 1;
            occupied.reset(sq); // Remove piece

            let piece = self.board[sq as usize];
            let value = PIECE_VALUES[piece as usize];
            gains[d] = value - gains[d - 1];

            // Get square of least valuable piece remaining
            let attacks = self.attacks_to(capture.to(), occupied);
            sq = self.lvp(side, attacks, occupied);
        }

        while { d -= 1; d > 0 } {
            gains[d - 1] = -cmp::max(-gains[d - 1], gains[d]);
        }

        gains[0]
    }

    // Get square of least valuable piece
    fn lvp(&self, side: Color, attacks: Bitboard, occupied: Bitboard) -> Square {
        for p in PIECES.iter() {
            let piece = side | p;
            // NOTE: we need `occupied` only to be able to hide some pieces
            // from the bitboard.
            let subset = attacks & occupied & self.bitboards[piece as usize];
            if subset > 0 {
                return subset.trailing_zeros() as Square
            }
        }

        OUT
    }
}

#[cfg(test)]
mod tests {
    use common::*;
    use super::*;
    use fen::FEN;
    use game::Game;
    use moves::Move;

    #[test]
    fn test_see() {
        let fen = "1k1r4/1pp4p/p7/4p3/8/P5P1/1PP4P/2K1R3 w - -";
        let game = Game::from_fen(fen);
        assert_eq!(game.see(Move::new(E1, E5, CAPTURE)), 100);

        let fen = "1k1r3q/1ppn3p/p4b2/4p3/8/P2N2P1/1PP1R1BP/2K1Q3 w - -";
        let game = Game::from_fen(fen);
        assert_eq!(game.see(Move::new(D3, E5, CAPTURE)), -225);

        let fen = "rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 2";
        let game = Game::from_fen(fen);
        assert_eq!(game.see(Move::new(E4, D5, CAPTURE)), 0);

        let fen = "rnbqkb1r/ppp1pppp/5n2/3p4/4P3/2N5/PPPP1PPP/R1BQKBNR w KQkq - 2 3";
        let game = Game::from_fen(fen);
        assert_eq!(game.see(Move::new(E4, D5, CAPTURE)), 0);

        let fen = "rnbqkb1r/pp2pppp/2p2n2/1B1p4/4P3/2N5/PPPP1PPP/R1BQK1NR w KQkq - 0 4";
        let game = Game::from_fen(fen);
        assert_eq!(game.see(Move::new(E4, D5, CAPTURE)), 0);
        assert_eq!(game.see(Move::new(C3, D5, CAPTURE)), -225);
        assert_eq!(game.see(Move::new(B5, C6, CAPTURE)), -225);

        let fen = "rnbqkbnr/pppp1ppp/8/4p3/3P4/8/PPP1PPPP/RNBQKBNR w KQkq e6 0 2";
        let game = Game::from_fen(fen);
        assert_eq!(game.see(Move::new(D4, E5, CAPTURE)), 100);
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
