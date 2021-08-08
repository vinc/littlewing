use color::*;
use piece::*;
use square::*;
use common::*;
use bitboard::{Bitboard, BitboardExt};
use game::Game;
use hyperbola::bishop_attacks;
use hyperbola::rook_attacks;

pub trait Attack {
    fn is_check(&self, side: Color) -> bool;
    fn is_attacked(&self, square: Square, side: Color) -> bool;
    fn attacks_to(&self, square: Square, occupied: Bitboard) -> Bitboard;
}

impl Attack for Game {
    fn is_check(&self, side: Color) -> bool {
        let king = self.bitboards[(side | KING) as usize];
        king == 0 || self.is_attacked(king.scan() as Square, side)
    }

    fn is_attacked(&self, square: Square, side: Color) -> bool {
        let bbs = &self.bitboards;

        let occupied = bbs[WHITE as usize] | bbs[BLACK as usize];

        let pawns = bbs[(side ^ 1 | PAWN) as usize];
        let attacks = PAWN_ATTACKS[side as usize][square as usize];
        if attacks & pawns > 0 {
            return true;
        }

        let knights = bbs[(side ^ 1 | KNIGHT) as usize];
        let attacks = PIECE_MASKS[KNIGHT as usize][square as usize];
        if attacks & knights > 0 {
            return true;
        }

        let king = bbs[(side ^ 1 | KING) as usize];
        let attacks = PIECE_MASKS[KING as usize][square as usize];
        if attacks & king > 0 {
            return true;
        }

        let queens = bbs[(side ^ 1 | QUEEN) as usize];

        let bishops = bbs[(side ^ 1 | BISHOP) as usize];
        let attacks = bishop_attacks(square, occupied);
        if attacks & (bishops | queens) > 0 {
            return true;
        }

        let rooks = bbs[(side ^ 1 | ROOK) as usize];
        let attacks = rook_attacks(square, occupied);
        if attacks & (rooks | queens) > 0 {
            return true;
        }

        false
    }

    fn attacks_to(&self, square: Square, occupied: Bitboard) -> Bitboard {
        let bbs = &self.bitboards;

        // Read the array in sequential order from bbs[0] to bbs[13]
        let wpawns  = bbs[WHITE_PAWN   as usize];
        let bpawns  = bbs[BLACK_PAWN   as usize];
        let knights = bbs[WHITE_KNIGHT as usize] | bbs[BLACK_KNIGHT as usize];
        let kings   = bbs[WHITE_KING   as usize] | bbs[BLACK_KING   as usize];
        let bishops = bbs[WHITE_BISHOP as usize] | bbs[BLACK_BISHOP as usize];
        let rooks   = bbs[WHITE_ROOK   as usize] | bbs[BLACK_ROOK   as usize];
        let queens  = bbs[WHITE_QUEEN  as usize] | bbs[BLACK_QUEEN  as usize];

        (wpawns             & piece_attacks(BLACK_PAWN, square, occupied)) |
        (bpawns             & piece_attacks(WHITE_PAWN, square, occupied)) |
        (knights            & piece_attacks(KNIGHT,     square, occupied)) |
        (kings              & piece_attacks(KING,       square, occupied)) |
        ((queens | bishops) & piece_attacks(BISHOP,     square, occupied)) |
        ((queens | rooks)   & piece_attacks(ROOK,       square, occupied))
    }
}

/// Return the attacks bitboard of a piece attacks to a square
pub fn piece_attacks(piece: Piece, square: Square, occupied: Bitboard) -> Bitboard {
    match piece.kind() {
        PAWN   => PAWN_ATTACKS[piece.color() as usize][square as usize],
        KNIGHT => PIECE_MASKS[KNIGHT as usize][square as usize],
        KING   => PIECE_MASKS[KING as usize][square as usize],
        BISHOP => bishop_attacks(square, occupied),
        ROOK   => rook_attacks(square, occupied),
        QUEEN  => bishop_attacks(square, occupied) | rook_attacks(square, occupied),
        _      => unreachable!()
    }
}

lazy_static! {
    pub static ref PAWN_ATTACKS: [[Bitboard; 64]; 2] = {
        let xdirs = [LEFT, RIGHT];
        let ydirs = [DOWN, UP];
        let files = [FILE_H, FILE_A];
        let mut attacks = [[0; 64]; 2];
        for side in 0..2 {
            for square in 0..64 {
                for i in 0..2 {
                    let dir = ydirs[side ^ 1] + xdirs[i];
                    attacks[side][square] |= (1 << square).shift(dir) & !files[i];
                }
            }
        }
        attacks
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use fen::FEN;

    #[test]
    fn test_piece_attacks() {
        let fen = "r1bqk2r/1pppbppp/p1n2n2/4p3/B3P3/5N2/PPPP1PPP/RNBQR1K1 b kq - 5 6";
        let game = Game::from_fen(fen).unwrap();

        let occupied = game.bitboard(WHITE) | game.bitboard(BLACK);

        assert_eq!(game.board[A4 as usize], WHITE | BISHOP);
        assert_eq!(game.board[C2 as usize], WHITE | PAWN);
        assert_eq!(game.board[C6 as usize], BLACK | KNIGHT);

        // Return the attacks set of bishop attacks to C6
        let attacks = piece_attacks(WHITE | BISHOP, C6, occupied);
        assert_eq!(attacks.count(), 6);
        assert_eq!(attacks & game.bitboard(WHITE | BISHOP), 1 << A4);

        // Return the attacks set of bishop attacks from A4
        let moves = piece_attacks(WHITE | BISHOP, A4, occupied);
        assert_eq!(moves.count(), 4);

        let quiet_moves = moves & !occupied;
        assert_eq!(quiet_moves.count(), 2);

        let captures = moves & game.bitboard(BLACK);
        assert_eq!(captures.count(), 1);
        assert_eq!(captures.scan() as Square, C6);

        let defended = moves & game.bitboard(WHITE);
        assert_eq!(defended.count(), 1);
        assert_eq!(defended.scan() as Square, C2);
    }

    #[test]
    fn test_bishop_attacks() {
        let fen = "r1bqk1nr/ppppbppp/2n5/1B2p3/4P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4";
        let game = Game::from_fen(fen).unwrap();
        let occupied = game.bitboard(WHITE) | game.bitboard(BLACK);

        bishop_attacks(B5, occupied).debug();
        bishop_attacks(C8, occupied).debug();
        bishop_attacks(E7, occupied).debug();
        assert_eq!(bishop_attacks(B5, occupied), 0x0000050005081020);
        assert_eq!(bishop_attacks(C8, occupied), 0x000A000000000000);
        assert_eq!(bishop_attacks(E7, occupied), 0x2800284482010000);
    }

    #[test]
    fn test_rook_attacks() {
        let fen = "r3k3/8/8/8/3R4/8/8/R3K3 w - - 0 1";
        let game = Game::from_fen(fen).unwrap();
        let occupied = game.bitboard(WHITE) | game.bitboard(BLACK);

        rook_attacks(A1, occupied).debug();
        rook_attacks(A8, occupied).debug();
        rook_attacks(D4, occupied).debug();
        assert_eq!(rook_attacks(A1, occupied), 0x010101010101011E);
        assert_eq!(rook_attacks(A8, occupied), 0x1E01010101010101);
        assert_eq!(rook_attacks(D4, occupied), 0x08080808F7080808);
    }
}
