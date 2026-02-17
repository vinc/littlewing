use std::prelude::v1::*;
use std::cmp;

use crate::color::*;
use crate::piece::*;
use crate::square::*;
use crate::common::*;
use crate::attack::Attack;
use crate::attack::piece_attacks;
use crate::bitboard::{Bitboard, BitboardExt, BitboardIterator};
use crate::bitboard::filefill;
use crate::game::Game;
use crate::piece_move::PieceMove;
use crate::piece_square_table::PST;

//pub const HALF_OPEN_FILE:  Score =     5;
//pub const KNIGHT_PAWNS:    Score =     5;
//pub const ROOK_OPEN_FILE:  Score =    20;
//pub const ROOK_PAWNS:      Score =     5;
//pub const DOUBLED_PAWN:    Score =   -10;

pub const KING_VALUE:      Score = 10000;
pub const PAWN_VALUE:      Score =   100;

pub const KNIGHT_VALUE:    Score =   304;
pub const BISHOP_VALUE:    Score =   292;
pub const ROOK_VALUE:      Score =   487;
pub const QUEEN_VALUE:     Score =  1013;
pub const BISHOP_PAIR:     Score =    33;

pub const KNIGHT_MOBILITY: Score =    24;
pub const BISHOP_MOBILITY: Score =    47;
pub const ROOK_MOBILITY:   Score =    39;
pub const QUEEN_MOBILITY:  Score =    21;

pub const TEMPO:           Score =    11;

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

    pub static ref MOBILITY: [Score; 14] = {
        let mut mobility = [0; 14];

        mobility[KNIGHT as usize] = KNIGHT_MOBILITY;
        mobility[BISHOP as usize] = BISHOP_MOBILITY;
        mobility[ROOK   as usize] = ROOK_MOBILITY;
        mobility[QUEEN  as usize] = QUEEN_MOBILITY;

        mobility
    };
}

/// Evaluation algorithms
pub trait Eval {
    /// Evaluate the current position
    fn eval(&self) -> Score;

    /// Evaluate material at the current position for the given side
    fn eval_material(&self, c: Color) -> Score;

    /// Static Exchange Evaluation
    fn see(&self, capture: PieceMove) -> Score;
}

trait EvalExt {
    fn eval_ending(&self, c: Color) -> Option<Score>;
    fn lvp(&self, side: Color, attacks: Bitboard, occupied: Bitboard) -> Square;
}

impl Eval for Game {
    fn eval(&self) -> Score {
        let occupied = self.bitboard(WHITE) | self.bitboard(BLACK);
        let side = self.side();

        // Look for win/loss/draw
        if let Some(score) = self.eval_ending(side) {
            return score;
        }

        let mut material = [0; 2];
        let mut mobility = [0; 2];
        let mut position = [[0; 2]; 2]; // Opening and ending phases

        for &c in &COLORS {
            for &p in &PIECES {
                let piece = c | p;
                let mut pieces = self.bitboards[piece as usize];
                let n = pieces.count() as Score;
                material[c as usize] += n * PIECE_VALUES[piece as usize];
                if p == BISHOP && n > 1 { // FIXME: Slows eval from 1250ns to 1350ns
                    material[c as usize] += BISHOP_PAIR;
                }
                while let Some(square) = pieces.next() {
                    let targets = piece_attacks(piece, square, occupied);
                    mobility[c as usize] += MOBILITY[p as usize] * targets.count() as Score;
                    position[c as usize][0] += PST[piece as usize][square as usize][0];
                    position[c as usize][1] += PST[piece as usize][square as usize][1];
                }
            }
        }

        let mut position_score = 0;
        let mut material_score = 0;
        let mut mobility_score = 0;
        let c = side as usize;

        // Linear interpolation between opening and ending scores
        // based on the number of pieces on the board
        let x0 = 32; // Max
        let x1 = 2; // Min
        let x = occupied.count() as Score; // Current

        let y0 = position[c][0];
        let y1 = position[c][1];
        position_score += (y0 * (x1 - x) + y1 * (x - x0)) / (x1 - x0);
        material_score += material[c];
        mobility_score += mobility[c] / 10;

        let y0 = position[c ^ 1][0];
        let y1 = position[c ^ 1][1];
        position_score -= (y0 * (x1 - x) + y1 * (x - x0)) / (x1 - x0);
        material_score -= material[c ^ 1];
        mobility_score -= mobility[c ^ 1] / 10;

        let score = position_score + material_score + mobility_score + TEMPO;

        if self.is_eval_verbose {
            println!("material: {:>5.2}", 0.01 * material_score as f64);
            println!("position: {:>5.2}", 0.01 * position_score as f64);
            println!("mobility: {:>5.2}", 0.01 * mobility_score as f64);
            println!("total:    {:>5.2}", 0.01 * score as f64);
        }

        score
    }

    fn eval_material(&self, c: Color) -> Score {
        let mut score = 0;

        /*
        let mut pawns_count = 0;

        let color_pawns = self.bitboards[(c | PAWN) as usize];
        let other_pawns = self.bitboards[(c ^ 1 | PAWN) as usize];

        let open_files = open_files(color_pawns, other_pawns);

        let half_open_files = half_open_files(color_pawns, other_pawns);
        let half_open_files_count = (half_open_files & RANK_1).count() as Score;
        score += half_open_files_count * HALF_OPEN_FILE;
        */

        for &p in &PIECES {
            let piece = c | p;
            let pieces = self.bitboards[piece as usize];
            let n = pieces.count() as Score;
            score += n * PIECE_VALUES[piece as usize];

            /*
            match p { // FIXME: Slows eval from 65 to 130ns
                PAWN => {
                    pawns_count = n;

                    let pawns_files_count = (filefill(pieces) & RANK_1).count() as Score;
                    score += (pawns_count - pawns_files_count) * DOUBLED_PAWN;
                },
                KNIGHT => {
                    score += n * pawns_count * KNIGHT_PAWNS;
                },
                BISHOP if n == 2 => {
                    score += BISHOP_PAIR;
                },
                ROOK => {
                    let rooks_on_open_files = (pieces & open_files).count();
                    let rooks_on_half_open_files = (pieces & half_open_files).count();
                    score += (rooks_on_open_files as Score) * ROOK_OPEN_FILE;
                    score += (rooks_on_half_open_files as Score) * ROOK_OPEN_FILE / 2;
                    score += n * (8 - pawns_count) * ROOK_PAWNS;
                },
                _ => { }
            }
            */
        }

        score
    }

    fn see(&self, capture: PieceMove) -> Score {
        let mut occupied = self.bitboard(WHITE) | self.bitboard(BLACK);
        let mut sq = capture.from();
        let mut side = self.side();
        let mut gains = [0; 32];
        let mut d = 0;

        let mut piece = self.board[capture.to() as usize];
        let mut value = PIECE_VALUES[self.board[sq as usize] as usize];
        gains[d] = PIECE_VALUES[piece as usize];

        if capture.is_promotion() {
            value = PIECE_VALUES[capture.promotion_kind() as usize];
            gains[d] += value - PIECE_VALUES[PAWN as usize];
        }
        while sq != OUT {
            d += 1;
            side ^= 1;
            occupied.reset(sq); // Remove piece

            gains[d] = value - gains[d - 1];

            // Get square of least valuable piece remaining
            let attacks = self.attacks_to(capture.to(), occupied);
            sq = self.lvp(side, attacks, occupied);

            if sq != OUT {
                piece = self.board[sq as usize];
                value = PIECE_VALUES[piece as usize];

                if piece.is_pawn() && capture.to().flip(side).rank() == RANK_8 as u8 {
                    value = PIECE_VALUES[QUEEN as usize];
                }
            }
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
                return subset.scan() as Square;
            }
        }

        OUT
    }
}

#[allow(dead_code)]
fn closed_files(white_pawns: Bitboard, black_pawns: Bitboard) -> Bitboard {
    filefill(white_pawns) & filefill(black_pawns)
}

#[allow(dead_code)]
fn open_files(white_pawns: Bitboard, black_pawns: Bitboard) -> Bitboard {
    !filefill(white_pawns) & !filefill(black_pawns)
}

#[allow(dead_code)]
fn half_open_files(pawns: Bitboard, opponent_pawns: Bitboard) -> Bitboard {
    !filefill(pawns) ^ open_files(pawns, opponent_pawns)
}

#[cfg(test)]
mod tests {
    use crate::color::*;
    use crate::piece::*;
    use super::*;
    use crate::fen::FEN;
    use crate::game::Game;
    use crate::piece_move_notation::PieceMoveNotation;

    #[test]
    fn test_draw() {
        let mut game = Game::new();

        game.load_fen("8/8/4k3/8/8/4K3/8/8 w - - 0 1").unwrap();
        assert_eq!(game.eval(), 0);

        game.load_fen("8/8/4k3/8/4B3/4K3/8/8 w - - 0 1").unwrap();
        assert_eq!(game.eval(), 0);

        game.load_fen("8/8/4k3/8/4N3/4K3/8/8 w - - 0 1").unwrap();
        assert_eq!(game.eval(), 0);
    }

    #[test]
    fn test_see() {
        let p = PAWN_VALUE;
        let n = KNIGHT_VALUE;
        let b = BISHOP_VALUE;
        let r = ROOK_VALUE;
        let q = QUEEN_VALUE;

        let list = [
            ("1k1r4/1pp4p/p7/4p3/8/P5P1/1PP4P/2K1R3 w - -", "Rxe5", p),
            ("1k1r3q/1ppn3p/p4b2/4p3/8/P2N2P1/1PP1R1BP/2K1Q3 w - -", "Nxe5", p - n),
            ("rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 2", "exd5", 0),
            ("rnbqkb1r/ppp1pppp/5n2/3p4/4P3/2N5/PPPP1PPP/R1BQKBNR w KQkq - 2 3", "exd5", 0),
            ("rnbqkb1r/pp2pppp/2p2n2/1B1p4/4P3/2N5/PPPP1PPP/R1BQK1NR w KQkq - 0 4", "exd5", 0),
            ("rnbqkb1r/pp2pppp/2p2n2/1B1p4/4P3/2N5/PPPP1PPP/R1BQK1NR w KQkq - 0 4", "Nxd5", p - n),
            ("rnbqkb1r/pp2pppp/2p2n2/1B1p4/4P3/2N5/PPPP1PPP/R1BQK1NR w KQkq - 0 4", "Bxc6", p - b),
            ("rnbqkbnr/pppp1ppp/8/4p3/3P4/8/PPP1PPPP/RNBQKBNR w KQkq e6 0 2", "dxe5", p),
            ("1K1k4/8/5n2/3p4/8/1BN2B2/6b1/7b w - -", "Bxd5", p),
            ("3r2k1/pppb2pp/5q2/5p2/3R1P2/2B5/PPP3PP/5RK1 w - - 0 1", "Rxd7", b - r),
            ("k1K5/8/4N3/1p6/2rp1n2/1P2P3/3Q4/8 w - - 0 1", "bxc4", r - p),
            ("7k/p7/1p6/8/8/1Q6/8/7K w - - 0 1", "Qxb6", p - q),
            ("7k/2p5/1p6/8/8/1Q6/8/7K w - - 0 1", "Qxb6", p - q),
            ("7k/3n4/1p6/8/8/1Q6/8/7K w - - 0 1", "Qxb6", p - q),

            // From: https://github.com/jdart1/arasan-chess/blob/v25.3/src/unit.cpp#L170
            ("4R3/2r3p1/5bk1/1p1r3p/p2PR1P1/P1BK1P2/1P6/8 b - -", "hxg4", 0),
            ("4R3/2r3p1/5bk1/1p1r1p1p/p2PR1P1/P1BK1P2/1P6/8 b - -", "hxg4", 0),
            //("4r1k1/5pp1/nbp4p/1p2p2q/1P2P1b1/1BP2N1P/1B2QPPK/3R4 b - -", "Bxf3", 0),
            ("2r1r1k1/pp1bppbp/3p1np1/q3P3/2P2P2/1P2B3/P1N1B1PP/2RQ1RK1 b - -", "dxe5", p),
            ("7r/5qpk/p1Qp1b1p/3r3n/BB3p2/5p2/P1P2P2/4RK1R w - -", "Re8", 0),
            ("6rr/6pk/p1Qp1b1p/2n5/1B3p2/5p2/P1P2P2/4RK1R w - -", "Re8", -r),
            ("7r/5qpk/2Qp1b1p/1N1r3n/BB3p2/5p2/P1P2P2/4RK1R w - -", "Re8", -r),
            ("6RR/4bP2/8/8/5r2/3K4/5p2/4k3 w - -", "f8=Q", b - p),
            //("6RR/4bP2/8/8/5r2/3K4/5p2/4k3 w - -", "f8=N", n - p),
            //("7R/5P2/8/8/8/3K2r1/5p2/4k3 w - -", "f8=Q", q - p),
            //("7R/5P2/8/8/8/3K2r1/5p2/4k3 w - -", "f8=B", b - p),
            ("7R/4bP2/8/8/1q6/3K4/5p2/4k3 w - -", "f8=R", -p),
            ("8/4kp2/2npp3/1Nn5/1p2PQP1/7q/1PP1B3/4KR1r b - -", "Rxf1+", 0),
            ("8/4kp2/2npp3/1Nn5/1p2P1P1/7q/1PP1B3/4KR1r b - -", "Rxf1+",  0),
            ("2r2r1k/6bp/p7/2q2p1Q/3PpP2/1B6/P5PP/2RR3K b - -", "Qxc1", 2 * r - q),
            //("r2qk1nr/pp2ppbp/2b3p1/2p1p3/8/2N2N2/PPPP1PPP/R1BQR1K1 w kq -", "Nxe5", p),
            ("6r1/4kq2/b2p1p2/p1pPb3/p1P2B1Q/2P4P/2B1R1P1/6K1 w - -", "Bxe5", 0),
            //("3q2nk/pb1r1p2/np6/3P2Pp/2p1P3/2R4B/PQ3P1P/3R2K1 w - h6", "gxh6", 0),
            //("3q2nk/pb1r1p2/np6/3P2Pp/2p1P3/2R1B2B/PQ3P1P/3R2K1 w - h6", "gxh6", p),
            ("2r4r/1P4pk/p2p1b1p/7n/BB3p2/2R2p2/P1P2P2/4RK2 w - -", "Rxc8", r),
            //("2r5/1P4pk/p2p1b1p/5b1n/BB3p2/2R2p2/P1P2P2/4RK2 w - -", "Rxc8", r),
            ("2r4k/2r4p/p7/2b2p1b/4pP2/1BR5/P1R3PP/2Q4K w - -", "Rxc5", b),
            ("8/pp6/2pkp3/4bp2/2R3b1/2P5/PP4B1/1K6 w - -", "Bxc6", p - b),
            ("4q3/1p1pr1k1/1B2rp2/6p1/p3PP2/P3R1P1/1P2R1K1/4Q3 b - -", "Rxe4", p - r),
            ("4q3/1p1pr1kb/1B2rp2/6p1/p3PP2/P3R1P1/1P2R1K1/4Q3 b - -", "Bxe4", p),

            // From: https://github.com/lithander/Leorik/blob/3.2/Leorik.Test/see.epd
            ("6k1/1pp4p/p1pb4/6q1/3P1pRr/2P4P/PP1Br1P1/5RKN w - -", "Rfxf4", p - r + b),
            //("5rk1/1pp2q1p/p1pb4/8/3P1NP1/2P5/1P1BQ1P1/5RK1 b - -", "Bxf4", -n + b),
            ("4R3/2r3p1/5bk1/1p1r3p/p2PR1P1/P1BK1P2/1P6/8 b - -", "hxg4", 0),
            ("4R3/2r3p1/5bk1/1p1r1p1p/p2PR1P1/P1BK1P2/1P6/8 b - -", "hxg4", 0),
            //("4r1k1/5pp1/nbp4p/1p2p2q/1P2P1b1/1BP2N1P/1B2QPPK/3R4 b - -", "Bxf3", 0),
            ("2r1r1k1/pp1bppbp/3p1np1/q3P3/2P2P2/1P2B3/P1N1B1PP/2RQ1RK1 b - -", "dxe5", p),
            ("7r/5qpk/p1Qp1b1p/3r3n/BB3p2/5p2/P1P2P2/4RK1R w - -", "Re8", 0),
            ("6rr/6pk/p1Qp1b1p/2n5/1B3p2/5p2/P1P2P2/4RK1R w - -", "Re8",  -r),
            ("7r/5qpk/2Qp1b1p/1N1r3n/BB3p2/5p2/P1P2P2/4RK1R w - -", "Re8", -r),
            ("6RR/4bP2/8/8/5r2/3K4/5p2/4k3 w - -", "f8=Q", b - p),
            //("6RR/4bP2/8/8/5r2/3K4/5p2/4k3 w - -", "f8=N", n - p),
            ("7R/5P2/8/8/6r1/3K4/5p2/4k3 w - -", "f8=Q", q - p),
            ("7R/5P2/8/8/6r1/3K4/5p2/4k3 w - -", "f8=B", b - p),
            ("7R/4bP2/8/8/1q6/3K4/5p2/4k3 w - -", "f8=R", -p),
            ("8/4kp2/2npp3/1Nn5/1p2PQP1/7q/1PP1B3/4KR1r b - -", "Rxf1+", 0),
            ("8/4kp2/2npp3/1Nn5/1p2P1P1/7q/1PP1B3/4KR1r b - -", "Rxf1+", 0),
            ("2r2r1k/6bp/p7/2q2p1Q/3PpP2/1B6/P5PP/2RR3K b - -", "Qxc1", r - q + r),
            //("r2qk1nr/pp2ppbp/2b3p1/2p1p3/8/2N2N2/PPPP1PPP/R1BQR1K1 w kq -", "Nxe5", p),
            ("6r1/4kq2/b2p1p2/p1pPb3/p1P2B1Q/2P4P/2B1R1P1/6K1 w - -", "Bxe5", 0),
            //("3q2nk/pb1r1p2/np6/3P2Pp/2p1P3/2R4B/PQ3P1P/3R2K1 w - h6", "gxh6", 0),
            //("3q2nk/pb1r1p2/np6/3P2Pp/2p1P3/2R1B2B/PQ3P1P/3R2K1 w - h6", "gxh6", p),
            ("2r4r/1P4pk/p2p1b1p/7n/BB3p2/2R2p2/P1P2P2/4RK2 w - -", "Rxc8", r),
            //("2r5/1P4pk/p2p1b1p/5b1n/BB3p2/2R2p2/P1P2P2/4RK2 w - -", "Rxc8", r),
            ("2r4k/2r4p/p7/2b2p1b/4pP2/1BR5/P1R3PP/2Q4K w - -", "Rxc5", b),
            ("8/pp6/2pkp3/4bp2/2R3b1/2P5/PP4B1/1K6 w - -", "Bxc6", p - b),
            ("4q3/1p1pr1k1/1B2rp2/6p1/p3PP2/P3R1P1/1P2R1K1/4Q3 b - -", "Rxe4", p - r),
            ("4q3/1p1pr1kb/1B2rp2/6p1/p3PP2/P3R1P1/1P2R1K1/4Q3 b - -", "Bxe4", p),
            //("3r3k/3r4/2n1n3/8/3p4/2PR4/1B1Q4/3R3K w - -", "Rxd4", p - r + n - p + n - b + r - q + r),
            ("1k1r4/1ppn3p/p4b2/4n3/8/P2N2P1/1PP1R1BP/2K1Q3 w - -", "Nxe5", n - n + b - r + n),
            ("1k1r3q/1ppn3p/p4b2/4p3/8/P2N2P1/1PP1R1BP/2K1Q3 w - -", "Nxe5", p - n),
            ("rnb2b1r/ppp2kpp/5n2/4P3/q2P3B/5R2/PPP2PPP/RN1QKB2 w Q -", "Bxf6", n - b + p),
            ("r2q1rk1/2p1bppp/p2p1n2/1p2P3/4P1b1/1nP1BN2/PP3PPP/RN1QR1K1 b - -", "Bxf3", n - b),
            ("r1bqkb1r/2pp1ppp/p1n5/1p2p3/3Pn3/1B3N2/PPP2PPP/RNBQ1RK1 b kq -", "Nxd4", p - n + n - p),
            ("r1bq1r2/pp1ppkbp/4N1p1/n3P1B1/8/2N5/PPP2PPP/R2QK2R w KQ -", "Nxg7", b - n),
            ("r1bq1r2/pp1ppkbp/4N1pB/n3P3/8/2N5/PPP2PPP/R2QK2R w KQ -", "Nxg7", b),
            ("rnq1k2r/1b3ppp/p2bpn2/1p1p4/3N4/1BN1P3/PPP2PPP/R1BQR1K1 b kq -", "Bxh2", p - b),
            ("rn2k2r/1bq2ppp/p2bpn2/1p1p4/3N4/1BN1P3/PPP2PPP/R1BQR1K1 b kq -", "Bxh2", p),
            ("r2qkbn1/ppp1pp1p/3p1rp1/3Pn3/4P1b1/2N2N2/PPP2PPP/R1BQKB1R b KQq -", "Bxf3", n - b + p),
            ("rnbq1rk1/pppp1ppp/4pn2/8/1bPP4/P1N5/1PQ1PPPP/R1B1KBNR b KQ -", "Bxc3", n - b),
            ("r4rk1/3nppbp/bq1p1np1/2pP4/8/2N2NPP/PP2PPB1/R1BQR1K1 b - -", "Qxb2", p - q),
            ("r4rk1/1q1nppbp/b2p1np1/2pP4/8/2N2NPP/PP2PPB1/R1BQR1K1 b - -", "Nxd5", p - n),
            ("1r3r2/5p2/4p2p/2k1n1P1/2PN1nP1/1P3P2/8/2KR1B1R b - -", "Rxb3", p - r),
            ("1r3r2/5p2/4p2p/4n1P1/kPPN1nP1/5P2/8/2KR1B1R b - -", "Rxb4", p),
            ("2r2rk1/5pp1/pp5p/q2p4/P3n3/1Q3NP1/1P2PP1P/2RR2K1 b - -", "Rxc1", r - r),
            //("5rk1/5pp1/2r4p/5b2/2R5/6Q1/R1P1qPP1/5NK1 b - -", "Bxc2", p - b + r - q + r),
            ("1r3r1k/p4pp1/2p1p2p/qpQP3P/2P5/3R4/PP3PP1/1K1R4 b - -", "Qxa2", p - q),
            ("1r5k/p4pp1/2p1p2p/qpQP3P/2P2P2/1P1R4/P4rP1/1K1R4 b - -", "Qxa2", p),
            ("r2q1rk1/1b2bppp/p2p1n2/1ppNp3/3nP3/P2P1N1P/BPP2PP1/R1BQR1K1 w - -", "Nxe7", b - n),
            ("rnbqrbn1/pp3ppp/3p4/2p2k2/4p3/3B1K2/PPP2PPP/RNB1Q1NR w - -", "Bxe4", p),
            ("rnb1k2r/p3p1pp/1p3p1b/7n/1N2N3/3P1PB1/PPP1P1PP/R2QKB1R w KQkq -", "Nd6", -n + p),
            ("r1b1k2r/p4npp/1pp2p1b/7n/1N2N3/3P1PB1/PPP1P1PP/R2QKB1R w KQkq -", "Nd6", -n + n),
            ("2r1k2r/pb4pp/5p1b/2KB3n/4N3/2NP1PB1/PPP1P1PP/R2Q3R w k -", "Bc6", -b),
            ("2r1k2r/pb4pp/5p1b/2KB3n/1N2N3/3P1PB1/PPP1P1PP/R2Q3R w k -", "Bc6", -b + b),
            //("2r1k3/pbr3pp/5p1b/2KB3n/1N2N3/3P1PB1/PPP1P1PP/R2Q3R w - -", "Bc6", -b + b - n),
            ("5k2/p2P2pp/8/1pb5/1Nn1P1n1/6Q1/PPP4P/R3K1NR w KQ -", "d8=Q", q - p),
            ("r4k2/p2P2pp/8/1pb5/1Nn1P1n1/6Q1/PPP4P/R3K1NR w KQ -", "d8=Q", (q - p) - q),
            ("5k2/p2P2pp/1b6/1p6/1Nn1P1n1/8/PPP4P/R2QK1NR w KQ -", "d8=Q",(q - p) - q + b),
            ("4kbnr/p1P1pppp/b7/4q3/7n/8/PP1PPPPP/RNBQKBNR w KQk -", "c8=Q", (q - p) - q),
            ("4kbnr/p1P1pppp/b7/4q3/7n/8/PPQPPPPP/RNB1KBNR w KQk -", "c8=Q", (q - p) - q + b),
            //("4kbnr/p1P1pppp/b7/4q3/7n/8/PPQPPPPP/RNB1KBNR w KQk -", "c8=Q", (q - p)),
            //("4kbnr/p1P4p/b1q5/5pP1/4n3/5Q2/PP1PPP1P/RNB1KBNR w KQk f6", "gxf6", p - p),
            //("4kbnr/p1P4p/b1q5/5pP1/4n3/5Q2/PP1PPP1P/RNB1KBNR w KQk f6", "gxf6",	p - p),
            //("4kbnr/p1P4p/b1q5/5pP1/4n2Q/8/PP1PPP1P/RNB1KBNR w KQk f6", "gxf6", p - p),
            ("1n2kb1r/p1P4p/2qb4/5pP1/4n2Q/8/PP1PPP1P/RNB1KBNR w KQk -", "cxb8=Q", n + (q - p) - q),
            ("rnbqk2r/pp3ppp/2p1pn2/3p4/3P4/N1P1BN2/PPB1PPPb/R2Q1RK1 w kq -", "Kxh2", b),
            ("3N4/2K5/2n5/1k6/8/8/8/8 b - -", "Nxd8", n - n),
            //("3N4/2P5/2n5/1k6/8/8/8/4K3 b - -", "Nxd8", n - (n + q - p)),
            //("3n3r/2P5/8/1k6/8/8/3Q4/4K3 w - -", "Qxd8", n),
            ("3n3r/2P5/8/1k6/8/8/3Q4/4K3 w - -", "cxd8=Q", (n + q - p) - q + r),
            ("r2n3r/2P1P3/4N3/1k6/8/8/8/4K3 w - -", "Nxd8", n),
            ("8/8/8/1k6/6b1/4N3/2p3K1/3n4 w - -", "Nxd1", n - n),
            //("8/8/1k6/8/8/2N1N3/2p1p1K1/3n4 w - -", "Nxd1", n - (n + q - p)),
            ("8/8/1k6/8/8/2N1N3/4p1K1/3n4 w - -", "Ncxd1", n - (n + q - p) + q),
            ("r1bqk1nr/pppp1ppp/2n5/1B2p3/1b2P3/5N2/PPPP1PPP/RNBQK2R w KQkq -", "O-O", 0),
        ];

        let mut game = Game::new();
        for (fen, m, score) in list {
            println!("{}", fen);
            game.load_fen(fen).unwrap();
            let m = game.parse_move(m).unwrap();
            assert_eq!(game.see(m), score);
        }
    }

    #[test]
    fn test_open_files() {
        let game = Game::from_fen("8/8/3k4/3p4/8/2PP4/3R1R2/3K4 w - - 0 1").unwrap();

        let black_pawns = game.bitboards[(BLACK | PAWN) as usize];
        let white_pawns = game.bitboards[(WHITE | PAWN) as usize];
        let white_rooks = game.bitboards[(WHITE | ROOK) as usize];

        let open_files = open_files(white_pawns, black_pawns);

        assert_eq!(white_rooks.count(), 2);
        assert_eq!((white_rooks & open_files).count(), 1);
    }

    #[test]
    fn test_closed_files() {
        let game = Game::from_fen("8/8/3k4/3p4/8/2PP4/3R1R2/3K4 w - - 0 1").unwrap();

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
        let game = Game::from_fen("8/8/3k4/3p4/8/2PP4/3R1R2/3K4 w - - 0 1").unwrap();

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
