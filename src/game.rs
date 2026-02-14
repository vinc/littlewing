use std::prelude::v1::*;
use std::fmt;

use crate::board;
use crate::color::*;
use crate::piece::*;
use crate::common::*;
use crate::bitboard::Bitboard;
use crate::clock::Clock;
use crate::history::HistoryHeuristic;
use crate::piece_move::PieceMove;
use crate::piece_move_list::PieceMoveList;
use crate::positions::Positions;
use crate::transposition_table::TranspositionTable;
use crate::zobrist::Zobrist;
use crate::piece::{PieceAttr, PieceChar};
#[cfg(feature = "std")]
use crate::protocols::Protocol;

#[derive(Clone)]
pub struct TunableParam {
    pub val: i32,
    pub min: i32,
    pub max: i32,
    pub step: i32,
}

impl TunableParam {
    pub fn new(val: i32, min: i32, max: i32, step: i32) -> Self {
        Self { val, min, max, step }
    }
}

#[derive(Clone)]
pub struct TunableParams {
    // History Heuristic
    pub hhb_quadra: TunableParam, // Bonus: based on depth
    pub hhb_linear: TunableParam, //        (x * d * d) + (y * d) + z
    pub hhb_offset: TunableParam, //          quadratic + linear  + offset
    pub hhm_quadra: TunableParam, // Malus: same formula (negated)
    pub hhm_linear: TunableParam,
    pub hhm_offset: TunableParam,
    pub hh_clamp: TunableParam,   // Clamp: bonus.clamp(0, max)

    // Delta Pruning
    pub dp_margin: TunableParam,

    // Futility Pruning
    pub fp_margin: TunableParam, // Check: eval + margin * depth < alpha

    // Late Move Reduction
    pub lmr_hm: TunableParam,  // History Margin
    pub lmr_min: TunableParam, // Reduction: based on depth and moves tried
    pub lmr_div: TunableParam, //            min + ln(depth) * ln(moves) / div
    pub lmr: Box<[[Depth; MAX_MOVES]; MAX_PLY]>,
}

impl TunableParams {
    pub fn new() -> Self {
        let mut params = Self {
            // History Heuristic
            hhb_quadra: TunableParam::new(16, 2, 64, 8),
            hhb_linear: TunableParam::new(128, 16, 512, 64),
            hhb_offset: TunableParam::new(-256, -512, -128, 32),
            hhm_quadra: TunableParam::new(16, 2, 64, 8),
            hhm_linear: TunableParam::new(128, 16, 512, 8),
            hhm_offset: TunableParam::new(-256, -500, -100, 25),
            hh_clamp: TunableParam::new(4096, 1024, 16384, 1024),

            // Delta Pruning
            dp_margin: TunableParam::new(1000, 800, 1200, 50),

            // Futility Pruning
            fp_margin: TunableParam::new(100, 25, 250, 25),

            // Late Move Reduction
            lmr_hm: TunableParam::new(1024, 0, 8192, 256),
            lmr_min: TunableParam::new(75, 50, 100, 10),
            lmr_div: TunableParam::new(250, 200, 300, 25),
            lmr: Box::new([[0; MAX_MOVES]; MAX_PLY]),
        };
        params.compute_lmr();
        params
    }

    pub fn compute_lmr(&mut self) {
        let min = (self.lmr_min.val as f64) / 100.0;
        let div = (self.lmr_div.val as f64) / 100.0;
        for depth in 1..MAX_PLY {
            for moves in 1..MAX_MOVES {
                let r = min + (depth as f64).ln() * (moves as f64).ln() / div;
                debug_assert!(r >= 0.0);
                debug_assert!(r < Depth::MAX as f64);
                self.lmr[depth][moves] = r.round() as Depth;
            }
        }
    }
}

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
    pub threads_count: usize,
    pub nodes_count: u64,
    pub clock: Clock,
    pub bitboards: [Bitboard; 14],
    pub board: [Piece; 64],
    pub moves: PieceMoveList,
    pub plies: Vec<PieceMove>,
    pub history: [[[Score; 64]; 64]; 2],
    pub positions: Positions,
    pub zobrist: Zobrist,
    pub tt: TranspositionTable,
    pub params: TunableParams,
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
            threads_count: 0,
            nodes_count: 0,
            clock: Clock::new(40, 5 * 60),
            bitboards: [0; 14],
            board: [EMPTY; 64],
            moves: PieceMoveList::new(),
            plies: Vec::new(),
            history: [[[0; 64]; 64]; 2],
            positions: Positions::new(),
            zobrist: Zobrist::new(),
            tt: TranspositionTable::with_memory(TT_SIZE),
            params: TunableParams::new(),
        }
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
        self.bitboards = [0; 14];
        self.board = [EMPTY; 64];
        self.moves.clear_all();
        self.positions.clear();
        self.clear_history();
        self.plies.clear();
        //self.tt.clear();
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
