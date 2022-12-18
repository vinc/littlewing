use std::prelude::v1::*;

#[cfg(feature = "std")]
use regex::Regex;

use crate::attack::piece_attacks;
use crate::bitboard::BitboardExt;
use crate::color::*;
use crate::common::*;
use crate::game::Game;
use crate::piece::*;
use crate::piece::{PieceAttr, PieceChar};
use crate::piece_move::*;
use crate::square::*;
use crate::square::SquareExt;
#[cfg(feature = "std")]
use crate::search::Search;
use crate::attack::Attack;
use crate::piece_move_generator::PieceMoveGenerator;

#[cfg(feature = "std")]
static RE_LAN: &str = r"^(?P<from>[a-h][1-8])(?P<to>[a-h][1-8])(?P<promotion>[nbrq])?$";

#[cfg(feature = "std")]
static RE_SAN: &str = r"(?x)
    ^(?P<piece>[NBRQK])?(?P<file>[a-h])?(?P<rank>[1-8])?(?P<capture>x)?(?P<to>[a-h][1-8])=?(?P<promotion>[KBRQ])?
    |(?P<queen>O-O-O)
    |(?P<king>O-O)";

/// PieceMoveList generator
pub trait PieceMoveNotation {
    /// Parse move from string
    #[cfg(feature = "std")]
    fn parse_move(&mut self, s: &str) -> Option<PieceMove>;

    /// Get move from string in long algebraic notation (LAN)
    fn move_from_lan(&mut self, s: &str) -> PieceMove;

    /// Get move from string in standard algebraic notation (SAN)
    #[cfg(feature = "std")]
    fn move_from_san(&mut self, s: &str) -> Option<PieceMove>;

    /// Get SAN string from move
    fn move_to_san(&mut self, m: PieceMove) -> String;

    /// Check parsed move legality (slow)
    fn is_parsed_move_legal(&mut self, m: PieceMove) -> bool;
}

trait PieceMoveNotationExt {
    #[cfg(feature = "std")]
    fn move_from_lan_checked(&mut self, s: &str) -> Option<PieceMove>;
}

impl PieceMoveNotation for Game {
    #[cfg(feature = "std")]
    fn parse_move(&mut self, s: &str) -> Option<PieceMove> {
        self.move_from_san(s).or(self.move_from_lan_checked(s))
    }

    fn move_from_lan(&mut self, s: &str) -> PieceMove {
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

    #[cfg(feature = "std")]
    fn move_from_san(&mut self, s: &str) -> Option<PieceMove> {
        lazy_static! {
            static ref RE: Regex = Regex::new(RE_SAN).unwrap();
        }
        let caps = match RE.captures(s) {
            Some(caps) => caps,
            None => return None,
        };

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
        let mut out = String::new();

        if m.is_castle() {
            if m.castle_kind() == KING {
                out.push_str("O-O");
            } else {
                out.push_str("O-O-O");
            }
            return out;
        }

        let piece = self.board[m.from() as usize];
        if !piece.is_pawn() {
            out.push(piece.kind().to_char());
        }

        // Piece disambiguation or pawn capture
        if !piece.is_pawn() || m.is_capture() || m.is_en_passant() {
            let occupied = self.bitboard(WHITE) | self.bitboard(BLACK);
            let pieces = self.bitboard(piece);
            let attacks = piece_attacks(piece, m.to(), occupied);
            let attackers = pieces & attacks;
            if attackers.count() > 1 || piece.is_pawn() {
                if (attackers != attackers & FILES[m.from().file() as usize]) || piece.is_pawn() {
                    out.push(m.from().file_to_char());
                } else if attackers != attackers & RANKS[m.from().rank() as usize] {
                    out.push(m.from().rank_to_char());
                } else {
                    out.push(m.from().file_to_char());
                    out.push(m.from().rank_to_char());
                }
            }
        }

        // TODO: Should en passant be a capture?
        if m.is_capture() || m.is_en_passant() {
            out.push('x');
        }

        out.push_str(m.to().to_coord().as_str());

        if m.is_promotion() {
            out.push(m.promotion_kind().to_char());
        }

        out
    }

    // This method is slow compared to `is_move_legal` but it is exhaustive
    // and applicable to any parsed move instead of only moves generated by
    // the engine
    fn is_parsed_move_legal(&mut self, m: PieceMove) -> bool {
        let side = self.side();
        self.moves.clear();
        while let Some(generated_move) = self.next_move() {
            if m == generated_move {
                self.make_move(m);
                let is_legal = !self.is_check(side);
                self.undo_move(m);
                return is_legal;
            }
        }
        false
    }
}

impl PieceMoveNotationExt for Game {
    #[cfg(feature = "std")]
    fn move_from_lan_checked(&mut self, s: &str) -> Option<PieceMove> {
        lazy_static! {
            static ref RE: Regex = Regex::new(RE_LAN).unwrap();
        }
        if RE.is_match(s) {
            Some(self.move_from_lan(s))
        } else {
            None
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::common::*;
    use crate::piece_move::PieceMove;
    use crate::fen::FEN;
    use crate::game::Game;
    use super::*;

    #[test]
    fn test_move_from_lan() {
        let mut game = Game::from_fen(DEFAULT_FEN).unwrap();

        let m = game.move_from_lan("e2e4");
        assert_eq!(m, PieceMove::new(E2, E4, DOUBLE_PAWN_PUSH));

        let m = game.move_from_lan("g1f3");
        assert_eq!(m, PieceMove::new(G1, F3, QUIET_MOVE));
    }

    #[test]
    fn test_move_from_lan_checked() {
        let mut game = Game::from_fen(DEFAULT_FEN).unwrap();

        let m = game.move_from_lan_checked("e2e4");
        assert_eq!(m, Some(PieceMove::new(E2, E4, DOUBLE_PAWN_PUSH)));

        let m = game.move_from_lan_checked("g1f3");
        assert_eq!(m, Some(PieceMove::new(G1, F3, QUIET_MOVE)));

        let m = game.move_from_lan_checked("none");
        assert_eq!(m, None);
    }

    #[test]
    fn test_move_to_san() {
        let fen = "7k/3P1ppp/4PQ2/8/8/8/8/6RK w - - 0 1";
        let mut game = Game::from_fen(fen).unwrap();

        // NOTE: This move should end with `#` but this is added in `search::get_pv()`.
        assert_eq!(game.move_to_san(PieceMove::new(F6, G7, CAPTURE)), "Qxg7");

        let fen = "1q3rk1/Pbpp1p1p/2nb1n1Q/1p2p1pP/2NPP3/1B3N2/1PPB1PP1/R3K2R w KQ g6 0 25";
        let mut game = Game::from_fen(fen).unwrap();
        assert_eq!(game.move_to_san(PieceMove::new(D4, E5, CAPTURE)), "dxe5");
        assert_eq!(game.move_to_san(PieceMove::new(F3, E5, CAPTURE)), "Nfxe5");
        assert_eq!(game.move_to_san(PieceMove::new(C4, E5, CAPTURE)), "Ncxe5");
        assert_eq!(game.move_to_san(PieceMove::new(A7, B8, KNIGHT_PROMOTION_CAPTURE)), "axb8N");
        assert_eq!(game.move_to_san(PieceMove::new(H5, G6, EN_PASSANT)), "hxg6");

        let fen = "1q3rk1/Pbpp1p1p/2nb1n1Q/1p2p2P/2NPP1p1/1B3N2/1PPB1PP1/R4RK1 w - - 0 26";
        let mut game = Game::from_fen(fen).unwrap();
        assert_eq!(game.move_to_san(PieceMove::new(A1, E1, QUIET_MOVE)), "Rae1");
        assert_eq!(game.move_to_san(PieceMove::new(F1, E1, QUIET_MOVE)), "Rfe1");

        let fen = "1q3rk1/Pbpp1p1p/2nb1n1Q/Rp2p2P/2NPP1p1/1B3N2/1PPB1PP1/R5K1 w - - 4 28";
        let mut game = Game::from_fen(fen).unwrap();
        assert_eq!(game.move_to_san(PieceMove::new(A5, A4, QUIET_MOVE)), "R5a4");
        assert_eq!(game.move_to_san(PieceMove::new(A1, A4, QUIET_MOVE)), "R1a4");
    }

    #[test]
    fn test_move_from_san() {
        let fen = "1q3rk1/Pbpp1p1p/2nb1n1Q/1p2p1pP/2NPP3/1B3N2/1PPB1PP1/R3K2R w KQ g6 0 25";
        let mut game = Game::from_fen(fen).unwrap();
        assert_eq!(game.move_from_san("none"), None);
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

        let fen = "1q3rk1/Pbpp1p1p/2nb1n1Q/1p2p2P/2NPP1p1/1B3N2/1PPB1PP1/R4RK1 w - - 0 26";
        let mut game = Game::from_fen(fen).unwrap();
        assert_eq!(game.move_from_san("Rae1"), Some(PieceMove::new(A1, E1, QUIET_MOVE)));
        assert_eq!(game.move_from_san("Rfe1"), Some(PieceMove::new(F1, E1, QUIET_MOVE)));
        for m in game.get_moves() {
            let san = game.move_to_san(m);
            assert_eq!(game.move_from_san(&san), Some(m));
        }

        let fen = "1q3rk1/Pbpp1p1p/2nb1n1Q/Rp2p2P/2NPP1p1/1B3N2/1PPB1PP1/R5K1 w - - 4 28";
        let mut game = Game::from_fen(fen).unwrap();
        assert_eq!(game.move_from_san("R5a4"), Some(PieceMove::new(A5, A4, QUIET_MOVE)));
        assert_eq!(game.move_from_san("R1a4"), Some(PieceMove::new(A1, A4, QUIET_MOVE)));
        for m in game.get_moves() {
            let san = game.move_to_san(m);
            assert_eq!(game.move_from_san(&san), Some(m));
        }
    }

    #[test]
    fn test_parse_move() {
        let fen = "1q3rk1/Pbpp1p1p/2nb1n1Q/1p2p1pP/2NPP3/1B3N2/1PPB1PP1/R3K2R w KQ g6 0 25";
        let mut game = Game::from_fen(fen).unwrap();
        assert_eq!(game.parse_move("O-O"), Some(PieceMove::new(E1, G1, KING_CASTLE)));
        assert_eq!(game.parse_move("O-O-O"), Some(PieceMove::new(E1, C1, QUEEN_CASTLE)));
        assert_eq!(game.parse_move("g3"), Some(PieceMove::new(G2, G3, QUIET_MOVE)));
        assert_eq!(game.parse_move("Ng1"), Some(PieceMove::new(F3, G1, QUIET_MOVE)));
        assert_eq!(game.parse_move("g2g3"), Some(PieceMove::new(G2, G3, QUIET_MOVE)));
        assert_eq!(game.parse_move("f3g1"), Some(PieceMove::new(F3, G1, QUIET_MOVE)));
    }
}
