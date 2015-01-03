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
    pub bitboards: [Bitboard, ..14],
    pub board: [Piece, ..64],
    pub moves: Moves,
    pub positions: Vec<Position>
}

impl Game {
    fn new() -> Game {
        Game {
            bitboards: [0, ..14],
            board: [EMPTY, ..64],
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
                    String::from_chars(['|', ' ', c, ' '].as_slice())
                })
                .fold(String::new(), |r, s| {
                    r + s.as_slice()
                }) + "|\n" + sep.as_slice()
        }).fold(String::new(), |r, s| r + s.as_slice()).as_slice()
    }

    pub fn perft(&mut self, depth: uint) -> u64 {
        //println!("perft({})", depth);
        if depth == 0 {
            1
        } else {
            self.generate_moves();
            let n = self.moves.len();
            let mut r = 0;
            for i in range(0u, n) {
                let m = self.moves.get(i);
                //println!("{}: {}", i, m.to_string(&self.board));
                self.make_move(m);
                //println!("{}", self.to_string());
                if !self.is_check() {
                    r += self.perft(depth - 1);
                }
                self.undo_move(m);
            }
            r
        }
    }

    pub fn make_move(&mut self, m: Move) {
        let &mut position = self.positions.top();

        let piece = self.board[m.from];
        let capture = self.board[m.to]; // TODO: En passant

        self.bitboards[piece].toggle(m.from);
        self.board[m.from] = EMPTY;

        if m.is_promotion() {
            let promoted_piece = position.side | m.promotion_kind();
            self.board[m.to] = promoted_piece;
            self.bitboards[promoted_piece].toggle(m.to);
        } else {
            self.board[m.to] = piece;
            self.bitboards[piece].toggle(m.to);
        }

        self.bitboards[position.side].toggle(m.from);
        self.bitboards[position.side].toggle(m.to);

        if capture != EMPTY {
            self.bitboards[capture].toggle(m.to);
            self.bitboards[position.side ^ 1].toggle(m.to);
        }

        // FIXME
        position.side ^= 1; // TODO: Define self.side.toggle(0)
        position.capture = capture;

        self.positions.push(position);
        self.moves.inc();
    }

    pub fn undo_move(&mut self, m: Move) {
        let piece = self.board[m.to];
        let capture = self.positions.top().capture;

        self.positions.pop();
        self.moves.dec();

        let &position = self.positions.top();

        if m.is_promotion() {
            let pawn = position.side | PAWN;
            self.board[m.from] = pawn;
            self.bitboards[pawn].toggle(m.from);
            self.bitboards[pawn].toggle(m.to);
        } else {
            self.board[m.from] = piece;
            self.bitboards[piece].toggle(m.from);
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
        let side = self.positions.top().side;

        self.moves.clear();
        self.moves.add_pawns_moves(bitboards, side);
        self.moves.add_knights_moves(bitboards, side);
        self.moves.add_king_moves(bitboards, side);
        self.moves.add_bishops_moves(bitboards, side);
        self.moves.add_rooks_moves(bitboards, side);
        self.moves.add_queens_moves(bitboards, side);
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
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w";
        let game = Game::from_fen(fen);
        assert_eq!(game.board[E2], WHITE_PAWN);
    }

    #[test]
    fn test_to_fen() {
        let fens = [
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w",
            "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b",
            "rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R w"
        ];
        for &fen in fens.iter() {
            let game = Game::from_fen(fen);
            assert_eq!(game.to_fen().as_slice(), fen);
        }
    }

    #[test]
    fn test_perft() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w";
        let mut game = Game::from_fen(fen);
        assert_eq!(game.perft(1), 20);
        assert_eq!(game.perft(2), 400);
        assert_eq!(game.perft(3), 8902);
        assert_eq!(game.perft(4), 197281);

        let fen = "7k/8/8/p7/1P6/8/8/7K b - - 0 1";
        let mut game = Game::from_fen(fen);
        assert_eq!(game.perft(1), 5);

        let fen = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w";
        let mut game = Game::from_fen(fen);
        assert_eq!(game.perft(1), 14);
        //assert_eq!(game.perft(2), 191);

        let fen = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w";
        let mut game = Game::from_fen(fen);
        assert_eq!(game.perft(1), 6);
        //assert_eq!(game.perft(2), 264);

        let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w";
        let mut game = Game::from_fen(fen);
        //assert_eq!(game.perft(1), 48);
        //assert_eq!(game.perft(2), 2039);
        //assert_eq!(game.perft(3), 97862);
    }

    #[test]
    fn test_generate_moves() {
        println!("test_generate_moves()");
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w";
        let mut game = Game::from_fen(fen);
        game.generate_moves();
        println!("{}", game.to_string());
        assert_eq!(game.moves.len(), 20);

        // Pawn right capture
        let fen = "8/8/4k3/4p3/3P4/3K4/8/8 b";
        let mut game = Game::from_fen(fen);
        game.generate_moves();
        println!("{}", game.to_string());
        assert_eq!(game.moves.len(), 9);

        let fen = "8/8/4k3/4p3/3P4/3K4/8/8 w";
        let mut game = Game::from_fen(fen);
        game.generate_moves();
        println!("{}", game.to_string());
        assert_eq!(game.moves.len(), 9);

        // Pawn left capture
        let fen = "8/8/2p5/2p1P3/1p1P4/3P4/8/8 w";
        let mut game = Game::from_fen(fen);
        game.generate_moves();
        println!("{}", game.to_string());
        assert_eq!(game.moves.len(), 3);

        let fen = "8/8/2p5/2p1P3/1p1P4/3P4/8/8 b";
        let mut game = Game::from_fen(fen);
        game.generate_moves();
        println!("{}", game.to_string());
        assert_eq!(game.moves.len(), 3);

        // Bishop
        let fen = "8/8/8/8/3B4/8/8/8 w";
        let mut game = Game::from_fen(fen);
        game.generate_moves();
        println!("{}", game.to_string());
        assert_eq!(game.moves.len(), 13);

        // Rook
        let fen = "8/8/8/8/1r1R4/8/8/8 w";
        let mut game = Game::from_fen(fen);
        game.generate_moves();
        println!("{}", game.to_string());
        assert_eq!(game.moves.len(), 13);
    }

    #[test]
    fn test_make_move() {
        let fens = [
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w",
            "rnbqkbnr/pppppppp/8/8/8/4P3/PPPP1PPP/RNBQKBNR b"
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
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w",
            "rnbqkbnr/pppppppp/8/8/8/4P3/PPPP1PPP/RNBQKBNR b"
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
            "r1bqkbnr/1ppp1ppp/p1n5/1B2p3/4P3/5N2/PPPP1PPP/RNBQK2R w",
            "r1bqkbnr/1ppp1ppp/p1B5/4p3/4P3/5N2/PPPP1PPP/RNBQK2R b"
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
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w";
        let mut game = Game::from_fen(fen);

        b.iter(|| {
            game.perft(1);
        })
    }

    #[bench]
    fn bench_generate_moves(b: &mut Bencher) {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w";
        let mut game = Game::from_fen(fen);

        b.iter(|| {
            game.generate_moves();
        })
    }

    #[bench]
    fn bench_make_move(b: &mut Bencher) {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w";
        let mut game = Game::from_fen(fen);
        let m = Move::new(E2, E3, QUIET_MOVE);

        b.iter(|| {
            game.make_move(m);
        })
    }
}
