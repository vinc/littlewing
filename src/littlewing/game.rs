use littlewing::piece;
use littlewing::piece::*;

use littlewing::bitboard;
use littlewing::bitboard::Bitboard;
use littlewing::bitboard::BitwiseOperations;
use littlewing::fen::FENBuilder;
use littlewing::moves::Move;
use littlewing::moves::Moves;
use littlewing::moves::MovesOperations;
use littlewing::moves::MoveCategory;

const UP:    uint = 8u;
const DOWN:  uint = -8u;
const LEFT:  uint = -1u;
const RIGHT: uint = 1u;

#[deriving(Copy)]
pub struct Game {
    bitboards: [Bitboard, ..14]
}

impl Game {
    pub fn new() -> Game {
        Game {
            bitboards: [0, ..14]
        }
    }

    pub fn from_fen(fen: &str) -> Game {
        let mut game = Game::new();
        let mut i = 0u;
        for c in fen.chars() {
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
                ' ' => break,
                '/' => continue,
                _   => {
                    if '1' <= c && c <= '8' {
                        i += c.to_digit(10).unwrap();
                    }
                    continue
                }
            };
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
        game
    }

    pub fn to_fen(&self) -> String {
        let mut fen_builder = FENBuilder::new();
        for i in range(0u, 64) {
            if i > 0 && i % 8 == 0 {
                fen_builder.next_rank();
            }
            for &piece in piece::PIECES.iter() {
                if self.bitboards[piece as uint].get(i) {
                    fen_builder.push(piece);
                    break;
                }
            }
            fen_builder.next_file();
        }
        fen_builder.to_string()
    }

    pub fn perft(&self, depth: uint) -> uint {
        let mut n = 0;

        if depth == 0 {
            return n
        }

        for m in self.generate_moves().iter() {
            // TODO: play move
            n += 1 + self.perft(depth - 1);
        }

        n
    }

    pub fn generate_moves(&self) -> Moves {
        let bitboards = &self.bitboards; // Make self implicite
        let mut moves = Vec::new();

        let occupied = bitboards[WHITE] | bitboards[BLACK];

        let pushes = (bitboards[WHITE_PAWN] << 8) & !occupied;
        moves.add_moves(pushes, UP, MoveCategory::QuietMove);

        let double_pushes = ((pushes & bitboard::RANK_3) << 8) & !occupied;
        moves.add_moves(double_pushes, UP + UP, MoveCategory::DoublePawnPush);

        /*
        let left_attacks = (bitboards[WHITE_PAWN] << 7) & bitboards[BLACK];
        moves.add_moves(left_attacks, UP + LEFT, MoveCategory::Capture);

        let right_attacks = (bitboards[WHITE_PAWN] << 9) & bitboards[BLACK];
        moves.add_moves(right_attacks, UP + RIGHT, MoveCategory::Capture);
        */

        moves
    }
}

#[cfg(test)]
mod test {
    use super::Game;
    use littlewing::moves::MoveCategory;

    #[test]
    fn test_fen() {
        let fens = [
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR",
            "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR",
            "rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R"
        ];
        for &fen in fens.iter() {
            let game = Game::from_fen(fen);
            assert!(game.to_fen().as_slice() == fen);
        }
    }

    #[test]
    fn test_perft() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR";
        let mut game = Game::from_fen(fen);
        assert!(game.perft(1) == 16u); // FIXME
        assert!(game.perft(2) == 272u); // FIXME
        //assert!(game.perft(1) == 20u);
        //assert!(game.perft(2) == 400u);
        //assert!(game.perft(3) == 8902u);
    }

    #[test]
    fn test_generate_moves() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR";
        let mut game = Game::from_fen(fen);
        let moves = game.generate_moves();
        assert!(moves.len() == 16);
    }
}
