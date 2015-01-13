use std;

use littlewing::common::*;
use littlewing::bitboard::BitboardExt;
use littlewing::game::Game;
use littlewing::piece::PieceChar;
use littlewing::position::Position;

pub trait FEN {
    fn from_fen(fen: &str) -> Self;
    fn load_fen(&mut self, fen: &str);
    fn to_fen(&self) -> String;
}

impl FEN for Game {
    fn from_fen(fen: &str) -> Game {
        let mut game = Game::new();

        game.load_fen(fen);
        game
    }

    fn load_fen(&mut self, fen: &str) {
        self.clear();

        let mut fields = fen.words();

        let mut sq = A8;
        for c in fields.next().unwrap().chars() {
            sq += if c == '/' {
                2 * DOWN
            } else if '1' <= c && c <= '8' {
                c.to_digit(10).unwrap()
            } else {
                let p = PieceChar::from_char(c);
                self.board[sq] = p;
                self.bitboards[p].set(sq);
                self.bitboards[p & 1].set(sq); // TODO: p.color()

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

        self.positions.push(position);
    }

    fn to_fen(&self) -> String {
        let mut fen = String::new();
        let mut n = 0;
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
}

#[cfg(test)]
mod tests {
    use littlewing::common::*;
    use littlewing::fen::FEN;
    use littlewing::game::Game;

    #[test]
    fn test_from_fen() {
        let game: Game = FEN::from_fen(DEFAULT_FEN);
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
            let game: Game = FEN::from_fen(fen);
            assert_eq!(game.to_fen().as_slice(), fen);
        }
    }
}
