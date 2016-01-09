use littlewing::common::*;
use littlewing::attack::Attack;
use littlewing::attack::attacks;
use littlewing::bitboard::BitboardExt;
use littlewing::clock::Clock;
use littlewing::moves::Move;
use littlewing::moves::Moves;
use littlewing::piece::PieceAttr;
use littlewing::piece::PieceChar;
use littlewing::position::Positions;
use littlewing::square::SquareString;
use littlewing::zobrist::Zobrist;

pub struct Game {
    pub is_verbose: bool,
    pub nodes_count: u64,
    pub clock: Clock,
    pub bitboards: [Bitboard; 14],
    pub board: [Piece; 64],
    pub moves: Moves,
    pub positions: Positions,
    pub zobrist: Zobrist,
    pub history: Vec<Move>
}

impl Game {
    pub fn new() -> Game {
        Game {
            is_verbose: false,
            nodes_count: 0,
            clock: Clock::new(40, 5 * 60),
            bitboards: [0; 14],
            board: [EMPTY; 64],
            moves: Moves::new(),
            positions: Positions::new(),
            zobrist: Zobrist::new(),
            history: Vec::new()
        }
    }

    pub fn clear(&mut self) {
        self.bitboards = [0; 14];
        self.board = [EMPTY; 64];
        self.moves.clear_all();
        self.positions.clear();
        self.history.clear();
    }

    pub fn to_string(&self) -> String {
        // FIXME: Testing `map` and `fold` for the lulz

        let sep = (0..8)
            .map(|_| "+---")
            .fold(String::new(), |r, s| r + s) + "+\n";

        String::new() + sep.as_str() + (0..8).map(|i| {
            (0..8)
                .map(|j| {
                    let c = (self.board[8 * (7 - i) + j as usize]).to_char();
                    String::from("| ") + c.to_string().as_str() + " "
                })
                .fold(String::new(), |r, s| {
                    r + s.as_str()
                }) + "|\n" + sep.as_str()
        }).fold(String::new(), |r, s| r + s.as_str()).as_str()
    }

    pub fn make_move(&mut self, m: Move) {
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

    pub fn undo_move(&mut self, m: Move) {
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
            self.bitboards[pawn as usize].toggle(m.to());
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

    pub fn generate_moves(&mut self) {
        let &position = self.positions.top();
        let side = position.side;
        let ep = position.en_passant;

        self.moves.clear();
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

    // This function assumes that the move has not been played yet
    pub fn move_to_san(&mut self, m: Move) -> String {
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
                let rank = m.from().to_coord().as_str().char_at(0);
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
}

#[cfg(test)]
mod tests {
    extern crate test;
    use self::test::Bencher;
    use littlewing::common::*;
    use littlewing::moves::Move;
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

        let mut game: Game = FEN::from_fen(fens[0]);

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

        let mut game: Game = FEN::from_fen(fens[0]);
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
