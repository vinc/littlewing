use common::*;
use attack::Attack;
use attack::attacks;
use bitboard::BitboardExt;
use game::Game;
use moves::Move;
use piece::{PieceAttr, PieceChar};
use square::SquareString;

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

pub trait MovesGenerator {
    fn generate_moves(&mut self);
    fn next_move(&mut self) -> Option<Move>;
    fn make_move(&mut self, m: Move);
    fn undo_move(&mut self, m: Move);
    fn sort_moves(&mut self);
    fn move_to_san(&mut self, m: Move) -> String;

    fn mvv_lva(&self, m: Move) -> u8;
}

impl MovesGenerator for Game {
    fn make_move(&mut self, m: Move) {
        let &position = self.positions.top();
        let mut new_position = position;
        let side = position.side;

        let piece = self.board[m.from() as usize];
        let capture = self.board[m.to() as usize]; // TODO: En passant

        self.bitboards[piece as usize].toggle(m.from());
        self.board[m.from() as usize] = EMPTY;
        new_position.hash ^= self.zobrist.positions[piece as usize][m.from() as usize];

        // Update castling rights
        if piece.kind() == KING {
            new_position.castling_rights[side as usize][(KING >> 3) as usize] = false;
            new_position.castling_rights[side as usize][(QUEEN >> 3) as usize] = false;
            new_position.hash ^= self.zobrist.castling_rights[side as usize][(KING >> 3) as usize];
            new_position.hash ^= self.zobrist.castling_rights[side as usize][(QUEEN >> 3) as usize];
        } else if piece.kind() == ROOK {
            if m.from() == H1 ^ 56 * side {
                new_position.castling_rights[side as usize][(KING >> 3) as usize] = false;
                new_position.hash ^= self.zobrist.castling_rights[side as usize][(KING >> 3) as usize];
            }
            if m.from() == A1 ^ 56 * side {
                new_position.castling_rights[side as usize][(QUEEN >> 3) as usize] = false;
                new_position.hash ^= self.zobrist.castling_rights[side as usize][(QUEEN >> 3) as usize];
            }
        }
        if capture.kind() == ROOK {
            if m.to() == H1 ^ 56 * (side ^ 1) {
                new_position.castling_rights[(side ^ 1) as usize][(KING >> 3) as usize] = false;
                new_position.hash ^= self.zobrist.castling_rights[(side ^ 1) as usize][(KING >> 3) as usize];
            }
            if m.to() == A1 ^ 56 * (side ^ 1) {
                new_position.castling_rights[(side ^ 1) as usize][(QUEEN >> 3) as usize] = false;
                new_position.hash ^= self.zobrist.castling_rights[(side ^ 1) as usize][(QUEEN >> 3) as usize];
            }
        }

        if m.is_castle() {
            let rook = side | ROOK;

            let (rook_from, rook_to) = if m.castle_kind() == KING {
                (H1 ^ 56 * side, F1 ^ 56 * side)
            } else {
                (A1 ^ 56 * side, D1 ^ 56 * side)
            };

            self.board[rook_from as usize] = EMPTY;
            self.board[rook_to as usize] = rook;
            self.bitboards[rook as usize].toggle(rook_from);
            self.bitboards[rook as usize].toggle(rook_to);
            self.bitboards[side as usize].toggle(rook_from);
            self.bitboards[side as usize].toggle(rook_to);
            new_position.hash ^= self.zobrist.positions[rook as usize][rook_from as usize];
            new_position.hash ^= self.zobrist.positions[rook as usize][rook_to as usize];
        }

        if m.is_promotion() {
            let promoted_piece = side | m.promotion_kind();
            self.board[m.to() as usize] = promoted_piece;
            self.bitboards[promoted_piece as usize].toggle(m.to());
            new_position.hash ^= self.zobrist.positions[promoted_piece as usize][m.to() as usize];
        } else {
            self.board[m.to() as usize] = piece;
            self.bitboards[piece as usize].toggle(m.to());
            new_position.hash ^= self.zobrist.positions[piece as usize][m.to() as usize];
        }

        new_position.en_passant = if m.kind() == DOUBLE_PAWN_PUSH {
            //((m.from() ^ (56 * side)) + UP) ^ (56 * side)
            ((((m.from() as i8) ^ (56 * (side as i8))) + UP) ^ (56 * (side as i8))) as Square
        } else {
            OUT
        };

        if position.en_passant != OUT {
            new_position.hash ^= self.zobrist.en_passant[position.en_passant as usize]; // TODO ?
        }
        if new_position.en_passant != OUT {
            new_position.hash ^= self.zobrist.en_passant[new_position.en_passant as usize];
        }

        self.bitboards[side as usize].toggle(m.from());
        self.bitboards[side as usize].toggle(m.to());

        if capture != EMPTY {
            self.bitboards[capture as usize].toggle(m.to());
            self.bitboards[(side ^ 1) as usize].toggle(m.to());
            new_position.hash ^= self.zobrist.positions[capture as usize][m.to() as usize];
        }

        if m.kind() == EN_PASSANT {
            //let square = ((m.to() ^ (56 * side)) + DOWN) ^ (56 * side);
            let square = ((((m.to() as i8) ^ (56 * (side as i8))) + DOWN) ^ (56 * (side as i8))) as Square;
            self.board[square as usize] = EMPTY;
            self.bitboards[(side ^ 1 | PAWN) as usize].toggle(square);
            self.bitboards[(side ^ 1) as usize].toggle(square);
            new_position.hash ^= self.zobrist.positions[(side ^ 1 | PAWN) as usize][square as usize];
        }

        // FIXME
        new_position.side ^= 1; // TODO: Define self.side.toggle(0)
        new_position.capture = capture;
        new_position.hash ^= self.zobrist.side;

        self.positions.push(new_position);
        self.moves.inc();
    }

    fn undo_move(&mut self, m: Move) {
        let piece = self.board[m.to() as usize];
        let capture = self.positions.top().capture;

        self.positions.pop();
        self.moves.dec();

        let &position = self.positions.top();
        let side = position.side;

        if m.is_castle() {
            let rook = side | ROOK;

            let (rook_from, rook_to) = if m.castle_kind() == KING {
                (H1 ^ 56 * side, F1 ^ 56 * side)
            } else {
                (A1 ^ 56 * side, D1 ^ 56 * side)
            };

            self.board[rook_from as usize] = rook;
            self.board[rook_to as usize] = EMPTY;
            self.bitboards[rook as usize].toggle(rook_from);
            self.bitboards[rook as usize].toggle(rook_to);
            self.bitboards[side as usize].toggle(rook_from);
            self.bitboards[side as usize].toggle(rook_to);
        }

        if m.is_promotion() {
            let pawn = position.side | PAWN;
            self.board[m.from() as usize] = pawn;
            self.bitboards[pawn as usize].toggle(m.from());
        } else {
            self.board[m.from() as usize] = piece;
            self.bitboards[piece as usize].toggle(m.from());
        }

        if m.kind() == EN_PASSANT {
            //let square = ((m.to() ^ (56 * side)) + DOWN) ^ (56 * side);
            let square = ((((m.to() as i8) ^ (56 * (side as i8))) + DOWN) ^ (56 * (side as i8))) as Square;
            self.board[square as usize] = side ^ 1 | PAWN;
            self.bitboards[(side ^ 1 | PAWN) as usize].toggle(square);
            self.bitboards[(side ^ 1) as usize].toggle(square);
        }

        self.board[m.to() as usize] = capture;
        self.bitboards[piece as usize].toggle(m.to());

        self.bitboards[position.side as usize].toggle(m.from());
        self.bitboards[position.side as usize].toggle(m.to());

        if capture != EMPTY {
            self.bitboards[capture as usize].toggle(m.to());
            self.bitboards[(position.side ^ 1) as usize].toggle(m.to());
        }
    }

    fn generate_moves(&mut self) {
        // TODO: make sure that `moves.clear()` has been called at this ply
        let &position = self.positions.top();
        let side = position.side;
        let ep = position.en_passant;

        self.moves.add_pawns_moves(&self.bitboards, side, ep);
        self.moves.add_knights_moves(&self.bitboards, side);
        self.moves.add_king_moves(&self.bitboards, side);
        self.moves.add_bishops_moves(&self.bitboards, side);
        self.moves.add_rooks_moves(&self.bitboards, side);
        self.moves.add_queens_moves(&self.bitboards, side);


        let occupied = self.bitboards[WHITE as usize] | self.bitboards[BLACK as usize];

        let mask = CASTLING_MASKS[side as usize][(KING >> 3) as usize];
        let can_castle =
            !occupied & mask == mask &&
            position.castling_rights[side as usize][(KING >> 3) as usize] &&
            !self.is_attacked(E1 ^ 56 * side, side) &&
            !self.is_attacked(F1 ^ 56 * side, side) &&
            !self.is_attacked(G1 ^ 56 * side, side); // TODO: Duplicate with is_check() ?
        if can_castle {
            self.moves.add_king_castle(side);
        }

        let mask = CASTLING_MASKS[side as usize][(QUEEN >> 3) as usize];
        let can_castle =
            !occupied & mask == mask &&
            position.castling_rights[side as usize][(QUEEN >> 3) as usize] &&
            !self.is_attacked(E1 ^ 56 * side, side) &&
            !self.is_attacked(D1 ^ 56 * side, side) &&
            !self.is_attacked(C1 ^ 56 * side, side);
        if can_castle {
            self.moves.add_queen_castle(side);
        }
    }

    // NOTE: this function assumes that the move has not been played yet
    fn move_to_san(&mut self, m: Move) -> String {
        let side = self.positions.top().side;
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
            let occupied = self.bitboards[piece as usize];
            let attackers = attacks(piece, m.to(), occupied) & occupied;
            if attackers.count_ones() > 1 || piece.kind() == PAWN {
                let rank = m.from().to_coord().as_str().chars().nth(0).unwrap();
                out.push(rank);
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

        self.make_move(m);
        if self.is_check(side ^ 1) {
            out.push('+');
        }
        self.undo_move(m);

        out
    }

    fn sort_moves(&mut self) {
        let a = self.moves.len_best_moves();
        let b = self.moves.len();
        for i in a..b {
            if self.moves[i].m.is_capture() {
                self.moves[i].s = self.mvv_lva(self.moves[i].m);
            }
            for j in a..i {
                if self.moves[j].s < self.moves[i].s {
                    self.moves.swap(i, j);
                }
            }
        }
    }

    fn mvv_lva(&self, m: Move) -> u8 {
        let a = self.board[m.from() as usize].kind();
        let v = if m.is_en_passant() {
            PAWN
        } else {
            self.board[m.to() as usize].kind()
        };

        MVV_LVA_SCORES[a as usize][v as usize]
    }

    fn next_move(&mut self) -> Option<Move> {
        let old_state = self.moves.state();

        self.moves.update_state();

        let new_state = self.moves.state();

        // First we search the best move if there is one,
        // then we generate all the other moves and search those.
        if new_state != old_state {
            self.generate_moves();
            self.sort_moves();
        }

        self.moves.next()
    }
}

#[cfg(test)]
mod tests {
    //extern crate test;
    //use self::test::Bencher;
    use common::*;
    use moves::Move;
    use fen::FEN;
    use game::Game;
    use super::*;

    #[test]
    fn test_generate_moves() {
        println!("test_generate_moves()");
        let mut game = Game::new();

        game.load_fen(DEFAULT_FEN);
        game.moves.clear();
        game.generate_moves();
        println!("{}", game.to_string());
        assert_eq!(game.moves.len(), 20);

        // Pawn right capture
        let fen = "8/8/4k3/4p3/3P4/3K4/8/8 b - -";
        game.load_fen(fen);
        game.moves.clear();
        game.generate_moves();
        println!("{}", game.to_string());
        assert_eq!(game.moves.len(), 9);

        let fen = "8/8/4k3/4p3/3P4/3K4/8/8 w - -";
        game.load_fen(fen);
        game.moves.clear();
        game.generate_moves();
        println!("{}", game.to_string());
        assert_eq!(game.moves.len(), 9);

        // Pawn left capture
        let fen = "8/8/2p5/2p1P3/1p1P4/3P4/8/8 w - -";
        game.load_fen(fen);
        game.moves.clear();
        game.generate_moves();
        println!("{}", game.to_string());
        assert_eq!(game.moves.len(), 3);

        let fen = "8/8/2p5/2p1P3/1p1P4/3P4/8/8 b - -";
        game.load_fen(fen);
        game.moves.clear();
        game.generate_moves();
        println!("{}", game.to_string());
        assert_eq!(game.moves.len(), 3);

        // Bishop
        let fen = "8/8/8/8/3B4/8/8/8 w - -";
        game.load_fen(fen);
        game.moves.clear();
        game.generate_moves();
        println!("{}", game.to_string());
        assert_eq!(game.moves.len(), 13);

        // Rook
        let fen = "8/8/8/8/1r1R4/8/8/8 w - -";
        game.load_fen(fen);
        game.moves.clear();
        game.generate_moves();
        println!("{}", game.to_string());
        assert_eq!(game.moves.len(), 13);
    }

    #[test]
    fn test_make_move() {
        let fens = [
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            "rnbqkbnr/pppppppp/8/8/8/4P3/PPPP1PPP/RNBQKBNR b KQkq - 0 1"
        ];
        let m = Move::new(E2, E3, QUIET_MOVE);

        let mut game = Game::from_fen(fens[0]);
        assert_eq!(game.to_fen().as_str(), fens[0]);

        game.make_move(m);
        assert_eq!(game.to_fen().as_str(), fens[1]);
    }

    #[test]
    fn test_undo_move() {
        let fens = [
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            "rnbqkbnr/pppppppp/8/8/8/4P3/PPPP1PPP/RNBQKBNR b KQkq - 0 1"
        ];
        let m = Move::new(E2, E3, QUIET_MOVE);

        let mut game = Game::from_fen(fens[0]);

        game.make_move(m);
        assert_eq!(game.to_fen().as_str(), fens[1]);

        game.undo_move(m);
        assert_eq!(game.to_fen().as_str(), fens[0]);
    }

    #[test]
    fn test_capture() {
        let fens = [
            "r1bqkbnr/1ppp1ppp/p1n5/1B2p3/4P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 0 1",
            "r1bqkbnr/1ppp1ppp/p1B5/4p3/4P3/5N2/PPPP1PPP/RNBQK2R b KQkq - 0 1"
        ];
        let m = Move::new(B5, C6, CAPTURE);

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

        assert_eq!(game.mvv_lva(Move::new(B2, C3, CAPTURE)), 15); // PxN
        assert_eq!(game.mvv_lva(Move::new(B3, C3, CAPTURE)), 11); // QxN
        assert_eq!(game.mvv_lva(Move::new(D2, C3, CAPTURE)), 10); // KxN
        assert_eq!(game.mvv_lva(Move::new(B3, C2, CAPTURE)),  3); // QxP
        assert_eq!(game.mvv_lva(Move::new(D2, C2, CAPTURE)),  2); // KxP

        game.generate_moves();
        game.sort_moves();

        assert_eq!(game.moves.next(), Some(Move::new(B2, C3, CAPTURE)));
        assert_eq!(game.moves.next(), Some(Move::new(B3, C3, CAPTURE)));
        assert_eq!(game.moves.next(), Some(Move::new(D2, C3, CAPTURE)));
        assert_eq!(game.moves.next(), Some(Move::new(B3, C2, CAPTURE)));
        assert_eq!(game.moves.next(), Some(Move::new(D2, C2, CAPTURE)));

        assert!(!game.moves.next().unwrap().is_capture());
    }

    /*
    #[bench]
    fn bench_perft(b: &mut Bencher) {
        let mut game = Game::from_fen(fens[0]);

        b.iter(|| {
            game.perft(1);
        })
    }

    #[bench]
    fn bench_generate_moves(b: &mut Bencher) {
        let mut game = Game::from_fen(fens[0]);

        b.iter(|| {
            game.moves.clear();
            game.generate_moves();
        })
    }

    #[bench]
    fn bench_make_undo_move(b: &mut Bencher) {
        let mut game: Game = FEN::from_fen(DEFAULT_FEN);
        let m = Move::new(E2, E3, QUIET_MOVE);

        b.iter(|| {
            game.make_move(m);
            game.undo_move(m);
        })
    }
    */
}
