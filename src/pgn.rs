use std::collections::BTreeMap;
use std::fmt;

use attack::*;
use color::*;
use common::*;
use fen::FEN;
use game::Game;
use piece_move_notation::PieceMoveNotation;
use piece_move_generator::PieceMoveGenerator;
use search::*;

#[derive(Debug)]
pub struct PGN {
    // Header keys are sorted by prefixing them with a number
    headers: BTreeMap<String, String>,
    game: String,
}

impl PGN {
    fn new() -> PGN {
        PGN {
            headers: vec![
                ("1Event".to_string(), "?".to_string()),
                ("2Site".to_string(), "?".to_string()),
                ("3White".to_string(), "?".to_string()),
                ("4Black".to_string(), "?".to_string()),
                ("5Result".to_string(), "?".to_string())
            ].into_iter().collect(),
            game: "".to_string(),
        }
    }

    pub fn set_white(&mut self, white: &str) {
        self.headers.insert("3White".to_string(), white.to_string());
    }

    pub fn set_black(&mut self, black: &str) {
        self.headers.insert("4Black".to_string(), black.to_string());
    }

    pub fn set_result(&mut self, result: &str) {
        self.headers.insert("5Result".to_string(), result.to_string());
    }

    pub fn set_fen(&mut self, fen: &str) {
        self.headers.insert("6FEN".to_string(), fen.to_string());
        self.headers.insert("7SetUp".to_string(), "1".to_string());
    }
}

impl fmt::Display for PGN {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (key, val) in self.headers.iter() {
            writeln!(f, "[{} \"{}\"]", key.trim_start_matches(char::is_numeric), val)?;
        }
        writeln!(f, "")?;
        writeln!(f, "{}{}", self.game, self.headers["5Result"])?;
        Ok(())
    }
}

/// Portable Game Notation support
pub trait ToPGN {
    /// Export Game to PGN
    fn to_pgn(&mut self) -> PGN;
}

impl ToPGN for Game {
    fn to_pgn(&mut self) -> PGN {
        let mut pgn = PGN::new();

        let starting_fen = self.starting_fen.clone();

        if starting_fen != DEFAULT_FEN {
            pgn.set_fen(&starting_fen);
        }

        if self.is_mate() {
            if self.is_check(WHITE) {
                pgn.set_result("0-1");
            } else if self.is_check(BLACK) {
                pgn.set_result("1-0");
            } else {
                pgn.set_result("1/2-1/2");
            }
        } else {
            pgn.set_result("*");
        }

        let moves = self.history.clone();
        self.load_fen(&starting_fen);

        let mut first_move = true;
        let mut line = String::new();
        for m in moves {
            let fm = self.positions.fullmoves();
            if self.side() == WHITE {
                line.push_str(&format!("{}. ", fm));
            } else if first_move {
                line.push_str(&format!("{}. ... ", fm));
            }
            first_move = false;

            line.push_str(&self.move_to_san(m));

            self.make_move(m);
            self.history.push(m);

            if self.is_mate() {
                line.push('#');
            } else if self.is_check(self.side()) {
                line.push('+');
            }

            if line.len() > 70 {
                pgn.game.push_str(&format!("{}\n", line));
                line = String::new();
            } else {
                line.push(' ');
            }
        }
        pgn.game.push_str(&format!("{}", line));

        pgn
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::path::Path;
    use std::io::prelude::*;

    use square::*;
    use common::*;
    use game::Game;
    use piece_move::PieceMove;
    use piece_move_generator::PieceMoveGenerator;

    use super::*;

    #[test]
    fn test_game_to_pgn() {
        let mut game = Game::from_fen(DEFAULT_FEN);
        let moves = vec![
            PieceMove::new(F2, F3, QUIET_MOVE),
            PieceMove::new(E7, E5, DOUBLE_PAWN_PUSH),
            PieceMove::new(G2, G4, DOUBLE_PAWN_PUSH),
            PieceMove::new(D8, H4, QUIET_MOVE)
        ];
        for m in moves {
            game.make_move(m);
            game.history.push(m);
        }
        let pgn = game.to_pgn();

        let path = Path::new("tests/fool.pgn");
        let mut file = File::open(&path).unwrap();
        let mut res = String::new();
        file.read_to_string(&mut res).unwrap();

        assert_eq!(pgn.to_string(), res);
    }
}
