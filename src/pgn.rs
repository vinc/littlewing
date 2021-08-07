use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use core::fmt;

#[cfg(feature = "std")]
use regex::Regex;

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

    body: String,
}

lazy_static! {
    static ref DEFAULT_HEADERS: Vec<(String, String)> = vec![
        ("1Event".to_string(), "?".to_string()),
        ("2Site".to_string(), "?".to_string()),
        ("3White".to_string(), "?".to_string()),
        ("4Black".to_string(), "?".to_string()),
        ("5Result".to_string(), "*".to_string()),
    ];
}

impl PGN {
    fn new() -> PGN {
        PGN {
            headers: DEFAULT_HEADERS.clone().into_iter().collect(),
            body: "".to_string(),
        }
    }

    pub fn white(&self) -> String {
        self.headers["3White"].clone()
    }

    pub fn black(&self) -> String {
        self.headers["4Black"].clone()
    }

    pub fn result(&self) -> String {
        self.headers["5Result"].clone()
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
        self.headers.insert("FEN".to_string(), fen.to_string());
        self.headers.insert("SetUp".to_string(), "1".to_string());
    }

    pub fn set_header(&mut self, key: &str, val: &str) {
        self.headers.insert(key.to_string(), val.to_string());
    }
}

impl fmt::Display for PGN {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (key, val) in self.headers.iter() {
            writeln!(f, "[{} \"{}\"]", key.trim_start_matches(char::is_numeric), val)?;
        }
        writeln!(f, "")?;
        write!(f, "{}", self.body)?;
        Ok(())
    }
}

#[cfg(feature = "std")]
impl From<String> for PGN {
    fn from(s: String) -> PGN {
        lazy_static! {
            static ref RE: Regex = Regex::new("\\[(?P<key>\\w+) \"(?P<val>.*)\"\\]").unwrap();
        }
        let mut pgn = PGN::new();
        for line in s.lines() {
            match RE.captures(line) {
                Some(header) => {
                    // Keep only the last game
                    if !pgn.body.is_empty() {
                        pgn = PGN::new();
                    }

                    let key = header["key"].to_string();
                    let val = header["val"].to_string();
                    let mut is_default_header = false;
                    for (k, _) in DEFAULT_HEADERS.iter() {
                        if key == k.trim_start_matches(char::is_numeric) {
                            pgn.set_header(k, &val);
                            is_default_header = true;
                            break;
                        }
                    }
                    if !is_default_header {
                        pgn.set_header(&key, &val);
                    }
                }
                None => {
                    if !line.is_empty() {
                        pgn.body.push_str(line);
                        pgn.body.push('\n');
                    }
                }
            }
        }
        pgn
    }
}

/// Portable Game Notation export
pub trait ToPGN {
    /// Export to PGN
    fn to_pgn(&mut self) -> PGN;
}

impl ToPGN for Game {
    fn to_pgn(&mut self) -> PGN {
        let mut pgn = PGN::new();

        let starting_fen = self.starting_fen.clone();
        if starting_fen != DEFAULT_FEN {
            pgn.set_fen(&starting_fen);
        }

        let result = if self.is_mate() {
            if self.is_check(WHITE) {
                "0-1"
            } else if self.is_check(BLACK) {
                "1-0"
            } else {
                "1/2-1/2"
            }
        } else {
            "*"
        };
        pgn.set_result(result);

        let moves = self.history.clone();
        self.load_fen(&starting_fen).unwrap();

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
                pgn.body.push_str(&format!("{}\n", line));
                line = String::new();
            } else {
                line.push(' ');
            }
        }
        pgn.body.push_str(&format!("{}{}\n", line, result));

        pgn
    }
}

/// Portable Game Notation import
#[cfg(feature = "std")]
pub trait LoadPGN {
    /// Load PGN
    fn load_pgn(&mut self, pgn: PGN);
}

#[cfg(feature = "std")]
impl LoadPGN for Game {
    fn load_pgn(&mut self, pgn: PGN) {
        self.clear();
        let starting_fen = pgn.headers.get("FEN").map_or(DEFAULT_FEN, String::as_str);
        self.load_fen(starting_fen).unwrap();
        let mut comment_level = 0;
        let mut variation_level = 0;
        for line in pgn.body.lines() {
            for word in line.split(" ") {
                if word.starts_with(";") {
                    break;
                }
                comment_level += word.matches("{").count();
                comment_level -= word.matches("}").count();
                variation_level += word.matches("(").count();
                variation_level -= word.matches(")").count();
                if comment_level > 0 || variation_level > 0 {
                    continue;
                }

                if let Some(m) = self.parse_move(word) {
                    self.make_move(m);
                    self.history.push(m);
                }
            }
        }
    }
}

#[cfg(feature = "std")]
#[cfg(test)]
mod tests {
    use std::fs;

    use square::*;
    use common::*;
    use game::Game;
    use piece_move::PieceMove;
    use piece_move_generator::PieceMoveGenerator;

    use super::*;

    #[test]
    fn test_game_to_pgn() {
        let mut game = Game::from_fen(DEFAULT_FEN).unwrap();
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

        let content = fs::read_to_string("tests/fool.pgn").unwrap();
        assert_eq!(pgn.to_string(), content);
    }

    #[test]
    fn test_string_to_pgn() {
        let content = fs::read_to_string("tests/fool.pgn").unwrap();
        let pgn = PGN::from(content.clone());
        assert_eq!(pgn.to_string(), content);
        assert_eq!(pgn.result(), "0-1".to_string());
    }

    #[test]
    fn test_game_load_pgn() {
        let mut game = Game::new();

        let s1 = fs::read_to_string("tests/fool.pgn").unwrap();
        let pgn = PGN::from(s1.clone());
        game.load_pgn(pgn);
        assert_eq!(game.history.len(), 4);

        let s2 = fs::read_to_string("tests/zukertort_vs_steinitz_1886.pgn").unwrap();
        let pgn = PGN::from(s2.clone());
        game.load_pgn(pgn);
        assert_eq!(game.history.len(), 58);

        let pgn = PGN::from(format!("{}\n{}", s1, s2));
        game.load_pgn(pgn);
        assert_eq!(game.history.len(), 58);
    }
}
