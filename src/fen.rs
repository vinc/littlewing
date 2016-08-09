use std;

use common::*;
use bitboard::BitboardExt;
use game::Game;
use piece::PieceChar;
use position::Position;

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

        let mut fields = fen.split_whitespace();

        let mut sq = A8;
        for c in fields.next().unwrap().chars() {
            let dir = if c == '/' {
                2 * DOWN
            } else if '1' <= c && c <= '8' {
                c.to_digit(10).unwrap() as Direction
            } else {
                let p = PieceChar::from_char(c);
                self.board[sq as usize] = p;
                self.bitboards[(p) as usize].set(sq);
                self.bitboards[(p & 1) as usize].set(sq); // TODO: p.color()

                1
            };
            //sq += dir;
            sq = ((sq as i8) + dir) as Square;
        }

        let mut position = Position::new();

        position.side = match fields.next().unwrap() {
            "w" => WHITE,
            "b" => BLACK,
            _   => panic!("wrong side char")
        };

        for c in fields.next().unwrap().chars() {
            match c {
                'K' => position.castling_rights[WHITE as usize][(KING >> 3) as usize] = true,
                'Q' => position.castling_rights[WHITE as usize][(QUEEN >> 3) as usize] = true,
                'k' => position.castling_rights[BLACK as usize][(KING >> 3) as usize] = true,
                'q' => position.castling_rights[BLACK as usize][(QUEEN >> 3) as usize] = true,
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
            let p = self.board[sq as usize];

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
                //sq += 2 * DOWN;
                sq = ((sq as i8) + 2 * DOWN) as Square; // 0 <= sq <= 64
            }

            //sq += RIGHT;
            sq = ((sq as i8) + RIGHT) as Square; // 0 <= sq <= 64
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
        if castling_rights[WHITE as usize][(KING >> 3) as usize] {
            castles.push('K');
        }
        if castling_rights[WHITE as usize][(QUEEN >> 3) as usize] {
            castles.push('Q');
        }
        if castling_rights[BLACK as usize][(KING >> 3) as usize] {
            castles.push('k');
        }
        if castling_rights[BLACK as usize][(QUEEN >> 3) as usize] {
            castles.push('q');
        }
        if castles.is_empty() {
            castles.push('-');
        }
        fen.push_str(&*castles);

        fen.push_str(" - 0 1"); // TODO

        fen
    }
}

#[cfg(test)]
mod tests {
    use common::*;
    use fen::FEN;
    use game::Game;

    #[test]
    fn test_from_fen() {
        let game: Game = FEN::from_fen(DEFAULT_FEN);
        assert_eq!(game.board[E2 as usize], WHITE_PAWN);
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
            assert_eq!(&*(game.to_fen()), fen);
        }
    }
}
