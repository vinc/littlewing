#[macro_use]
extern crate lazy_static;
extern crate regex;

mod attack;
mod bitboard;
mod clock;
mod common;
mod eval;
mod fen;
mod game;
mod moves;
mod piece;
mod position;
mod protocols;
mod search;
mod square;
mod transpositions;
mod zobrist;

use protocols::cli::CLI;

fn main() {
    println!("Little Wing v0.1.0");
    println!("");

    let mut cli = CLI::new();
    cli.run();
}
