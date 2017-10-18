//! Little Wing
//!
//! Little Wing is a bitboard chess engine.
//!
//! It has been a work in progress since 2014 and the library is not stabilized
//! yet, but the goal is to have an engine with modern algorithms, a simple API
//! and a great CLI with support for XBoard and UCI protocols.
//!
//! # Example
//!
//! ```rust
//! use littlewing::game::Game;
//! use littlewing::fen::FEN;
//! use littlewing::clock::Clock;
//! use littlewing::search::Search;
//! use littlewing::moves_generator::MovesGenerator;
//!
//! // Byrne vs Fischer (1956)
//! let fen = "r3r1k1/pp3pbp/1Bp1b1p1/8/2BP4/Q1n2N2/P4PPP/3R1K1R b - - 0 18";
//!
//! let mut game = Game::from_fen(fen);
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
//!         println!("Engine played {}", m.to_can());
//!     },
//!     None => {
//!         println!("Engine could not find a move to play");
//!     }
//! }
//! ```

#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate rustyline;

mod attack;
mod common;
mod moves;
mod positions;
mod transpositions;
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

/// Moves generator
pub mod moves_generator;

/// Piece type
pub mod piece;

/// Communication protocols
pub mod protocols;

/// Search algorithms
pub mod search;

/// Square type
pub mod square;

/// Return Little Wing's version
pub fn version() -> String {
    let ver = String::from("v") + env!("CARGO_PKG_VERSION");
    let ver = option_env!("LITTLEWING_VERSION").unwrap_or(&ver);
    format!("Little Wing {}", ver)
}
