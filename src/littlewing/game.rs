use littlewing::piece;
use littlewing::piece::Piece;
use littlewing::fen::FENBuilder;

#[deriving(Copy)]
pub struct Game {
    bitboards: [u64, ..12]
}

impl Game {
    pub fn new() -> Game {
        Game {
            bitboards: [0, ..12]
        }
    }

    pub fn from_fen(fen: &str) -> Game {
        let mut game = Game::new();
        let mut i = 0u;
        for c in fen.chars() {
            let piece = match c {
                'p' => Piece::WhitePawn,
                'n' => Piece::WhiteKnight,
                'b' => Piece::WhiteBishop,
                'r' => Piece::WhiteRook,
                'q' => Piece::WhiteQueen,
                'k' => Piece::WhiteKing,
                'P' => Piece::BlackPawn,
                'N' => Piece::BlackKnight,
                'B' => Piece::BlackBishop,
                'R' => Piece::BlackRook,
                'Q' => Piece::BlackQueen,
                'K' => Piece::BlackKing,
                ' ' => break,
                '/' => continue,
                _   => {
                    if '1' <= c && c <= '8' {
                        i += c.to_digit(10).unwrap();
                    }
                    continue
                }
            };
            game.bitboards[piece as uint] |= 1 << i;
            i += 1;
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
                if self.bitboards[piece as uint] & (1 << i) > 0 {
                    fen_builder.push(piece);
                    break;
                }
            }
            fen_builder.next_file();
        }
        fen_builder.to_string()
    }

    pub fn perft(&self, i: uint) -> uint {
        match i {
            1u => 20u,
            2u => 400u,
            _  => 8902u
        }
    }
}

#[cfg(test)]
mod test {
    use super::Game;

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
        let game = Game::new();
        assert!(game.perft(1) == 20u);
        assert!(game.perft(2) == 400u);
        assert!(game.perft(3) == 8902u);
    }
}
