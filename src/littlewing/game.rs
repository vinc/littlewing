use littlewing::common::*;

use littlewing::bitboard::BitwiseOperations;
use littlewing::fen::FENBuilder;
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
        let mut i = 0u;
        let mut fields = fen.words();
        for c in fields.next().unwrap().chars() {
            let piece = match c {
                'p' => WHITE_PAWN,
                'n' => WHITE_KNIGHT,
                'b' => WHITE_BISHOP,
                'r' => WHITE_ROOK,
                'q' => WHITE_QUEEN,
                'k' => WHITE_KING,
                'P' => BLACK_PAWN,
                'N' => BLACK_KNIGHT,
                'B' => BLACK_BISHOP,
                'R' => BLACK_ROOK,
                'Q' => BLACK_QUEEN,
                'K' => BLACK_KING,
                '/' => continue,
                _   => {
                    if '1' <= c && c <= '8' {
                        i += c.to_digit(10).unwrap();
                    }
                    continue
                }
            };
            game.board[i] = piece;
            game.bitboards[piece].set(i);
            i += 1;
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
        let mut fen_builder = FENBuilder::new();
        for i in range(0u, 64) {
            if i > 0 && i % 8 == 0 {
                fen_builder.next_rank();
            }
            for &piece in PIECES.iter() {
                if self.bitboards[piece as uint].get(i) {
                    fen_builder.push(piece);
                    break;
                }
            }
            fen_builder.next_file();
        }
        fen_builder.set_side(self.side);
        fen_builder.to_string()
    }

    pub fn perft(&self, depth: uint) -> uint {
        let mut n = 0;

        if depth == 0 {
            return n
        }

        for _ in self.generate_moves().iter() {
            // TODO: play move
            n += 1 + self.perft(depth - 1);
        }

        n
    }

    pub fn generate_moves(&self) -> Moves {
        let mut moves = Vec::new();

        // FIXME: This is done to avoid writing self everywhere
        let bitboards = &self.bitboards;
        let side = self.side;

        let dirs = [UP, DOWN];
        let ranks = [RANK_3, RANK_6];
        let occupied = bitboards[WHITE] | bitboards[BLACK];

        let pushes = (bitboards[side | PAWN] << dirs[side]) & !occupied;
        moves.add_moves(pushes, dirs[side], QUIET_MOVE);

        let double_pushes = ((pushes & ranks[side]) << dirs[side]) & !occupied;
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
mod test {
    use super::Game;

    #[test]
    fn test_fen() {
        let fens = [
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w",
            "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b",
            "rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R w"
        ];
        for &fen in fens.iter() {
            let game = Game::from_fen(fen);
            assert!(game.to_fen().as_slice() == fen);
        }
    }

    #[test]
    fn test_perft() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w";
        let game = Game::from_fen(fen);
        assert!(game.perft(1) == 16u); // FIXME
        assert!(game.perft(2) == 272u); // FIXME
        //assert!(game.perft(1) == 20u);
        //assert!(game.perft(2) == 400u);
        //assert!(game.perft(3) == 8902u);
    }

    #[test]
    fn test_generate_moves() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w";
        let game = Game::from_fen(fen);
        let moves = game.generate_moves();
        assert!(moves.len() == 16);
    }
}
