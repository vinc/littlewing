use littlewing::common::*;
use littlewing::attack::Attack;
use littlewing::bitboard::BitwiseOperations;
use littlewing::moves::Move;
use littlewing::moves::Moves;
use littlewing::piece::PieceAttr;
use littlewing::piece::PieceChar;
use littlewing::position::Positions;

pub struct Game {
    pub bitboards: [Bitboard; 14],
    pub board: [Piece; 64],
    pub moves: Moves,
    pub positions: Positions
}

impl Game {
    pub fn new() -> Game {
        Game {
            bitboards: [0; 14],
            board: [EMPTY; 64],
            moves: Moves::new(),
            positions: Positions::new()
        }
    }

    pub fn clear(&mut self) {
        self.bitboards = [0; 14];
        self.board = [EMPTY; 64];
        self.moves.clear();
        self.positions.clear();
    }

    pub fn to_string(&self) -> String {
        // FIXME: Testing `map` and `fold` for the lulz

        let sep = range(0u, 8)
            .map(|_| "+---")
            .fold(String::new(), |r, s| r + s) + "+\n";

        String::new() + sep.as_slice() + range(0u, 8).map(|i| {
            range(0u, 8)
                .map(|j| {
                    let c = (self.board[8 * (7 - i) + j]).to_char();
                    String::from_str("| ") + c.to_string().as_slice() + " "
                })
                .fold(String::new(), |r, s| {
                    r + s.as_slice()
                }) + "|\n" + sep.as_slice()
        }).fold(String::new(), |r, s| r + s.as_slice()).as_slice()
    }

    pub fn make_move(&mut self, m: Move) {
        let &position = self.positions.top();
        let mut new_position = position;
        let side = position.side;

        let piece = self.board[m.from];
        let capture = self.board[m.to]; // TODO: En passant

        self.bitboards[piece].toggle(m.from);
        self.board[m.from] = EMPTY;

        // Update castling rights
        if piece.kind() == KING {
            new_position.castling_rights[side][KING >> 3] = false;
            new_position.castling_rights[side][QUEEN >> 3] = false;
        } else if piece.kind() == ROOK {
            if m.from == H1 ^ 56 * side {
                new_position.castling_rights[side][KING >> 3] = false;
            }
            if m.from == A1 ^ 56 * side {
                new_position.castling_rights[side][QUEEN >> 3] = false;
            }
        }
        if capture.kind() == ROOK {
            if m.to == H1 ^ 56 * (side ^ 1) {
                new_position.castling_rights[side ^ 1][KING >> 3] = false;
            }
            if m.to == A1 ^ 56 * (side ^ 1) {
                new_position.castling_rights[side ^ 1][QUEEN >> 3] = false;
            }
        }

        if m.is_castle() {
            let rook = side | ROOK;

            let (rook_from, rook_to) = if m.castle_kind() == KING {
                (H1 ^ 56 * side, F1 ^ 56 * side)
            } else {
                (A1 ^ 56 * side, D1 ^ 56 * side)
            };

            self.board[rook_from] = EMPTY;
            self.board[rook_to] = rook;
            self.bitboards[rook].toggle(rook_from);
            self.bitboards[rook].toggle(rook_to);
            self.bitboards[side].toggle(rook_from);
            self.bitboards[side].toggle(rook_to);
        }

        if m.is_promotion() {
            let promoted_piece = side | m.promotion_kind();
            self.board[m.to] = promoted_piece;
            self.bitboards[promoted_piece].toggle(m.to);
        } else {
            self.board[m.to] = piece;
            self.bitboards[piece].toggle(m.to);
        }

        new_position.en_passant = if m.kind == DOUBLE_PAWN_PUSH {
            ((m.from ^ (56 * side)) + UP) ^ (56 * side)
        } else {
            OUT
        };

        if new_position.en_passant != OUT {
        }

        self.bitboards[side].toggle(m.from);
        self.bitboards[side].toggle(m.to);

        if capture != EMPTY {
            self.bitboards[capture].toggle(m.to);
            self.bitboards[side ^ 1].toggle(m.to);
        }
        if m.kind == EN_PASSANT {
            let square = ((m.to ^ (56 * side)) + DOWN) ^ (56 * side);
            self.board[square] = EMPTY;
            self.bitboards[side ^ 1 | PAWN].toggle(square);
            self.bitboards[side ^ 1].toggle(square);
        }

        // FIXME
        new_position.side ^= 1; // TODO: Define self.side.toggle(0)
        new_position.capture = capture;

        self.positions.push(new_position);
        self.moves.inc();
    }

    pub fn undo_move(&mut self, m: Move) {
        let piece = self.board[m.to];
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

            self.board[rook_from] = rook;
            self.board[rook_to] = EMPTY;
            self.bitboards[rook].toggle(rook_from);
            self.bitboards[rook].toggle(rook_to);
            self.bitboards[side].toggle(rook_from);
            self.bitboards[side].toggle(rook_to);
        }

        if m.is_promotion() {
            let pawn = position.side | PAWN;
            self.board[m.from] = pawn;
            self.bitboards[pawn].toggle(m.from);
            self.bitboards[pawn].toggle(m.to);
        } else {
            self.board[m.from] = piece;
            self.bitboards[piece].toggle(m.from);
        }

        if m.kind == EN_PASSANT {
            let square = ((m.to ^ (56 * side)) + DOWN) ^ (56 * side);
            self.board[square] = side ^ 1 | PAWN;
            self.bitboards[side ^ 1 | PAWN].toggle(square);
            self.bitboards[side ^ 1].toggle(square);
        }

        self.board[m.to] = capture;
        self.bitboards[piece].toggle(m.to);

        self.bitboards[position.side].toggle(m.from);
        self.bitboards[position.side].toggle(m.to);

        if capture != EMPTY {
            self.bitboards[capture].toggle(m.to);
            self.bitboards[position.side ^ 1].toggle(m.to);
        }
    }

    pub fn generate_moves(&mut self) {
        let bitboards = self.bitboards.as_slice();
        let &position = self.positions.top();
        let side = position.side;
        let ep = position.en_passant;

        self.moves.clear();
        self.moves.add_pawns_moves(bitboards, side, ep);
        self.moves.add_knights_moves(bitboards, side);
        self.moves.add_king_moves(bitboards, side);
        self.moves.add_bishops_moves(bitboards, side);
        self.moves.add_rooks_moves(bitboards, side);
        self.moves.add_queens_moves(bitboards, side);


        let occupied = bitboards[WHITE] | bitboards[BLACK];

        let mask = CASTLING_MASKS[side][KING >> 3];
        let can_castle =
            !occupied & mask == mask &&
            position.castling_rights[side][KING >> 3] &&
            !self.is_attacked(E1 ^ 56 * side, side) &&
            !self.is_attacked(F1 ^ 56 * side, side) &&
            !self.is_attacked(G1 ^ 56 * side, side); // TODO: Duplicate with is_check() ?
        if can_castle {
            self.moves.add_king_castle(side);
        }

        let mask = CASTLING_MASKS[side][QUEEN >> 3];
        let can_castle =
            !occupied & mask == mask &&
            position.castling_rights[side][QUEEN >> 3] &&
            !self.is_attacked(E1 ^ 56 * side, side) &&
            !self.is_attacked(D1 ^ 56 * side, side) &&
            !self.is_attacked(C1 ^ 56 * side, side);
        if can_castle {
            self.moves.add_queen_castle(side);
        }

    }
}

#[cfg(test)]
mod tests {
    extern crate test;
    use self::test::Bencher;
    use littlewing::common::*;
    use littlewing::moves::Move;
    use littlewing::position::Positions;
    use littlewing::fen::FEN;
    use littlewing::game::Game;
    use littlewing::search::Search;

    #[test]
    fn test_generate_moves() {
        println!("test_generate_moves()");
        let mut game = Game::new();

        game.load_fen(DEFAULT_FEN);
        game.generate_moves();
        println!("{}", game.to_string());
        assert_eq!(game.moves.len(), 20);

        // Pawn right capture
        let fen = "8/8/4k3/4p3/3P4/3K4/8/8 b - -";
        game.load_fen(fen);
        game.generate_moves();
        println!("{}", game.to_string());
        assert_eq!(game.moves.len(), 9);

        let fen = "8/8/4k3/4p3/3P4/3K4/8/8 w - -";
        game.load_fen(fen);
        game.generate_moves();
        println!("{}", game.to_string());
        assert_eq!(game.moves.len(), 9);

        // Pawn left capture
        let fen = "8/8/2p5/2p1P3/1p1P4/3P4/8/8 w - -";
        game.load_fen(fen);
        game.generate_moves();
        println!("{}", game.to_string());
        assert_eq!(game.moves.len(), 3);

        let fen = "8/8/2p5/2p1P3/1p1P4/3P4/8/8 b - -";
        game.load_fen(fen);
        game.generate_moves();
        println!("{}", game.to_string());
        assert_eq!(game.moves.len(), 3);

        // Bishop
        let fen = "8/8/8/8/3B4/8/8/8 w - -";
        game.load_fen(fen);
        game.generate_moves();
        println!("{}", game.to_string());
        assert_eq!(game.moves.len(), 13);

        // Rook
        let fen = "8/8/8/8/1r1R4/8/8/8 w - -";
        game.load_fen(fen);
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

        let mut game: Game = FEN::from_fen(fens[0]);
        assert_eq!(game.to_fen().as_slice(), fens[0]);

        game.make_move(m);
        assert_eq!(game.to_fen().as_slice(), fens[1]);
    }

    #[test]
    fn test_undo_move() {
        let fens = [
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            "rnbqkbnr/pppppppp/8/8/8/4P3/PPPP1PPP/RNBQKBNR b KQkq - 0 1"
        ];
        let m = Move::new(E2, E3, QUIET_MOVE);

        let mut game: Game = FEN::from_fen(fens[0]);

        game.make_move(m);
        assert_eq!(game.to_fen().as_slice(), fens[1]);

        game.undo_move(m);
        assert_eq!(game.to_fen().as_slice(), fens[0]);
    }

    #[test]
    fn test_capture() {
        let fens = [
            "r1bqkbnr/1ppp1ppp/p1n5/1B2p3/4P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 0 1",
            "r1bqkbnr/1ppp1ppp/p1B5/4p3/4P3/5N2/PPPP1PPP/RNBQK2R b KQkq - 0 1"
        ];
        let m = Move::new(B5, C6, CAPTURE);

        let mut game: Game = FEN::from_fen(fens[0]);
        assert_eq!(game.to_fen().as_slice(), fens[0]);
        assert_eq!(game.positions.len(), 1);
        assert_eq!(game.positions.top().capture, EMPTY);
        assert_eq!(game.positions[0].capture, EMPTY);
        assert_eq!(game.positions[0].side, WHITE);

        game.make_move(m);
        assert_eq!(game.to_fen().as_slice(), fens[1]);
        assert_eq!(game.positions.len(), 2);
        assert_eq!(game.positions.top().capture, BLACK_KNIGHT);
        assert_eq!(game.positions[0].capture, EMPTY);
        assert_eq!(game.positions[0].side, WHITE);
        assert_eq!(game.positions[1].capture, BLACK_KNIGHT);
        assert_eq!(game.positions[1].side, BLACK);

        game.undo_move(m);
        assert_eq!(game.to_fen().as_slice(), fens[0]);
        assert_eq!(game.positions.len(), 1);
        assert_eq!(game.positions.top().capture, EMPTY);
        assert_eq!(game.positions[0].capture, EMPTY);
        assert_eq!(game.positions[0].side, WHITE);
    }

    #[bench]
    fn bench_perft(b: &mut Bencher) {
        let mut game: Game = FEN::from_fen(DEFAULT_FEN);

        b.iter(|| {
            game.perft(1);
        })
    }

    #[bench]
    fn bench_generate_moves(b: &mut Bencher) {
        let mut game: Game = FEN::from_fen(DEFAULT_FEN);

        b.iter(|| {
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
}
