use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use color::*;
use piece::*;
use common::*;
use bitboard::Bitboard;
use clock::Clock;
use piece_move::PieceMove;
use piece_move_list::PieceMoveList;
use positions::Positions;
use protocols::Protocol;
use transposition_table::TranspositionTable;
use zobrist::Zobrist;
use piece::{PieceAttr, PieceChar};

/// A `Game` type to store the state of a chess game
#[derive(Clone)]
pub struct Game {
    pub protocol: Protocol,
    pub starting_fen: String,
    pub is_debug: bool,  // Print debugging
    pub is_eval_verbose: bool, // Print thinking in eval
    pub is_search_verbose: bool, // Print thinking in search
    pub is_colored: bool,
    pub show_coordinates: bool,
    pub threads_index: usize,
    pub threads_count: usize,
    pub current_depth: Arc<AtomicUsize>,
    pub nodes_count: u64,
    pub clock: Clock,
    pub bitboards: [Bitboard; 14],
    pub board: [Piece; 64],
    pub moves: PieceMoveList,
    pub positions: Positions,
    pub zobrist: Zobrist,
    pub history: Vec<PieceMove>,
    pub tt: TranspositionTable
}

impl Game {
    /// Create a new `Game`
    pub fn new() -> Game {
        Game {
            protocol: Protocol::CLI,
            starting_fen: String::from(DEFAULT_FEN),
            is_debug: false,
            is_eval_verbose: false,
            is_search_verbose: false,
            is_colored: false,
            show_coordinates: false,
            threads_index: 0,
            threads_count: 0,
            current_depth: Arc::new(AtomicUsize::new(0)),
            nodes_count: 0,
            clock: Clock::new(40, 5 * 60),
            bitboards: [0; 14],
            board: [EMPTY; 64],
            moves: PieceMoveList::new(),
            positions: Positions::new(),
            zobrist: Zobrist::new(),
            history: Vec::new(),
            tt: TranspositionTable::with_memory(TT_SIZE)
        }
    }

    pub fn get_current_depth(&mut self) -> Depth {
        self.current_depth.load(Ordering::SeqCst) as Depth
    }

    pub fn set_current_depth(&mut self, d: Depth) {
        let old = self.current_depth.load(Ordering::SeqCst);
        let new = d as usize;
        if new > old {
            self.current_depth.compare_and_swap(old, new, Ordering::SeqCst);
        }
    }

    pub fn reset_current_depth(&mut self) {
        self.current_depth.store(0, Ordering::SeqCst)
    }

    /// Get the transposition table size in byte
    pub fn tt_size(&self) -> usize {
        self.tt.memory()
    }

    /// Resize the transposition table at the given size in byte or the next
    /// power of two
    pub fn tt_resize(&mut self, memory: usize) {
        self.tt = TranspositionTable::with_memory(memory);
    }

    /// Clear the current game state
    pub fn clear(&mut self) {
        self.set_current_depth(0);
        self.bitboards = [0; 14];
        self.board = [EMPTY; 64];
        self.moves.clear_all();
        self.positions.clear();
        self.history.clear();
        self.tt.clear();
    }

    /// Get a bitboard representation of the given piece in the game
    #[inline]
    pub fn bitboard(&self, piece: Piece) -> &Bitboard {
        &self.bitboards[piece as usize]
    }

    /// Get the current side color
    pub fn side(&self) -> Color {
        self.positions.top().side
    }
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut lines = vec![];

        let sep = "+---".repeat(8) + "+";
        lines.push(sep.clone());

        for rank in 0..8 {
            let mut line = String::new();

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

            // Right border of the board
            let s = if self.show_coordinates {
                format!("| {}", 8 - rank)
            } else {
                format!("|")
            };
            line.push_str(&s);

            lines.push(line);
            lines.push(sep.clone());
        }

        if self.show_coordinates {
            let line = "abcdefgh".chars().
                map(|c| format!("  {} ", c)).collect();

            lines.push(line);
        }

        let indented_lines: Vec<String> = lines.iter().
            map(|l| format!("  {}", l)).collect();

        write!(f, "{}", indented_lines.join("\n"))
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
