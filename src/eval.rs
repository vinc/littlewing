use std::cmp;

use color::*;
use piece::*;
use square::*;
use common::*;
use attack::Attack;
use attack::piece_attacks;
use bitboard::{Bitboard, BitboardExt, BitboardIterator};
use bitboard::filefill;
use game::Game;
use moves::Move;
use pst::{PST_OPENING, PST_ENDING};

pub const PAWN_VALUE:       Score =   100;
pub const KNIGHT_VALUE:     Score =   350;
pub const BISHOP_VALUE:     Score =   350;
pub const ROOK_VALUE:       Score =   500;
pub const QUEEN_VALUE:      Score =  1000; // R + B + P + bonus bishop pair
pub const KING_VALUE:       Score = 10000;

const BONUS_BISHOP_PAIR:    Score =    50;
const BONUS_HALF_OPEN_FILE: Score =     5;
const BONUS_KNIGHT_PAWNS:   Score =     5;
const BONUS_ROOK_OPEN_FILE: Score =    20;
const BONUS_ROOK_PAWNS:     Score =     5;
const MALUS_DOUBLED_PAWN:   Score =   -10;

lazy_static! {
    static ref PIECE_VALUES: [Score; 14] = {
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

    static ref PST: [[[Score; 2]; 64]; 14] = {
        let mut pst_values = [[[0; 2]; 64]; 14];

        for c in 0..2 {
            for p in 0..6 {
                for s in 0..64 {
                    let square = (s as Square).flip((c as Color) ^ 1);
                    let piece = (c as Color) | PIECES[p];

                    let score = PST_OPENING[p][s] as Score;
                    pst_values[piece as usize][square as usize][0] = score;

                    let score = PST_ENDING[p][s] as Score;
                    pst_values[piece as usize][square as usize][1] = score;
                }
            }
        }

        pst_values
    };
}

/// Evaluation algorithms
pub trait Eval {
    /// Evaluate the current position
    fn eval(&self) -> Score;

    /// Evaluate piece squate table at the current position for the given side
    fn eval_pst(&self, c: Color) -> Score;

    /// Evaluate material at the current position for the given side
    fn eval_material(&self, c: Color) -> Score;

    /// Evaluate mobility at the current position for the given side
    fn eval_mobility(&self, c: Color) -> Score;

    /// Static Exchange Evaluation
    fn see(&self, capture: Move) -> Score;
}

trait EvalExt {
    fn eval_ending(&self, c: Color) -> Option<Score>;
    fn lvp(&self, side: Color, attacks: Bitboard, occupied: Bitboard) -> Square;
}

impl Eval for Game {
    fn eval(&self) -> Score {
        let occupied = self.bitboard(WHITE) | self.bitboard(BLACK);
        let side = self.positions.top().side;

        // Look for win/loss/draw
        if let Some(score) = self.eval_ending(side) {
            return score;
        }

        let mut score = 0;

        let mut material = [0, 0];
        let mut mobility = [0, 0];
        let mut position = [[0, 0], [0, 0]]; // Opening and ending phases

        for &c in &COLORS {
            for &p in &PIECES {
                let piece = c | p;
                let mut pieces = self.bitboards[piece as usize];
                let n = pieces.count() as Score;
                material[c as usize] += n * PIECE_VALUES[piece as usize];
                while let Some(square) = pieces.next() {
                    let targets = piece_attacks(piece, square, occupied);
                    mobility[c as usize] += targets.count() as Score;
                    position[c as usize][0] += PST[piece as usize][square as usize][0];
                    position[c as usize][1] += PST[piece as usize][square as usize][1];
                }
            }
        }

        let c = side as usize;

        // Linear interpolation between opening and ending scores
        // based on the number of pieces on the board
        let x0 = 32; // Max
        let x1 = 2; // Min
        let x = occupied.count() as Score; // Current

        let y0 = position[c][0];
        let y1 = position[c][1];
        score += (y0 * (x1 - x) + y1 * (x - x0)) / (x1 - x0);
        score += material[c];
        score += mobility[c];

        let y0 = position[c ^ 1][0];
        let y1 = position[c ^ 1][1];
        score -= (y0 * (x1 - x) + y1 * (x - x0)) / (x1 - x0);
        score -= material[c ^ 1];
        score -= mobility[c ^ 1];

        score
    }

    fn eval_material(&self, c: Color) -> Score {
        let mut score = 0;
        let mut pawns_count = 0;

        let color_pawns = self.bitboards[(c | PAWN) as usize];
        let other_pawns = self.bitboards[(c ^ 1 | PAWN) as usize];

        let open_files = open_files(color_pawns, other_pawns);

        let half_open_files = half_open_files(color_pawns, other_pawns);
        let half_open_files_count = (half_open_files & RANK_1).count() as Score;
        score += half_open_files_count * BONUS_HALF_OPEN_FILE;

        for &p in &PIECES {
            let piece = c | p;
            let pieces = self.bitboards[piece as usize];
            let n = pieces.count() as Score;
            score += n * PIECE_VALUES[piece as usize];

            match p {
                PAWN => {
                    pawns_count = n;

                    let pawns_files_count = (filefill(pieces) & RANK_1).count() as Score;
                    score += (pawns_count - pawns_files_count) * MALUS_DOUBLED_PAWN;
                },
                KNIGHT => {
                    score += n * pawns_count * BONUS_KNIGHT_PAWNS;
                },
                BISHOP if n == 2 => {
                    score += BONUS_BISHOP_PAIR;
                },
                ROOK => {
                    score += ((pieces & open_files).count() as Score) * BONUS_ROOK_OPEN_FILE;
                    score += ((pieces & half_open_files).count() as Score) * BONUS_ROOK_OPEN_FILE / 2;
                    score += n * (8 - pawns_count) * BONUS_ROOK_PAWNS;
                },
                _ => { }
            }
        }

        score
    }

    fn eval_pst(&self, c: Color) -> Score {
        let mut score_0 = 0; // Opening score
        let mut score_1 = 0; // Ending score

        let occupied = self.bitboards[WHITE as usize] | self.bitboards[BLACK as usize];

        for p in &PIECES {
            let piece = c | p;
            let mut pieces = self.bitboards[piece as usize];
            while let Some(square) = pieces.next() {
                score_0 += PST[piece as usize][square as usize][0];
                score_1 += PST[piece as usize][square as usize][1];
            }
        }

        let x0 = 32;
        let x1 = 2;
        let x = occupied.count() as Score;

        let y0 = score_0;
        let y1 = score_1;

        // Linear interpolation between opening and ending scores
        (y0 * (x1 - x) + y1 * (x - x0)) / (x1 - x0)
    }

    fn eval_mobility(&self, c: Color) -> Score {
        let mut score = 0;

        let occupied = self.bitboards[WHITE as usize] | self.bitboards[BLACK as usize];
        for p in &PIECES {
            let piece = c | p;
            let mut pieces = self.bitboards[piece as usize];
            while let Some(from) = pieces.next() {
                let targets = piece_attacks(piece, from, occupied);
                score += targets.count() as Score;
            }
        }

        score
    }

    fn see(&self, capture: Move) -> Score {
        let mut occupied = self.bitboard(WHITE) | self.bitboard(BLACK);
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
}

impl EvalExt for Game {
    fn eval_ending(&self, side: Color) -> Option<Score> {
        let occupied = self.bitboard(WHITE) | self.bitboard(BLACK);

        let kings = self.bitboard(WHITE | KING) | self.bitboard(BLACK | KING);
        if kings.count() < 2 {
            if self.bitboard(side | KING).count() == 0 {
                return Some(-INF); // Loss
            } else {
                return Some(INF); // Win
            }
        }

        // Draw by insufficient material
        if occupied.count() < 4 {
            let knights = self.bitboard(WHITE | KNIGHT) | self.bitboard(BLACK | KNIGHT);
            let bishops = self.bitboard(WHITE | BISHOP) | self.bitboard(BLACK | BISHOP);
            if (kings | knights | bishops) == occupied {
                return Some(0); // Draw
            }
        }

        None
    }

    // Get square of least valuable piece
    fn lvp(&self, side: Color, attacks: Bitboard, occupied: Bitboard) -> Square {
        for p in &PIECES {
            let piece = side | p;
            // NOTE: we need `occupied` only to be able to hide some pieces
            // from the bitboard.
            let subset = attacks & occupied & self.bitboards[piece as usize];
            if subset > 0 {
                return subset.trailing_zeros() as Square;
            }
        }

        OUT
    }
}

#[allow(dead_code)]
fn closed_files(white_pawns: Bitboard, black_pawns: Bitboard) -> Bitboard {
    filefill(white_pawns) & filefill(black_pawns)
}

fn open_files(white_pawns: Bitboard, black_pawns: Bitboard) -> Bitboard {
    !filefill(white_pawns) & !filefill(black_pawns)
}

fn half_open_files(pawns: Bitboard, opponent_pawns: Bitboard) -> Bitboard {
    !filefill(pawns) ^ open_files(pawns, opponent_pawns)
}

#[cfg(test)]
mod tests {
    use color::*;
    use piece::*;
    use common::*;
    use super::*;
    use fen::FEN;
    use game::Game;
    use moves::Move;

    #[test]
    fn test_draw() {
        let mut game = Game::new();

        game.load_fen("8/8/4k3/8/8/4K3/8/8 w - - 0 1");
        assert_eq!(game.eval(), 0);

        game.load_fen("8/8/4k3/8/4B3/4K3/8/8 w - - 0 1");
        assert_eq!(game.eval(), 0);

        game.load_fen("8/8/4k3/8/4N3/4K3/8/8 w - - 0 1");
        assert_eq!(game.eval(), 0);
    }

    #[test]
    fn test_see() {
        let mut game = Game::new();

        let fen = "1k1r4/1pp4p/p7/4p3/8/P5P1/1PP4P/2K1R3 w - -";
        game.load_fen(fen);
        assert_eq!(game.see(Move::new(E1, E5, CAPTURE)), PAWN_VALUE);

        let fen = "1k1r3q/1ppn3p/p4b2/4p3/8/P2N2P1/1PP1R1BP/2K1Q3 w - -";
        game.load_fen(fen);
        assert_eq!(game.see(Move::new(D3, E5, CAPTURE)), PAWN_VALUE - KNIGHT_VALUE);

        let fen = "rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 2";
        game.load_fen(fen);
        assert_eq!(game.see(Move::new(E4, D5, CAPTURE)), 0);

        let fen = "rnbqkb1r/ppp1pppp/5n2/3p4/4P3/2N5/PPPP1PPP/R1BQKBNR w KQkq - 2 3";
        game.load_fen(fen);
        assert_eq!(game.see(Move::new(E4, D5, CAPTURE)), 0);

        let fen = "rnbqkb1r/pp2pppp/2p2n2/1B1p4/4P3/2N5/PPPP1PPP/R1BQK1NR w KQkq - 0 4";
        game.load_fen(fen);
        assert_eq!(game.see(Move::new(E4, D5, CAPTURE)), 0);
        assert_eq!(game.see(Move::new(C3, D5, CAPTURE)), PAWN_VALUE - KNIGHT_VALUE);
        assert_eq!(game.see(Move::new(B5, C6, CAPTURE)), PAWN_VALUE - BISHOP_VALUE);

        let fen = "rnbqkbnr/pppp1ppp/8/4p3/3P4/8/PPP1PPPP/RNBQKBNR w KQkq e6 0 2";
        game.load_fen(fen);
        assert_eq!(game.see(Move::new(D4, E5, CAPTURE)), PAWN_VALUE);

        let fen = "1K1k4/8/5n2/3p4/8/1BN2B2/6b1/7b w - -";
        game.load_fen(fen);
        assert_eq!(game.see(Move::new(B3, D5, CAPTURE)), PAWN_VALUE);

        let fen = "3r2k1/pppb2pp/5q2/5p2/3R1P2/2B5/PPP3PP/5RK1 w - - 0 1";
        game.load_fen(fen);
        assert_eq!(game.see(Move::new(D4, D7, CAPTURE)), BISHOP_VALUE - ROOK_VALUE);

        let fen = "k1K5/8/4N3/1p6/2rp1n2/1P2P3/3Q4/8 w - - 0 1";
        game.load_fen(fen);
        assert_eq!(game.see(Move::new(E3, F4, CAPTURE)), KNIGHT_VALUE);
        assert_eq!(game.see(Move::new(E6, F4, CAPTURE)), KNIGHT_VALUE);
        assert_eq!(game.see(Move::new(B3, C4, CAPTURE)), ROOK_VALUE - PAWN_VALUE);
        assert_eq!(game.see(Move::new(D2, D4, CAPTURE)), PAWN_VALUE + ROOK_VALUE - QUEEN_VALUE);

        let fen = "7k/p7/1p6/8/8/1Q6/8/7K w - - 0 1";
        game.load_fen(fen);
        assert_eq!(game.see(Move::new(B3, B6, CAPTURE)), PAWN_VALUE - QUEEN_VALUE);

        let fen = "7k/2p5/1p6/8/8/1Q6/8/7K w - - 0 1";
        game.load_fen(fen);
        assert_eq!(game.see(Move::new(B3, B6, CAPTURE)), PAWN_VALUE - QUEEN_VALUE);

        let fen = "7k/3n4/1p6/8/8/1Q6/8/7K w - - 0 1";
        game.load_fen(fen);
        assert_eq!(game.see(Move::new(B3, B6, CAPTURE)), PAWN_VALUE - QUEEN_VALUE);
    }

    #[test]
    fn test_open_files() {
        let game = Game::from_fen("8/8/3k4/3p4/8/2PP4/3R1R2/3K4 w - - 0 1");

        let black_pawns = game.bitboards[(BLACK | PAWN) as usize];
        let white_pawns = game.bitboards[(WHITE | PAWN) as usize];
        let white_rooks = game.bitboards[(WHITE | ROOK) as usize];

        let open_files = open_files(white_pawns, black_pawns);

        assert_eq!(white_rooks.count(), 2);
        assert_eq!((white_rooks & open_files).count(), 1);
    }

    #[test]
    fn test_closed_files() {
        let game = Game::from_fen("8/8/3k4/3p4/8/2PP4/3R1R2/3K4 w - - 0 1");

        let black_pawns = game.bitboards[(BLACK | PAWN) as usize];
        let white_pawns = game.bitboards[(WHITE | PAWN) as usize];

        let closed_files = closed_files(white_pawns, black_pawns);

        assert_eq!(black_pawns.count(), 1);
        assert_eq!(white_pawns.count(), 2);
        assert_eq!((black_pawns & closed_files).count(), 1);
        assert_eq!((white_pawns & closed_files).count(), 1);
    }

    #[test]
    fn test_half_open_files() {
        let game = Game::from_fen("8/8/3k4/3p4/8/2PP4/3R1R2/3K4 w - - 0 1");

        let black_pawns = game.bitboards[(BLACK | PAWN) as usize];
        let white_pawns = game.bitboards[(WHITE | PAWN) as usize];

        // NOTE: Param order is important here
        let black_half_open_files = half_open_files(black_pawns, white_pawns);
        let white_half_open_files = half_open_files(white_pawns, black_pawns);

        assert_eq!(black_pawns.count(), 1);
        assert_eq!(white_pawns.count(), 2);
        assert_eq!((black_pawns & white_half_open_files).count(), 0);
        assert_eq!((white_pawns & black_half_open_files).count(), 1);
    }
}
