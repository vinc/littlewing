use std::fmt;
use std::mem;

use common::*;
use clock::Clock;
use moves::{Move, Moves};
use positions::Positions;
use transpositions::{Transposition, Transpositions};
use zobrist::Zobrist;
use piece::{PieceAttr, PieceChar};

#[derive(Clone)]
pub struct Game {
    pub is_debug: bool,  // Print debugging
    pub is_verbose: bool, // Print thinking
    pub is_colored: bool,
    pub show_coordinates: bool,
    pub threads_count: usize,
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
            is_debug: false,
            is_verbose: false,
            is_colored: false,
            show_coordinates: false,
            threads_count: 0,
            nodes_count: 0,
            clock: Clock::new(40, 5 * 60),
            bitboards: [0; 14],
            board: [EMPTY; 64],
            moves: Moves::new(),
            positions: Positions::new(),
            zobrist: Zobrist::new(),
            history: Vec::new(),
            tt: Transpositions::with_memory(TT_SIZE)
        }
    }

    pub fn tt_resize(&mut self, memory: usize) {
        self.tt = Transpositions::with_memory(memory);
    }

    pub fn tt_size(&self) -> usize {
        self.tt.len() * mem::size_of::<Transposition>()
    }

    pub fn clear(&mut self) {
        self.bitboards = [0; 14];
        self.board = [EMPTY; 64];
        self.moves.clear_all();
        self.positions.clear();
        self.history.clear();
        self.tt.clear();
    }

    pub fn bitboard(&self, piece: Piece) -> &Bitboard {
        &self.bitboards[piece as usize]
    }
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut lines = vec![];

        let mut sep = "+---".repeat(8) + "+";
        if self.show_coordinates {
            sep = format!("   {}", sep);
        }
        lines.push(sep.clone());

        for rank in 0..8 {
            let mut line = if self.show_coordinates {
                format!(" {} ", 8 - rank)
            } else {
                format!("")
            };

            for file in 0..8 {
                let p = self.board[8 * (7 - rank) + file];
                let c = p.to_char();
                let s = if self.is_colored && p.color() == WHITE {
                    format!("\x1B[1m\x1B[37m{}\x1B[0m", c)
                } else if self.is_colored && p.color() == BLACK {
                    format!("\x1B[1m\x1B[31m{}\x1B[0m", c)
                } else {
                    format!("{}", c)
                };
                let s = format!("| {} ", s);
                line.push_str(&s);
            }
            line.push('|');
            lines.push(line);
            lines.push(sep.clone());
        }

        if self.show_coordinates {
            let line = " abcdefgh".chars().map(|c| format!(" {}  ", c)).collect();
            lines.push(line);
        }

        write!(f, "{}", lines.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tt_resize() {
        let mut game = Game::new();

        let size = 4 << 20; // 4 MB
        game.tt_resize(size);
        assert_eq!(game.tt_size(), size);
    }
}
