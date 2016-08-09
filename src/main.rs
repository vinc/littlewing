#[macro_use]
extern crate lazy_static;
extern crate regex;

mod protocols;
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
mod search;
mod square;
mod zobrist;

use protocols::cli::CLI;

fn main() {
    println!("Little Wing v0.0.1");
    println!("");

    let mut cli = CLI::new();
    cli.run();
}
