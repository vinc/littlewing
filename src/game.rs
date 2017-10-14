use std::fmt;
use std::mem;
use std::sync::Arc;

use common::*;
use clock::Clock;
use moves::{Move, Moves};
use positions::Positions;
use transpositions::{Transposition, Transpositions, SharedTranspositions};
use zobrist::Zobrist;
use piece::{PieceAttr, PieceChar};

#[derive(Clone)]
pub struct Game {
    pub is_debug: bool,  // Print debugging
    pub is_verbose: bool, // Print thinking
    pub is_colored: bool,
    pub concurrency: usize,
    pub nodes_count: u64,
    pub clock: Clock,
    pub bitboards: [Bitboard; 14],
    pub board: [Piece; 64],
    pub moves: Moves,
    pub positions: Positions,
    pub zobrist: Zobrist,
    pub history: Vec<Move>,
    pub tt: Arc<SharedTranspositions>
}

impl Game {
    pub fn new() -> Game {
        Game {
            is_debug: false,
            is_verbose: false,
            is_colored: false,
            concurrency: 1,
            nodes_count: 0,
            clock: Clock::new(40, 5 * 60),
            bitboards: [0; 14],
            board: [EMPTY; 64],
            moves: Moves::new(),
            positions: Positions::new(),
            zobrist: Zobrist::new(),
            history: Vec::new(),
            tt: Arc::new(SharedTranspositions::with_memory(TT_SIZE))
        }
    }

    pub fn tt(&self) -> &mut Transpositions {
        self.tt.get()
    }

    pub fn tt_resize(&mut self, memory: usize) {
        self.tt = Arc::new(SharedTranspositions::with_memory(memory));
    }

    pub fn tt_size(&self) -> usize {
        self.tt().size * mem::size_of::<Transposition>()
    }

    pub fn clear(&mut self) {
        self.bitboards = [0; 14];
        self.board = [EMPTY; 64];
        self.moves.clear_all();
        self.positions.clear();
        self.history.clear();
        self.tt().clear();
    }

    pub fn bitboard(&self, piece: Piece) -> &Bitboard {
        &self.bitboards[piece as usize]
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
                if self.is_colored {
                    if p.color() == WHITE {
                        line.push_str(&format!("\x1B[1m\x1B[37m{}\x1B[0m", c));
                    } else if p.color() == BLACK {
                        line.push_str(&format!("\x1B[1m\x1B[31m{}\x1B[0m", c));
                    }
                } else {
                    line.push(c);
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
