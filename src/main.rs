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
mod moves_generator;
mod piece;
mod position;
mod protocols;
mod search;
mod square;
mod transpositions;
mod zobrist;

use std::env;

use protocols::cli::CLI;

fn version() -> String {
    let ver = String::from("v") + env!("CARGO_PKG_VERSION");
    let ver = option_env!("LITTLEWING_VERSION").unwrap_or(&ver);
    format!("Little Wing {}", ver)
}

fn main() {
    println!("{}", version());
    println!("");

    let mut cli = CLI::new(env::args().collect());
    cli.run();
}
