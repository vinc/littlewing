use color::*;
use piece::*;
use square::*;
use common::*;
use attack::Attack;
use attack::piece_attacks;
use bitboard::BitboardExt;
use game::Game;
use piece_move::*;
use piece_move_list::PieceMoveListStage;
use piece::PieceAttr;
use square::SquareExt;
use eval::Eval;

lazy_static! {
    // PxP =  7, PxN = 15, PxB = 23, PxR = 31, PxQ = 39, PxK = 47
    // NxP =  6, NxN = 14, NxB = 22, NxR = 30, NxQ = 38, NxK = 46
    // BxP =  5, BxN = 13, BxB = 21, BxR = 29, BxQ = 37, BxK = 45
    // RxP =  4, RxN = 12, RxB = 20, RxR = 28, RxQ = 36, RxK = 44
    // QxP =  3, QxN = 11, QxB = 19, QxR = 27, QxQ = 35, QxK = 43
    // KxP =  2, KxN = 10, KxB = 18, KxR = 26, KxQ = 34, KxK = 42
    pub static ref MVV_LVA_SCORES: [[u8; 13]; 13] = {
        let pieces = vec![EMPTY, PAWN, KNIGHT, BISHOP, ROOK, QUEEN, KING];
        let mut mvv_lva_scores = [[0; 13]; 13];
        for i in 1..7 {
            for j in 1..7 {
                let a = pieces[i as usize];
                let v = pieces[j as usize];
                mvv_lva_scores[a as usize][v as usize] = (8 * j) - i;
            }
        }
        mvv_lva_scores
    };
}

/// PieceMoveList generator
pub trait PieceMoveGenerator {
    /// Generate the list of moves from the current game position
    fn generate_moves(&mut self);

    /// Sort the moves list to try good candidates first in search
    fn sort_moves(&mut self);

    /// Get the next capture from the moves list (for quiescence search)
    fn next_capture(&mut self) -> Option<PieceMove>;

    /// Get the next move from the moves list (for regular search)
    fn next_move(&mut self) -> Option<PieceMove>;

    /// Make the given move and update the game state
    fn make_move(&mut self, m: PieceMove);

    /// Undo the given move and update the game state
    fn undo_move(&mut self, m: PieceMove);
}

trait PieceMoveGeneratorExt {
    fn is_legal_move(&mut self, m: PieceMove) -> bool;
    fn mvv_lva(&self, m: PieceMove) -> u8;
    fn can_king_castle(&mut self, side: Color) -> bool;
    fn can_queen_castle(&mut self, side: Color) -> bool;
    fn can_castle_on(&mut self, side: Color, wing: Piece) -> bool;
}

impl PieceMoveGenerator for Game {
    fn generate_moves(&mut self) {
        match self.moves.stage() {
            PieceMoveListStage::KillerPieceMove => {
                if !self.moves.skip_killers {
                    for i in 0..MAX_KILLERS {
                        let m = self.moves.get_killer_move(i);
                        if self.is_legal_move(m) {
                            self.moves.add_move(m);
                        }
                    }
                }
            },
            PieceMoveListStage::Capture | PieceMoveListStage::QuietPieceMove => {
                let &position = self.positions.top();
                let side = position.side;
                let ep = position.en_passant;

                self.moves.add_pawns_moves(&self.bitboards, side, ep);
                self.moves.add_knights_moves(&self.bitboards, side);
                self.moves.add_king_moves(&self.bitboards, side);
                self.moves.add_bishops_moves(&self.bitboards, side);
                self.moves.add_rooks_moves(&self.bitboards, side);
                self.moves.add_queens_moves(&self.bitboards, side);

                if self.moves.stage() == PieceMoveListStage::Capture {
                    if !self.moves.skip_ordering {
                        self.sort_moves();
                    }
                } else { // Castlings
                    if self.can_king_castle(side) {
                        self.moves.add_king_castle(side);
                    }
                    if self.can_queen_castle(side) {
                        self.moves.add_queen_castle(side);
                    }
                }
            },
            _ => () // Nothing to do in `BestPieceMove` or `Done` stages
        }
    }

    fn sort_moves(&mut self) {
        // Sort all moves currently in the list except the best move
        let a = if self.moves[0].score == BEST_MOVE_SCORE { 1 } else { 0 };
        let b = self.moves.len();
        for i in a..b {
            if self.moves[i].item.is_capture() {
                self.moves[i].score = self.mvv_lva(self.moves[i].item);
                if self.see(self.moves[i].item) >= 0 {
                    self.moves[i].score += GOOD_CAPTURE_SCORE;
                }
                debug_assert!(self.moves[i].score < BEST_MOVE_SCORE);
            }
            for j in a..i {
                if self.moves[j].score < self.moves[i].score {
                    self.moves.swap(i, j);
                }
            }
        }
    }

    fn next_move(&mut self) -> Option<PieceMove> {
        let mut next_move = self.moves.next();

        // Staged moves generation
        while next_move.is_none() && !self.moves.is_last_stage() {
            self.moves.next_stage();
            self.generate_moves();
            next_move = self.moves.next();
        }

        next_move
    }

    // Specialized version of `next_move` for quiescence search.
    fn next_capture(&mut self) -> Option<PieceMove> {
        if self.moves.stage() == PieceMoveListStage::BestPieceMove {
            self.moves.next_stage();
            self.generate_moves();
            debug_assert_eq!(self.moves.stage(), PieceMoveListStage::Capture);
        }

        // Skip bad captures
        let i = self.moves.index();
        let n = self.moves.len();
        if i < n {
            if self.moves[i].score < GOOD_CAPTURE_SCORE {
                return None;
            }
        }

        self.moves.next()
    }

    fn make_move(&mut self, m: PieceMove) {
        let mut position = *self.positions.top();
        let side = position.side;

        let piece = self.board[m.from() as usize];
        let capture = self.board[m.to() as usize]; // TODO: En passant

        position.halfmoves_count += 1;

        if !m.is_null() {
            self.bitboards[side as usize].toggle(m.from());
            self.bitboards[side as usize].toggle(m.to());
            self.bitboards[piece as usize].toggle(m.from());
            self.board[m.from() as usize] = EMPTY;

            position.hash ^= self.zobrist.pieces[piece as usize][m.from() as usize];
            position.capture = capture;

            if piece.kind() == PAWN {
                position.halfmoves_count = 0;
            } else if piece.kind() == KING || (piece.kind() == ROOK && m.from() == H1.flip(side)) {
                if position.castling_right(side, KING) {
                    position.reset_castling_right(side, KING);
                    position.hash ^= self.zobrist.castling_right(side, KING);
                }
            } else if piece.kind() == KING || (piece.kind() == ROOK && m.from() == A1.flip(side)) {
                if position.castling_right(side, QUEEN) {
                    position.reset_castling_right(side, QUEEN);
                    position.hash ^= self.zobrist.castling_right(side, QUEEN);
                }
            }

            let p = if m.is_promotion() { side | m.promotion_kind() } else { piece };
            self.board[m.to() as usize] = p;
            self.bitboards[p as usize].toggle(m.to());
            position.hash ^= self.zobrist.pieces[p as usize][m.to() as usize];

            if m.is_en_passant() {
                let sq = (((m.to().flip(side) as Shift) + DOWN) as Square).flip(side);
                let pawn = (side ^ 1) | PAWN;
                self.board[sq as usize] = EMPTY;
                self.bitboards[pawn as usize].toggle(sq);
                self.bitboards[(side ^ 1) as usize].toggle(sq);
                position.hash ^= self.zobrist.pieces[pawn as usize][sq as usize];
            } else if capture != EMPTY {
                position.halfmoves_count = 0;
                self.bitboards[capture as usize].toggle(m.to());
                self.bitboards[(side ^ 1) as usize].toggle(m.to());
                position.hash ^= self.zobrist.pieces[capture as usize][m.to() as usize];

                // Update opponent's castling rights on rook capture
                if capture.kind() == ROOK {
                    if m.to() == H1.flip(side ^ 1) {
                        if position.castling_right(side, KING) {
                            position.reset_castling_right(side ^ 1, KING);
                            position.hash ^= self.zobrist.castling_right(side ^ 1, KING);
                        }
                    } else if m.to() == A1.flip(side ^ 1) {
                        if position.castling_right(side, QUEEN) {
                            position.reset_castling_right(side ^ 1, QUEEN);
                            position.hash ^= self.zobrist.castling_right(side ^ 1, QUEEN);
                        }
                    }
                }
            } else if m.is_castle() {
                let rook = side | ROOK;

                let (rook_from, rook_to) = if m.castle_kind() == KING {
                    (H1.flip(side), F1.flip(side))
                } else {
                    (A1.flip(side), D1.flip(side))
                };

                self.board[rook_from as usize] = EMPTY;
                self.board[rook_to as usize] = rook;
                self.bitboards[rook as usize].toggle(rook_from);
                self.bitboards[rook as usize].toggle(rook_to);
                self.bitboards[side as usize].toggle(rook_from);
                self.bitboards[side as usize].toggle(rook_to);
                position.hash ^= self.zobrist.pieces[rook as usize][rook_from as usize];
                position.hash ^= self.zobrist.pieces[rook as usize][rook_to as usize];
            }
        }

        if position.en_passant != OUT {
            position.hash ^= self.zobrist.en_passant[position.en_passant as usize];
        }

        position.en_passant = if m.kind() == DOUBLE_PAWN_PUSH {
            ((((m.from().flip(side)) as Shift) + UP) as Square).flip(side)
        } else {
            OUT
        };

        if position.en_passant != OUT {
            position.hash ^= self.zobrist.en_passant[position.en_passant as usize];
        }

        position.side ^= 1; // TODO: Define Color#flip()
        position.hash ^= self.zobrist.side;

        self.positions.push(position);
        self.moves.inc();
    }

    fn undo_move(&mut self, m: PieceMove) {
        let piece = self.board[m.to() as usize];
        let capture = self.positions.top().capture;

        self.positions.pop();
        self.moves.dec();

        if m.is_null() {
            return;
        }

        let &position = self.positions.top();
        let side = position.side;

        let p = if m.is_promotion() { side | PAWN } else { piece };
        self.board[m.from() as usize] = p;
        self.bitboards[p as usize].toggle(m.from());

        self.bitboards[side as usize].toggle(m.from());
        self.bitboards[side as usize].toggle(m.to());
        self.bitboards[piece as usize].toggle(m.to());
        self.board[m.to() as usize] = capture;

        if capture != EMPTY {
            self.bitboards[capture as usize].toggle(m.to());
            self.bitboards[(side ^ 1) as usize].toggle(m.to());
        } else if m.is_en_passant() {
            let sq = (((m.to().flip(side) as Shift) + DOWN) as Square).flip(side);
            let pawn = side ^ 1 | PAWN;
            self.board[sq as usize] = pawn;
            self.bitboards[pawn as usize].toggle(sq);
            self.bitboards[(side ^ 1) as usize].toggle(sq);
        } else if m.is_castle() {
            let rook = side | ROOK;

            let (rook_from, rook_to) = if m.castle_kind() == KING {
                (H1.flip(side), F1.flip(side))
            } else {
                (A1.flip(side), D1.flip(side))
            };

            self.board[rook_from as usize] = rook;
            self.board[rook_to as usize] = EMPTY;
            self.bitboards[side as usize].toggle(rook_from);
            self.bitboards[side as usize].toggle(rook_to);
            self.bitboards[rook as usize].toggle(rook_from);
            self.bitboards[rook as usize].toggle(rook_to);
        }
    }
}

impl PieceMoveGeneratorExt for Game {
    fn can_castle_on(&mut self, side: Color, wing: Piece) -> bool {
        match wing {
            QUEEN => self.can_queen_castle(side),
            KING  => self.can_king_castle(side),
            _     => unreachable!()
        }
    }

    fn can_king_castle(&mut self, side: Color) -> bool {
        let &position = self.positions.top();
        let occupied = self.bitboards[WHITE as usize] | self.bitboards[BLACK as usize];
        let mask = CASTLING_MASKS[side as usize][(KING >> 3) as usize];

        !occupied & mask == mask &&
        self.board[E1.flip(side) as usize] == side | KING &&
        self.board[H1.flip(side) as usize] == side | ROOK &&
        position.castling_right(side, KING) &&
        !self.is_attacked(E1.flip(side), side) &&
        !self.is_attacked(F1.flip(side), side) &&
        !self.is_attacked(G1.flip(side), side) // TODO: Duplicate with is_check() ?
    }

    fn can_queen_castle(&mut self, side: Color) -> bool {
        let &position = self.positions.top();
        let occupied = self.bitboards[WHITE as usize] | self.bitboards[BLACK as usize];
        let mask = CASTLING_MASKS[side as usize][(QUEEN >> 3) as usize];

        !occupied & mask == mask &&
        self.board[E1.flip(side) as usize] == side | KING &&
        self.board[A1.flip(side) as usize] == side | ROOK &&
        position.castling_right(side, QUEEN) &&
        !self.is_attacked(E1.flip(side), side) &&
        !self.is_attacked(D1.flip(side), side) &&
        !self.is_attacked(C1.flip(side), side)
    }

    // Pseudo legal move checker (limited to moves generated by the engine)
    fn is_legal_move(&mut self, m: PieceMove) -> bool {
        if m.is_null() {
            return false;
        }

        let &position = self.positions.top();
        let side = position.side;

        let p = self.board[m.from() as usize];

        // There must be a piece to play
        if p == EMPTY {
            return false;
        }
        if p.color() != side {
            return false;
        }

        if m.is_promotion() || m.kind() == DOUBLE_PAWN_PUSH {
            if p.kind() != PAWN {
                return false;
            }
        }

        if m.is_en_passant() {
            if p.kind() != PAWN {
                return false;
            }

            if m.to() != position.en_passant {
                return false;
            }

            return true;
        }

        if m.is_castle() {
            let wing = m.castle_kind();

            return self.can_castle_on(side, wing);
        }

        // The piece must be able to reach its destination
        let pieces = self.bitboards[side as usize];
        let targets = self.bitboards[(side ^ 1) as usize];
        let occupied = pieces | targets;
        let attacks = piece_attacks(p, m.from(), occupied);

        if m.is_capture() {
            (attacks & targets).get(m.to())
        } else if p.kind() == PAWN {
            let y = YSHIFTS[side as usize];
            let mut s = m.from();

            s = ((s as Shift) + y) as Square;
            if m.kind() == DOUBLE_PAWN_PUSH {
                if occupied.get(s) {
                    return false;
                }
                s = ((s as Shift) + y) as Square;
            }

            if m.to() != s {
                return false;
            }

            if (RANK_1 | RANK_8).get(s) {
                if !m.is_promotion() {
                    return false;
                }
            }

            !occupied.get(m.to())
        } else {
            (attacks & !occupied).get(m.to())
        }
    }

    fn mvv_lva(&self, m: PieceMove) -> u8 {
        let a = self.board[m.from() as usize].kind();
        let v = if m.is_en_passant() {
            PAWN
        } else {
            self.board[m.to() as usize].kind()
        };

        MVV_LVA_SCORES[a as usize][v as usize]
    }
}

#[cfg(test)]
mod tests {
    use color::*;
    use piece::*;
    use common::*;
    use piece_move::PieceMove;
    use fen::FEN;
    use game::Game;
    use piece_move_notation::PieceMoveNotation;
    use super::*;

    fn perft(fen: &str) -> usize {
        let mut game = Game::from_fen(fen);

        game.moves.next_stage();
        game.generate_moves(); // Captures

        game.moves.next_stage(); // Killer PieceMoveList

        game.moves.next_stage();
        game.generate_moves(); // Quiet moves

        game.moves.len()
    }

    #[test]
    fn test_generate_moves() {
        let fen = DEFAULT_FEN;
        assert_eq!(perft(fen), 20);

        // Pawn right capture
        let fen = "8/8/4k3/4p3/3P4/3K4/8/8 b - -";
        assert_eq!(perft(fen), 9);

        let fen = "8/8/4k3/4p3/3P4/3K4/8/8 w - -";
        assert_eq!(perft(fen), 9);

        // Pawn left capture
        let fen = "8/8/2p5/2p1P3/1p1P4/3P4/8/8 w - -";
        assert_eq!(perft(fen), 3);

        let fen = "8/8/2p5/2p1P3/1p1P4/3P4/8/8 b - -";
        assert_eq!(perft(fen), 3);

        // Bishop
        let fen = "8/8/8/8/3B4/8/8/8 w - -";
        assert_eq!(perft(fen), 13);

        // Rook
        let fen = "8/8/8/8/1r1R4/8/8/8 w - -";
        assert_eq!(perft(fen), 13);
    }

    #[test]
    fn test_make_move_hash() {
        let mut game = Game::from_fen(DEFAULT_FEN);
        game.make_move(PieceMove::new(E2, E3, QUIET_MOVE));
        game.make_move(PieceMove::new(E7, E6, QUIET_MOVE));
        game.make_move(PieceMove::new(E3, E4, QUIET_MOVE));
        game.make_move(PieceMove::new(E6, E5, QUIET_MOVE));
        let hash1 = game.positions.top().hash;
        game.make_move(PieceMove::new(G1, F3, QUIET_MOVE));
        let hash2 = game.positions.top().hash;

        let mut game = Game::from_fen(DEFAULT_FEN);
        game.make_move(PieceMove::new(E2, E4, DOUBLE_PAWN_PUSH));
        game.make_move(PieceMove::new(E7, E5, DOUBLE_PAWN_PUSH));
        // This position is identical to hash1 except for the en passant
        let hash3 = game.positions.top().hash;
        assert_ne!(game.positions.top().hash, hash1);
        game.make_move(PieceMove::new(G1, F3, QUIET_MOVE));
        assert_eq!(game.positions.top().hash, hash2);

        game.make_move(PieceMove::new(G8, F5, QUIET_MOVE));
        game.make_move(PieceMove::new(F3, G1, QUIET_MOVE));
        game.make_move(PieceMove::new(F5, G8, QUIET_MOVE));
        // Back to hash1 position
        assert_eq!(game.positions.top().hash, hash1);
        assert_ne!(game.positions.top().hash, hash3);
        game.make_move(PieceMove::new(G1, F3, QUIET_MOVE));
        assert_eq!(game.positions.top().hash, hash2);
    }

    #[test]
    fn test_make_undo_move_hash() {
        let mut game = Game::from_fen(DEFAULT_FEN);
        let hash = game.positions.top().hash;

        let moves = vec![
            PieceMove::new(E2, E4, DOUBLE_PAWN_PUSH),
            PieceMove::new(E7, E5, DOUBLE_PAWN_PUSH),
            PieceMove::new(G1, F3, QUIET_MOVE),
            PieceMove::new(B8, C6, QUIET_MOVE),
            PieceMove::new(F1, B5, QUIET_MOVE),
            PieceMove::new(A7, A6, QUIET_MOVE),
            PieceMove::new(B5, C6, CAPTURE),
            PieceMove::new(B7, B5, DOUBLE_PAWN_PUSH),
            PieceMove::new(C6, D7, CAPTURE),
            PieceMove::new(E8, D7, CAPTURE),
            PieceMove::new(E1, G1, KING_CASTLE),
            PieceMove::new(C8, B7, QUIET_MOVE)
        ];

        for m in moves.iter() {
            game.make_move(*m);
            let fen = game.to_fen();
            println!("{} {}", fen, m.to_can());
            let copy = Game::from_fen(&fen);
            assert_eq!(copy.to_fen().as_str(), fen);
            assert_eq!(copy.positions.top().hash, game.positions.top().hash);
        }

        for m in moves.iter().rev() {
            game.undo_move(*m);
            let fen = game.to_fen();
            let copy = Game::from_fen(&fen);
            assert_eq!(copy.to_fen().as_str(), fen);
            assert_eq!(copy.positions.top().hash, game.positions.top().hash);
        }

        assert_eq!(game.to_fen().as_str(), DEFAULT_FEN);
        assert_eq!(game.positions.top().hash, hash);
    }

    #[test]
    fn test_make_undo_move() {
        let positions = vec![
            (PieceMove::new(E2, E3, QUIET_MOVE), "rnbqkbnr/pppppppp/8/8/8/4P3/PPPP1PPP/RNBQKBNR b KQkq - 0 1"),
            (PieceMove::new(E2, E4, DOUBLE_PAWN_PUSH), "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1")
        ];

        for (m, fen) in positions {
            let mut game = Game::from_fen(DEFAULT_FEN);
            let hash = game.positions.top().hash;

            game.make_move(m);
            assert_eq!(game.to_fen().as_str(), fen);

            game.undo_move(m);
            assert_eq!(game.to_fen().as_str(), DEFAULT_FEN);
            assert_eq!(game.positions.top().hash, hash);
        }
    }

    #[test]
    fn test_capture() {
        let fens = [
            "r1bqkbnr/1ppp1ppp/p1n5/1B2p3/4P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 0 1",
            "r1bqkbnr/1ppp1ppp/p1B5/4p3/4P3/5N2/PPPP1PPP/RNBQK2R b KQkq - 0 1"
        ];
        let m = PieceMove::new(B5, C6, CAPTURE);

        let mut game = Game::from_fen(fens[0]);
        assert_eq!(game.to_fen().as_str(), fens[0]);
        assert_eq!(game.positions.len(), 1);
        assert_eq!(game.positions.top().capture, EMPTY);
        assert_eq!(game.positions[0].capture, EMPTY);
        assert_eq!(game.positions[0].side, WHITE);

        game.make_move(m);
        assert_eq!(game.to_fen().as_str(), fens[1]);
        assert_eq!(game.positions.len(), 2);
        assert_eq!(game.positions.top().capture, BLACK_KNIGHT);
        assert_eq!(game.positions[0].capture, EMPTY);
        assert_eq!(game.positions[0].side, WHITE);
        assert_eq!(game.positions[1].capture, BLACK_KNIGHT);
        assert_eq!(game.positions[1].side, BLACK);

        game.undo_move(m);
        assert_eq!(game.to_fen().as_str(), fens[0]);
        assert_eq!(game.positions.len(), 1);
        assert_eq!(game.positions.top().capture, EMPTY);
        assert_eq!(game.positions[0].capture, EMPTY);
        assert_eq!(game.positions[0].side, WHITE);
    }

    #[test]
    fn test_mvv_lva() {
        let mut game = Game::from_fen("8/8/8/8/8/1Qn5/1PpK1k2/8 w - - 0 1");

        assert_eq!(game.mvv_lva(PieceMove::new(B2, C3, CAPTURE)), 15); // PxN
        assert_eq!(game.mvv_lva(PieceMove::new(B3, C3, CAPTURE)), 11); // QxN
        assert_eq!(game.mvv_lva(PieceMove::new(D2, C3, CAPTURE)), 10); // KxN
        assert_eq!(game.mvv_lva(PieceMove::new(B3, C2, CAPTURE)),  3); // QxP
        assert_eq!(game.mvv_lva(PieceMove::new(D2, C2, CAPTURE)),  2); // KxP

        game.moves.next_stage(); // Captures
        game.generate_moves();
        game.moves.next_stage(); // Killer moves
        game.moves.next_stage(); // Quiet moves
        game.generate_moves();

        assert_eq!(game.moves.next(), Some(PieceMove::new(B2, C3, CAPTURE)));
        assert_eq!(game.moves.next(), Some(PieceMove::new(B3, C3, CAPTURE)));
        assert_eq!(game.moves.next(), Some(PieceMove::new(D2, C3, CAPTURE)));
        assert_eq!(game.moves.next(), Some(PieceMove::new(B3, C2, CAPTURE)));
        assert_eq!(game.moves.next(), Some(PieceMove::new(D2, C2, CAPTURE)));

        assert!(!game.moves.next().unwrap().is_capture());
    }

    #[test]
    fn test_make_move_update_halfmoves_count() {
        let fen = "7r/k7/7p/r2p3P/p2PqB2/2R3P1/5K2/3Q3R w - - 25 45";
        let mut game = Game::from_fen(fen);

        assert_eq!(game.positions.top().halfmoves_count, 25);

        game.make_move(PieceMove::new(F2, G1, QUIET_MOVE));
        assert_eq!(game.positions.top().halfmoves_count, 26);

        game.make_move(PieceMove::new(A7, B7, QUIET_MOVE));
        assert_eq!(game.positions.top().halfmoves_count, 27);

        game.make_move(PieceMove::new(F4, H6, CAPTURE));
        assert_eq!(game.positions.top().halfmoves_count, 0);
    }

    #[test]
    fn test_next_move() {
        let fen = "k1K5/8/8/8/8/1p6/2P5/N7 w - - 0 1";
        let mut game = Game::from_fen(fen);

        game.moves.add_move(PieceMove::new(C2, C3, QUIET_MOVE)); // Best move

        assert_eq!(game.next_move(), Some(PieceMove::new(C2, C3, QUIET_MOVE)));
        assert_eq!(game.next_move(), Some(PieceMove::new(C2, B3, CAPTURE)));
        assert_eq!(game.next_move(), Some(PieceMove::new(A1, B3, CAPTURE)));
        assert_eq!(game.next_move(), Some(PieceMove::new(C2, C4, DOUBLE_PAWN_PUSH)));
        assert_eq!(game.next_move(), Some(PieceMove::new(C8, B7, QUIET_MOVE))); // Illegal
        assert_eq!(game.next_move(), Some(PieceMove::new(C8, C7, QUIET_MOVE)));
        assert_eq!(game.next_move(), Some(PieceMove::new(C8, D7, QUIET_MOVE)));
        assert_eq!(game.next_move(), Some(PieceMove::new(C8, B8, QUIET_MOVE))); // Illegal
        assert_eq!(game.next_move(), Some(PieceMove::new(C8, D8, QUIET_MOVE)));
        assert_eq!(game.next_move(), None);
    }

    #[test]
    fn test_next_capture() {
        let fen = "k1K5/8/8/8/8/1p6/2P5/N7 w - - 0 1";
        let mut game = Game::from_fen(fen);

        assert_eq!(game.next_capture(), Some(PieceMove::new(C2, B3, CAPTURE)));
        assert_eq!(game.next_capture(), Some(PieceMove::new(A1, B3, CAPTURE)));
        assert_eq!(game.next_capture(), None);

        let fen = "k1K5/8/2p1N3/1p6/2rp1n2/1P2P3/3Q4/8 w - - 0 1";
        let mut game = Game::from_fen(fen);

        let b3c4 = PieceMove::new(B3, C4, CAPTURE);
        let e3f4 = PieceMove::new(E3, F4, CAPTURE);
        let e6f4 = PieceMove::new(E6, F4, CAPTURE);
        let e3d4 = PieceMove::new(E3, D4, CAPTURE);
        let e6d4 = PieceMove::new(E6, D4, CAPTURE);
        let d2d4 = PieceMove::new(D2, D4, CAPTURE);
        println!("{}: {}", b3c4, game.see(b3c4));
        println!("{}: {}", e3f4, game.see(e3f4));
        println!("{}: {}", e6f4, game.see(e6f4));
        println!("{}: {}", e3d4, game.see(e3d4));
        println!("{}: {}", e6d4, game.see(e6d4));
        println!("{}: {}", d2d4, game.see(d2d4));
        assert_eq!(game.next_capture(), Some(b3c4));
        assert_eq!(game.next_capture(), Some(e3f4));
        assert_eq!(game.next_capture(), Some(e6f4));
        assert_eq!(game.next_capture(), Some(e3d4));
        assert_eq!(game.next_capture(), Some(e6d4));
        //assert_eq!(game.next_capture(), Some(d2d4)); // Skip bad capture
        assert_eq!(game.next_capture(), None);
    }

    #[test]
    fn test_is_legal_move() {
        let fen = "k1K5/8/8/8/8/1p6/2P5/N7 w - - 0 1";
        let mut game = Game::from_fen(fen);

        assert!(game.is_legal_move(PieceMove::new(C2, C3, QUIET_MOVE)));
        assert!(game.is_legal_move(PieceMove::new(C2, B3, CAPTURE)));
        assert!(game.is_legal_move(PieceMove::new(A1, B3, CAPTURE)));
        assert!(game.is_legal_move(PieceMove::new(C2, C4, DOUBLE_PAWN_PUSH)));
        assert!(game.is_legal_move(PieceMove::new(C8, C7, QUIET_MOVE)));
        assert!(game.is_legal_move(PieceMove::new(C8, D7, QUIET_MOVE)));
        assert!(game.is_legal_move(PieceMove::new(C8, D8, QUIET_MOVE)));

        assert!(!game.is_legal_move(PieceMove::new_null()));

        assert!(!game.is_legal_move(PieceMove::new(H1, H5, QUIET_MOVE)));

        // Cannot be done with pseudo legal move checking
        //assert!(!game.is_legal_move(PieceMove::new(C8, B8, QUIET_MOVE))); // Illegal
        //assert!(!game.is_legal_move(PieceMove::new(C8, B7, QUIET_MOVE))); // Illegal
    }

    #[test]
    fn test_moves_order() {
        let fen = "rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 2";
        let mut game = Game::from_fen(fen);

        let capture = game.move_from_can("e4d5");
        let first_quiet_move = game.move_from_can("a2a3");

        game.moves.clear();

        let mut n = 0;
        while let Some(m) = game.next_move() {
            match n {
                0 => assert_eq!(m, capture),
                1 => assert_eq!(m, first_quiet_move),
                _ => {}
            }
            n += 1;
        }
        assert_eq!(n, 31);
    }

    #[test]
    fn test_moves_order_with_best_and_killer_moves() {
        let fen = "rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 2";
        let mut game = Game::from_fen(fen);

        let capture = game.move_from_can("e4d5");
        let first_quiet_move = game.move_from_can("a2a3");

        let first_killer_move = game.move_from_can("f1b5");
        game.moves.add_killer_move(first_killer_move);

        game.moves.clear();

        let best_move = game.move_from_can("b1c3");
        game.moves.add_move(best_move);

        let mut n = 0;
        while let Some(m) = game.next_move() {
            match n {
                0 => assert_eq!(m, best_move),
                1 => assert_eq!(m, capture),
                2 => assert_eq!(m, first_killer_move),
                3 => assert_eq!(m, first_quiet_move),
                _ => {}
            }
            n += 1;
        }
        assert_eq!(n, 31);
    }

    #[test]
    fn test_moves_order_when_best_move_is_quiet_move() {
        // Ruy Lopez Opening: Morphy Defense (1. e4 e5 2. Nf3 Nc6 3. Bb5 a6)
        let fen = "r1bqkbnr/1ppp1ppp/p1n5/1B2p3/4P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 0 4";
        let mut game = Game::from_fen(fen);

        let best_move     = game.move_from_can("b5a4");
        let good_capture  = game.move_from_can("b5c6");
        let bad_capture_1 = game.move_from_can("f3e5");
        let bad_capture_2 = game.move_from_can("b5a6");
        let quiet_move_1  = game.move_from_can("a2a3");
        let killer_move_1 = game.move_from_can("b5c4");

        game.moves.add_killer_move(killer_move_1);
        game.moves.clear();
        game.moves.add_move(best_move);

        let mut n = 0;
        while let Some(m) = game.next_move() {
            match n {
                0 => assert_eq!(m, best_move),
                1 => assert_eq!(m, good_capture),
                2 => assert_eq!(m, bad_capture_1),
                3 => assert_eq!(m, bad_capture_2),
                4 => assert_eq!(m, killer_move_1),
                5 => assert_eq!(m, quiet_move_1),
                _ => {}
            }
            n += 1;
        }
        assert_eq!(n, 32);
    }

    #[test]
    fn test_moves_order_when_best_move_is_bad_capture() {
        // Ruy Lopez Opening: Morphy Defense (1. e4 e5 2. Nf3 Nc6 3. Bb5 a6)
        let fen = "r1bqkbnr/1ppp1ppp/p1n5/1B2p3/4P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 0 4";
        let mut game = Game::from_fen(fen);

        let good_capture  = game.move_from_can("b5c6");
        let bad_capture_1 = game.move_from_can("f3e5");
        let bad_capture_2 = game.move_from_can("b5a6");
        let quiet_move_1  = game.move_from_can("a2a3");
        let killer_move_1 = game.move_from_can("b5c4");

        let best_move = bad_capture_2;

        game.moves.add_killer_move(killer_move_1);
        game.moves.clear();
        game.moves.add_move(best_move);

        let mut n = 0;
        while let Some(m) = game.next_move() {
            match n {
                0 => assert_eq!(m, best_move),
                1 => assert_eq!(m, good_capture),
                2 => assert_eq!(m, bad_capture_1),
                3 => assert_eq!(m, killer_move_1),
                4 => assert_eq!(m, quiet_move_1),
                _ => {}
            }
            n += 1;
        }
        assert_eq!(n, 32);
    }
}
