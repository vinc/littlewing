use std;

use littlewing::common::*;
use littlewing::attack::Attack;
use littlewing::bitboard::BitwiseOperations;
use littlewing::piece::PieceChar;
use littlewing::piece::PieceAttr;
use littlewing::moves::Move;
use littlewing::moves::Moves;
use littlewing::position::Position;
use littlewing::position::Stack;

pub struct Game {
    pub bitboards: [Bitboard; 14],
    pub board: [Piece; 64],
    pub moves: Moves,
    pub positions: Vec<Position>
}

impl Game {
    fn new() -> Game {
        Game {
            bitboards: [0; 14],
            board: [EMPTY; 64],
            moves: Moves::new(),
            positions: Vec::with_capacity(512)
        }
    }

    pub fn from_fen(fen: &str) -> Game {
        let mut game = Game::new();
        let mut fields = fen.words();

        let mut sq = A8;
        for c in fields.next().unwrap().chars() {
            sq += if c == '/' {
                2 * DOWN
            } else if '1' <= c && c <= '8' {
                c.to_digit(10).unwrap()
            } else {
                let p = PieceChar::from_char(c);
                game.board[sq] = p;
                game.bitboards[p].set(sq);
                game.bitboards[p & 1].set(sq); // TODO: p.color()

                1
            };
        }

        let mut position = Position::new();

        position.side = match fields.next().unwrap() {
            "w" => WHITE,
            "b" => BLACK,
            _   => BLACK // FIXME
        };

        for c in fields.next().unwrap().chars() {
            match c {
                'K' => position.castling_rights[WHITE][KING >> 3] = true,
                'Q' => position.castling_rights[WHITE][QUEEN >> 3] = true,
                'k' => position.castling_rights[BLACK][KING >> 3] = true,
                'q' => position.castling_rights[BLACK][QUEEN >> 3] = true,
                _   => break
            }
        }

        game.positions.push(position);
        game
    }

    pub fn to_fen(&self) -> String {
        let mut fen = String::new();
        let mut n = 0u;
        let mut sq = A8;
        loop {
            let p = self.board[sq];

            if p == EMPTY {
                n += 1;
            } else {
                if n > 0 {
                    let c = std::char::from_digit(n, 10).unwrap();
                    fen.push(c);
                    n = 0;
                }
                fen.push(p.to_char());
            }

            if sq == H1 {
                break;
            }

            if sq & H1 == H1 { // TODO: is_file_h!(sq)
                if n > 0 { // TODO: DRY
                    let c = std::char::from_digit(n, 10).unwrap();
                    fen.push(c);
                    n = 0;
                }
                fen.push('/');
                sq += 2 * DOWN;
            }

            sq += RIGHT;
        }

        fen.push(' ');
        if self.positions.top().side == WHITE {
            fen.push('w');
        } else {
            fen.push('b');
        }

        fen.push(' ');
        let castling_rights = self.positions.top().castling_rights;
        let mut castles = String::new();
        if castling_rights[WHITE][KING >> 3] {
            castles.push('K');
        }
        if castling_rights[WHITE][QUEEN >> 3] {
            castles.push('Q');
        }
        if castling_rights[BLACK][KING >> 3] {
            castles.push('k');
        }
        if castling_rights[BLACK][QUEEN >> 3] {
            castles.push('q');
        }
        if castles.len() == 0 {
            castles.push('-');
        }
        fen.push_str(castles.as_slice());

        fen.push_str(" - 0 1"); // TODO

        fen
    }

    fn ply(&self) -> uint {
        self.positions.len() - 1
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

    pub fn perft(&mut self, depth: uint) -> u64 {
        if depth == 0 {
            1
        } else {
            self.generate_moves();
            let n = self.moves.len();
            let mut r = 0;
            for i in range(0u, n) {
                let m = self.moves.get(i);
                self.make_move(m);
                if !self.is_check() {
                    r += self.perft(depth - 1);
                }
                self.undo_move(m);
            }
            r
        }
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
    use littlewing::position::Stack;
    use super::Game;

    #[test]
    fn test_from_fen() {
        let mut game = Game::from_fen(DEFAULT_FEN);
        assert_eq!(game.board[E2], WHITE_PAWN);
    }

    #[test]
    fn test_to_fen() {
        let fens = [
            DEFAULT_FEN,
            "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1",
            "rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 0 1"
        ];
        for &fen in fens.iter() {
            let game = Game::from_fen(fen);
            assert_eq!(game.to_fen().as_slice(), fen);
        }
    }

    #[test]
    fn test_perft() {
        // Initial position
        let mut game = Game::from_fen(DEFAULT_FEN);
        assert_eq!(game.perft(1), 20);
        assert_eq!(game.perft(2), 400);
        assert_eq!(game.perft(3), 8902);
        assert_eq!(game.perft(4), 197281);

        let fen = "7k/8/8/p7/1P6/8/8/7K b - - 0 1";
        let mut game = Game::from_fen(fen);
        assert_eq!(game.perft(1), 5);

        let fen = "k6K/8/8/b6b/8/8/8/8 b - - 0 1";
        let mut game = Game::from_fen(fen);
        assert_eq!(game.perft(1), 17);

        let fen = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -";
        let mut game = Game::from_fen(fen);
        assert_eq!(game.perft(1), 14);
        assert_eq!(game.perft(2), 191);
        assert_eq!(game.perft(3), 2812);

        let fen = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1";
        let mut game = Game::from_fen(fen);
        assert_eq!(game.perft(1), 6);
        assert_eq!(game.perft(2), 264);
        assert_eq!(game.perft(3), 9467);

        let fen = "r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1";
        let mut game = Game::from_fen(fen);
        assert_eq!(game.perft(1), 6);
        assert_eq!(game.perft(2), 264);
        assert_eq!(game.perft(3), 9467);

        // Kiwipete position
        let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -";
        let mut game = Game::from_fen(fen);
        assert_eq!(game.perft(1), 48);
        assert_eq!(game.perft(2), 2039);
        assert_eq!(game.perft(3), 97862);

        let fen = "rnbqkb1r/pp1p1ppp/2p5/4P3/2B5/8/PPP1NnPP/RNBQK2R w KQkq - 0 6";
        let mut game = Game::from_fen(fen);
        assert_eq!(game.perft(1), 42);
        assert_eq!(game.perft(2), 1352);
        assert_eq!(game.perft(3), 53392);
    }

    #[test]
    fn test_generate_moves() {
        println!("test_generate_moves()");
        let mut game = Game::from_fen(DEFAULT_FEN);
        game.generate_moves();
        println!("{}", game.to_string());
        assert_eq!(game.moves.len(), 20);

        // Pawn right capture
        let fen = "8/8/4k3/4p3/3P4/3K4/8/8 b - -";
        let mut game = Game::from_fen(fen);
        game.generate_moves();
        println!("{}", game.to_string());
        assert_eq!(game.moves.len(), 9);

        let fen = "8/8/4k3/4p3/3P4/3K4/8/8 w - -";
        let mut game = Game::from_fen(fen);
        game.generate_moves();
        println!("{}", game.to_string());
        assert_eq!(game.moves.len(), 9);

        // Pawn left capture
        let fen = "8/8/2p5/2p1P3/1p1P4/3P4/8/8 w - -";
        let mut game = Game::from_fen(fen);
        game.generate_moves();
        println!("{}", game.to_string());
        assert_eq!(game.moves.len(), 3);

        let fen = "8/8/2p5/2p1P3/1p1P4/3P4/8/8 b - -";
        let mut game = Game::from_fen(fen);
        game.generate_moves();
        println!("{}", game.to_string());
        assert_eq!(game.moves.len(), 3);

        // Bishop
        let fen = "8/8/8/8/3B4/8/8/8 w - -";
        let mut game = Game::from_fen(fen);
        game.generate_moves();
        println!("{}", game.to_string());
        assert_eq!(game.moves.len(), 13);

        // Rook
        let fen = "8/8/8/8/1r1R4/8/8/8 w - -";
        let mut game = Game::from_fen(fen);
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

        let mut game = Game::from_fen(fens[0]);

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

        let mut game = Game::from_fen(fens[0]);
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
        let mut game = Game::from_fen(DEFAULT_FEN);

        b.iter(|| {
            game.perft(1);
        })
    }

    #[bench]
    fn bench_generate_moves(b: &mut Bencher) {
        let mut game = Game::from_fen(DEFAULT_FEN);

        b.iter(|| {
            game.generate_moves();
        })
    }

    #[bench]
    fn bench_make_move(b: &mut Bencher) {
        let mut game = Game::from_fen(DEFAULT_FEN);
        let m = Move::new(E2, E3, QUIET_MOVE);

        b.iter(|| {
            game.make_move(m);
        })
    }
}
