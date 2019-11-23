use regex::Regex;

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
use search::Search;

static RE_SAN: &str =
    r"^(?P<piece>[NBRQK])?(?P<file>[a-h])?(?P<rank>[1-9])?(?P<capture>x)?(?P<to>[a-h][1-9])=?(?P<promotion>[KBRQ])?|(?P<queen>O-O-O)|(?P<king>O-O)";

/// PieceMoveList generator
pub trait PieceMoveNotation {
    /// Get move from the given CAN string (fast)
    fn move_from_can(&mut self, s: &str) -> PieceMove;

    /// Get move from the given SAN string (slow)
    fn move_from_san(&mut self, s: &str) -> Option<PieceMove>;

    /// Get SAN string from the given move
    fn move_to_san(&mut self, m: PieceMove) -> String;
}

impl PieceMoveNotation for Game {
    fn move_from_can(&mut self, s: &str) -> PieceMove {
        debug_assert!(s.len() == 4 || s.len() == 5);

        let side = self.side();
        let from = Square::from_coord(&s[0..2]);
        let to = Square::from_coord(&s[2..4]);
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
            let d = (to.flip(side) as Shift) - (from.flip(side) as Shift);
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

    fn move_from_san(&mut self, s: &str) -> Option<PieceMove> {
        lazy_static! {
            static ref RE: Regex = Regex::new(RE_SAN).unwrap();
        }
        let caps = RE.captures(s).unwrap();

        let side = self.side();
        if caps.name("queen").is_some() {
            return Some(PieceMove::new(E1.flip(side), C1.flip(side), QUEEN_CASTLE));
        }
        if caps.name("king").is_some() {
            return Some(PieceMove::new(E1.flip(side), G1.flip(side), KING_CASTLE));
        }

        if caps.name("to").is_none() {
            return None;
        }
        let to = Square::from_coord(&caps["to"]);
        for m in self.get_moves() {
            if m.to() != to {
                continue;
            }

            let p = self.board[m.from() as usize];
            if let Some(piece) = caps.name("piece") {
                if p.kind().to_char().to_string() != piece.as_str() {
                    continue;
                }
            } else if p.kind() != PAWN {
                continue;
            }

            if let Some(file) = caps.name("file") {
                if m.from().file_to_char().to_string() != file.as_str() {
                    continue;
                }
            }

            if let Some(rank) = caps.name("rank") {
                if m.from().rank_to_char().to_string() != rank.as_str() {
                    continue;
                }
            }

            if let Some(promotion) = caps.name("promotion") {
                if m.promotion_kind().to_char().to_string() != promotion.as_str() {
                    continue;
                }
            }

            return Some(m);
        }

        None
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

        // NOTE: This move should end with `#` but this is added in `search::get_pv()`.
        assert_eq!(game.move_to_san(PieceMove::new(F6, G7, CAPTURE)), "Qxg7");
    }

    #[test]
    fn test_move_from_san() {
        let fen = "1q3rk1/Pbpp1p1p/2nb1n1Q/1p2p1pP/2NPP3/1B3N2/1PPB1PP1/R3K2R w KQ g6 0 25";
        let mut game = Game::from_fen(fen);
        assert_eq!(game.move_from_san("O-O"), Some(PieceMove::new(E1, G1, KING_CASTLE)));
        assert_eq!(game.move_from_san("O-O-O"), Some(PieceMove::new(E1, C1, QUEEN_CASTLE)));
        assert_eq!(game.move_from_san("g3"), Some(PieceMove::new(G2, G3, QUIET_MOVE)));
        assert_eq!(game.move_from_san("Ng1"), Some(PieceMove::new(F3, G1, QUIET_MOVE)));
        assert_eq!(game.move_from_san("Ng5"), Some(PieceMove::new(F3, G5, CAPTURE)));
        assert_eq!(game.move_from_san("dxe5"), Some(PieceMove::new(D4, E5, CAPTURE)));
        assert_eq!(game.move_from_san("Nfxe5"), Some(PieceMove::new(F3, E5, CAPTURE)));
        assert_eq!(game.move_from_san("Ncxe5"), Some(PieceMove::new(C4, E5, CAPTURE)));
        assert_eq!(game.move_from_san("a8Q"), Some(PieceMove::new(A7, A8, QUEEN_PROMOTION)));
        assert_eq!(game.move_from_san("axb8N"), Some(PieceMove::new(A7, B8, KNIGHT_PROMOTION_CAPTURE)));
        assert_eq!(game.move_from_san("g4"), Some(PieceMove::new(G2, G4, DOUBLE_PAWN_PUSH)));
        assert_eq!(game.move_from_san("hxg6"), Some(PieceMove::new(H5, G6, EN_PASSANT)));
        assert_eq!(game.move_from_san("hxg6e.p."), Some(PieceMove::new(H5, G6, EN_PASSANT)));
        assert_eq!(game.move_from_san("Qg7"), Some(PieceMove::new(H6, G7, QUIET_MOVE)));
        assert_eq!(game.move_from_san("Qxh7"), Some(PieceMove::new(H6, H7, CAPTURE)));
        assert_eq!(game.move_from_san("Qg7!"), Some(PieceMove::new(H6, G7, QUIET_MOVE)));
        assert_eq!(game.move_from_san("Qxh7!"), Some(PieceMove::new(H6, H7, CAPTURE)));

        for m in game.get_moves() {
            let san = game.move_to_san(m);
            assert_eq!(game.move_from_san(&san), Some(m));
        }
    }
}
