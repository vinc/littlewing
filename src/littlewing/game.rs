use std;

use littlewing::common::*;
use littlewing::bitboard::BitwiseOperations;
use littlewing::fen::FEN;
use littlewing::moves::Move;
use littlewing::moves::Moves;
use littlewing::moves::MovesOperations;

#[deriving(Copy)]
pub struct Game {
    board: [Piece, ..64],
    bitboards: [Bitboard, ..14],
    side: Color
}

impl Game {
    pub fn new() -> Game {
        Game {
            board: [EMPTY, ..64],
            bitboards: [0, ..14],
            side: WHITE
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
                let p = FEN::decode_piece(c);
                game.board[sq] = p;
                game.bitboards[p].set(sq);

                1
            };
        }
        game.bitboards[WHITE] = 0;
        for p in range(WHITE_PAWN, WHITE_KING + 1) {
            game.bitboards[WHITE] |= game.bitboards[p];
        }
        for p in range(BLACK_PAWN, BLACK_KING + 1) {
            game.bitboards[BLACK] |= game.bitboards[p];
        }
        game.side = match fields.next().unwrap() {
            "w" => WHITE,
            "b" => BLACK,
            _   => BLACK // FIXME
        };
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
                fen.push(FEN::encode_piece(p));
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
        if self.side == WHITE {
            fen.push('w');
        } else {
            fen.push('b');
        }

        fen
    }

    fn to_string(&self) -> String {
        // FIXME: Testing `map` and `fold` for the lulz

        let sep = range(0u, 8)
            .map(|_| "+---")
            .fold(String::new(), |r, s| r + s) + "+\n";

        String::new() + sep.as_slice() + range(0u, 8).map(|i| {
            let rank = range(0u, 8)
                .map(|j| {
                    let p = FEN::encode_piece(self.board[8 * (7 - i) + j]);
                    String::from_chars(['|', ' ', p, ' '].as_slice())
                })
                .fold(String::new(), |r, s| r + s.as_slice()) + "|\n";
            rank + sep.as_slice()
        }).fold(String::new(), |r, s| r + s.as_slice()).as_slice()
    }

    pub fn perft(&mut self, depth: uint) -> uint {
        let mut n = 0;

        if depth == 0 {
            return n
        }

        for &m in self.generate_moves().iter() {
            self.play_move(m);
            n += 1 + self.perft(depth - 1);
            self.undo_move(m);
        }

        n
    }

    pub fn play_move(&mut self, m: Move) {
        let piece = self.board[m.from];

        self.board[m.from] = EMPTY;
        self.board[m.to] = piece;

        self.bitboards[piece].toggle(m.from);
        self.bitboards[piece].toggle(m.to);

        self.side ^= 1; // TODO: Define self.side.toggle(0)
    }

    pub fn undo_move(&mut self, m: Move) {
        let piece = self.board[m.to];

        self.board[m.from] = piece;
        self.board[m.to] = EMPTY;

        self.bitboards[piece].toggle(m.from);
        self.bitboards[piece].toggle(m.to);

        self.side ^= 1; // TODO: Define self.side.toggle(0)
    }

    pub fn generate_moves(&self) -> Moves {
        let mut moves = Vec::new();

        // FIXME: This is done to avoid writing self everywhere
        let bitboards = &self.bitboards;
        let side = self.side;

        let dirs = [UP, DOWN];
        let ranks = [RANK_3, RANK_6];
        let occupied = bitboards[WHITE] | bitboards[BLACK];

        let pushes = bitboards[side | PAWN].shift(dirs[side]) & !occupied;
        moves.add_moves(pushes, dirs[side], QUIET_MOVE);

        let double_pushes = (pushes & ranks[side]).shift(dirs[side]) & !occupied;
        moves.add_moves(double_pushes, 2 * dirs[side], DOUBLE_PAWN_PUSH);

        /*
        let left_attacks = (bitboards[WHITE_PAWN] << 7) & bitboards[BLACK];
        moves.add_moves(left_attacks, UP + LEFT, CAPTURE);

        let right_attacks = (bitboards[WHITE_PAWN] << 9) & bitboards[BLACK];
        moves.add_moves(right_attacks, UP + RIGHT, CAPTURE);
        */

        moves
    }
}

#[cfg(test)]
mod tests {
    use littlewing::common::*;
    use littlewing::moves::Move;
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
        assert_eq!(game.perft(1), 16u); // FIXME
        assert_eq!(game.perft(2), 272u); // FIXME
        //assert_eq!(game.perft(1), 20u);
        //assert_eq!(game.perft(2), 400u);
        //assert_eq!(game.perft(3), 8902u);
    }

    #[test]
    fn test_generate_moves() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w";
        let game = Game::from_fen(fen);
        let moves = game.generate_moves();
        assert_eq!(moves.len(), 16);
    }

    #[test]
    fn test_play_move() {
        let fens = [
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w",
            "rnbqkbnr/pppppppp/8/8/8/4P3/PPPP1PPP/RNBQKBNR b"
        ];
        let m = Move::new(E2, E3, QUIET_MOVE);

        let mut game = Game::from_fen(fens[0]);
        assert_eq!(game.to_fen().as_slice(), fens[0]);
        game.play_move(m);
        assert_eq!(game.to_fen().as_slice(), fens[1]);
    }

    #[test]
    fn test_undo_move() {
        let fens = [
            "rnbqkbnr/pppppppp/8/8/8/4P3/PPPP1PPP/RNBQKBNR b",
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w"
        ];
        let m = Move::new(E2, E3, QUIET_MOVE);

        let mut game = Game::from_fen(fens[0]);
        assert_eq!(game.to_fen().as_slice(), fens[0]);
        game.undo_move(m);
        assert_eq!(game.to_fen().as_slice(), fens[1]);
    }
}
