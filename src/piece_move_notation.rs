use attack::piece_attacks;
use bitboard::BitboardExt;
use color::*;
use common::*;
use game::Game;
use piece::*;
use piece::{PieceAttr, PieceChar};
use piece_move::*;
use square::*;
use square::SquareExt;

/// PieceMoveList generator
pub trait PieceMoveNotation {
    /// Get move from the given SAN string
    fn move_from_can(&mut self, s: &str) -> PieceMove;

    /// Get SAN string from the given move
    fn move_to_san(&mut self, m: PieceMove) -> String;
}

impl PieceMoveNotation for Game {
    fn move_from_can(&mut self, s: &str) -> PieceMove {
        debug_assert!(s.len() == 4 || s.len() == 5);

        let side = self.positions.top().side;
        let (a, b) = s.split_at(2);
        let from = Square::from_coord(String::from(a));
        let to = Square::from_coord(String::from(b));
        let piece = self.board[from as usize];
        let capture = self.board[to as usize];

        let mt = if s.len() == 5 {
            let promotion = match s.chars().nth(4) {
                Some('n') => KNIGHT_PROMOTION,
                Some('b') => BISHOP_PROMOTION,
                Some('r') => ROOK_PROMOTION,
                Some('q') => QUEEN_PROMOTION,
                _         => panic!("could not parse promotion")
            };
            if capture == EMPTY {
                promotion
            } else {
                promotion | CAPTURE
            }
        } else if piece.kind() == KING && from == E1.flip(side) && to == G1.flip(side) {
            KING_CASTLE
        } else if piece.kind() == KING && from == E1.flip(side) && to == C1.flip(side) {
            QUEEN_CASTLE
        } else if capture == EMPTY {
            let d = (to.flip(side) as Direction) - (from.flip(side) as Direction);
            if piece.kind() == PAWN && (d == 2 * UP) {
                DOUBLE_PAWN_PUSH
            } else if piece.kind() == PAWN && to == self.positions.top().en_passant {
                EN_PASSANT
            } else {
                QUIET_MOVE
            }
        } else {
            CAPTURE
        };

        PieceMove::new(from, to, mt)
    }

    // NOTE: this function assumes that the move has not been played yet
    fn move_to_san(&mut self, m: PieceMove) -> String {
        let piece = self.board[m.from() as usize];

        let mut out = String::new();

        if m.is_castle() {
            if m.castle_kind() == KING {
                out.push_str("O-O");
            } else {
                out.push_str("O-O-O");
            }
            return out;
        }

        if piece.kind() != PAWN {
            out.push(piece.kind().to_char());
        }

        // Piece disambiguation or pawn capture
        if piece.kind() != PAWN || m.is_capture() {
            let occupied = self.bitboard(WHITE) | self.bitboard(BLACK);
            let pieces = self.bitboard(piece);
            let attacks = piece_attacks(piece, m.to(), occupied);
            let attackers = pieces & attacks;
            if attackers.count() > 1 || piece.kind() == PAWN {
                out.push(m.from().file_to_char());
            }
            // TODO: Pawn disambiguation
        }

        if m.is_capture() {
            out.push('x');
        }

        out.push_str(m.to().to_coord().as_str());

        if m.is_promotion() {
            out.push('=');
            out.push(m.promotion_kind().to_char());
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use common::*;
    use piece_move::PieceMove;
    use fen::FEN;
    use game::Game;
    use super::*;

    #[test]
    fn test_move_from_can() {
        let mut game = Game::from_fen(DEFAULT_FEN);

        let m = game.move_from_can("e2e4");
        assert_eq!(m, PieceMove::new(E2, E4, DOUBLE_PAWN_PUSH));

        let m = game.move_from_can("g1f3");
        assert_eq!(m, PieceMove::new(G1, F3, QUIET_MOVE));
    }

    #[test]
    fn test_move_to_san() {
        let fen = "7k/3P1ppp/4PQ2/8/8/8/8/6RK w - - 0 1";
        let mut game = Game::from_fen(fen);

        // NOTE: this move should really end with `#`, but this is done
        // in `search::get_pv()`.
        assert_eq!(game.move_to_san(PieceMove::new(F6, G7, CAPTURE)), "Qxg7");
    }
}
