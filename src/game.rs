use std::fmt;

use common::*;
use clock::Clock;
use moves::{Move, Moves};
use position::Positions;
use transpositions::Transpositions;
use zobrist::Zobrist;
use piece::{PieceAttr, PieceChar};

pub struct Game {
    pub is_verbose: bool,
    pub nodes_count: u64,
    pub clock: Clock,
    pub bitboards: [Bitboard; 14],
    pub board: [Piece; 64],
    pub moves: Moves,
    pub positions: Positions,
    pub zobrist: Zobrist,
    pub history: Vec<Move>,
    pub tt: Transpositions
}

impl Game {
    pub fn new() -> Game {
        Game {
            is_verbose: false,
            nodes_count: 0,
            clock: Clock::new(40, 5 * 60),
            bitboards: [0; 14],
            board: [EMPTY; 64],
            moves: Moves::new(),
            positions: Positions::new(),
            zobrist: Zobrist::new(),
            history: Vec::new(),
            tt: Transpositions::with_capacity(100000)
        }
    }

    pub fn clear(&mut self) {
        self.bitboards = [0; 14];
        self.board = [EMPTY; 64];
        self.moves.clear_all();
        self.positions.clear();
        self.history.clear();
    }
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut lines = vec![];

        let sep = (0..8).map(|_| "+---").fold(String::new(), |r, s| r + s) + "+";
        lines.push(sep.clone());
        for i in 0..8 {
            let mut line = String::from("");
            for j in 0..8 {
                line.push_str("| ");
                let p = self.board[8 * (7 - i) + j];
                let c = p.to_char();
                if p.color() == WHITE {
                    line.push_str(&format!("\x1B[1m\x1B[37m{}\x1B[0m", c));
                } else if p.color() == BLACK {
                    line.push_str(&format!("\x1B[1m\x1B[31m{}\x1B[0m", c));
                }
                line.push_str(" ");
            }
            line.push_str("|");
            lines.push(line);
            lines.push(sep.clone());
        }

        write!(f, "{}", lines.join("\n"))
    }
}
