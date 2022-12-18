use crate::std::prelude::v1::*;
use crate::std::fmt;

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

use crate::board;
use crate::color::*;
use crate::piece::*;
use crate::common::*;
use crate::bitboard::Bitboard;
use crate::clock::Clock;
use crate::piece_move::PieceMove;
use crate::piece_move_list::PieceMoveList;
use crate::positions::Positions;
use crate::transposition_table::TranspositionTable;
use crate::zobrist::Zobrist;
use crate::piece::{PieceAttr, PieceChar};
#[cfg(feature = "std")]
use crate::protocols::Protocol;

/// A `Game` type to store the state of a chess game
#[derive(Clone)]
pub struct Game {
    #[cfg(feature = "std")]
    pub protocol: Protocol,

    pub starting_fen: String,
    pub is_debug: bool,  // Print debugging
    pub is_eval_verbose: bool, // Print thinking in eval
    pub is_search_verbose: bool, // Print thinking in search
    pub show_coordinates: bool,
    pub threads_index: usize,
    pub threads_count: usize,
    pub current_depth: Arc<AtomicUsize>,
    pub nodes_count: Arc<AtomicU64>,
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
            #[cfg(feature = "std")]
            protocol: Protocol::CLI,

            starting_fen: String::from(DEFAULT_FEN),
            is_debug: false,
            is_eval_verbose: false,
            is_search_verbose: false,
            show_coordinates: false,
            threads_index: 0,
            threads_count: 0,
            current_depth: Arc::new(AtomicUsize::new(0)),
            nodes_count: Arc::new(AtomicU64::new(0)),
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

    pub fn current_depth(&self) -> Depth {
        self.current_depth.load(Ordering::Relaxed) as Depth
    }

    pub fn set_current_depth(&mut self, d: Depth) {
        let ord = Ordering::Relaxed;
        let old = self.current_depth.load(ord);
        let new = d as usize;
        if new > old {
            self.current_depth.compare_exchange(old, new, ord, ord);
        }
    }

    pub fn reset_current_depth(&mut self) {
        self.current_depth.store(0, Ordering::Relaxed)
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

    /// Get the shared nodes count
    pub fn nodes_count(&self) -> u64 {
        self.nodes_count.load(Ordering::Relaxed)
    }

    /// Increment the shared nodes count
    pub fn inc_nodes_count(&mut self) {
        self.nodes_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Reset the shared nodes count
    pub fn reset_nodes_count(&mut self) {
        self.nodes_count = Arc::new(AtomicU64::new(0));
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
        let squares = (0..64).map(|i| {
            let p = self.board[i];
            let c = p.to_char().to_string();
            if p.color() == WHITE {
                bold_white(&c)
            } else if p.color() == BLACK {
                bold_red(&c)
            } else {
                c
            }
        }).collect();

        let board = if self.show_coordinates {
            board::draw_with_coordinates(squares)
        } else {
            board::draw(squares)
        };

        write!(f, "{}", board)
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
