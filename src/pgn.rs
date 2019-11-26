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
    white: String,
    black: String,
    fen: String,
    game: String,
    result: String,
}

impl PGN {
    fn new() -> PGN {
        PGN {
            white: "?".to_string(),
            black: "?".to_string(),
            fen: "".to_string(),
            game: "".to_string(),
            result: "*".to_string(),
        }
    }

    pub fn set_white(&mut self, player: &str) {
        self.white = player.to_string();
    }

    pub fn set_black(&mut self, player: &str) {
        self.black = player.to_string();
    }
}

impl fmt::Display for PGN {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "[Event \"?\"")?;
        writeln!(f, "[Site \"?\"")?;
        writeln!(f, "[White \"{}\"", self.white)?;
        writeln!(f, "[Black \"{}\"", self.black)?;
        writeln!(f, "[Result \"{}\"", self.result)?;
        if self.fen != DEFAULT_FEN.to_string() {
            writeln!(f, "[FEN \"{}\"", self.fen)?;
            writeln!(f, "[SetUp \"1\"")?;
        }
        writeln!(f, "")?;
        writeln!(f, "{}{}", self.game, self.result)?;
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

        pgn.fen = self.starting_fen.clone();

        pgn.result = if self.is_mate() {
            if self.is_check(WHITE) {
                "0-1".to_string()
            } else if self.is_check(BLACK) {
                "1-0".to_string()
            } else {
                "1/2-1/2".to_string()
            }
        } else {
            "*".to_string()
        };

        let moves = self.history.clone();
        self.load_fen(&pgn.fen);

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
