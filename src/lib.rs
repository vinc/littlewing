//! Little Wing is a chess engine rated at 2050+ ELO, compatible with both UCI
//! and XBoard protocols, with a nice CLI, and a documented library.
//!
//! # Example
//!
//! ```rust
//! use littlewing::chess::*;
//!
//! // Byrne vs Fischer (1956)
//! let fen = "r3r1k1/pp3pbp/1Bp1b1p1/8/2BP4/Q1n2N2/P4PPP/3R1K1R b - - 0 18";
//!
//! let mut game = Game::from_fen(fen).unwrap();
//!
//! game.clock = Clock::new(1, 5000); // Search 1 move in 5 seconds
//!
//! match game.search(1..15) { // Search from depth 1 to 15
//!     Some(m) => {
//!         assert_eq!(game.move_to_san(m), "Bxc4");
//!
//!         game.make_move(m);
//!         game.history.push(m); // Keep track of the moves played
//!
//!         println!("Engine played {}", m.to_lan());
//!     },
//!     None => {
//!         println!("Engine could not find a move to play");
//!     }
//! }
//! ```

#[macro_use]
extern crate lazy_static;
extern crate colored;
extern crate dirs;
extern crate rand;
extern crate rand_xorshift;
extern crate regex;
extern crate rustyline;
extern crate rustyline_derive;

mod attack;
mod board;
mod common;
mod dumb7fill;
mod hyperbola;
mod piece_move;
mod piece_move_list;
mod positions;
mod piece_square_table;
mod transposition;
mod transposition_table;
mod zobrist;

/// Bitboard type
pub mod bitboard;

/// Clock controls
pub mod clock;

/// Color type
pub mod color;

/// Evaluation algorithms
pub mod eval;

/// Forsyth–Edwards Notation support
pub mod fen;

/// Game engine
pub mod game;

/// Portable Game Notation support
pub mod pgn;

/// Piece move generator
pub mod piece_move_generator;

/// Piece move notation
pub mod piece_move_notation;

/// Piece type
pub mod piece;

/// Communication protocols
pub mod protocols;

/// Search algorithms
pub mod search;

/// Square type
pub mod square;

/// Chess prelude
pub mod chess {
    pub use attack::Attack;
    pub use clock::Clock;
    pub use color;
    pub use fen::FEN;
    pub use game::Game;
    pub use piece_move_generator::PieceMoveGenerator;
    pub use piece_move_notation::PieceMoveNotation;
    pub use search::Search;
}

/// Return Little Wing's version
pub fn version() -> String {
    let ver = String::from("v") + env!("CARGO_PKG_VERSION");
    let ver = option_env!("LITTLEWING_VERSION").unwrap_or(&ver);
    format!("Little Wing {}", ver)
}
