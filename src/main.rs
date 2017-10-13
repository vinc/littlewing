#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate rustyline;
extern crate getopts;

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
mod positions;
mod protocols;
mod search;
mod square;
mod transpositions;
mod zobrist;

use std::env;

use getopts::Options;

use protocols::cli::CLI;

fn print_usage(opts: Options) {
    let brief = format!("Usage: littlewing [options]");
    print!("{}", opts.usage(&brief));
}

fn version() -> String {
    let ver = String::from("v") + env!("CARGO_PKG_VERSION");
    let ver = option_env!("LITTLEWING_VERSION").unwrap_or(&ver);
    format!("Little Wing {}", ver)
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut opts = Options::new();
    opts.optopt("t",  "tt",      "set transposition table size (in MB)", "SIZE");
    opts.optflag("c", "color",   "enable color output");
    opts.optflag("d", "debug",   "enable debug output");
    opts.optflag("h", "help",    "print this message");
    opts.optflag("v", "version", "print version");

    let matches = match opts.parse(&args) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };

    if matches.opt_present("h") {
        print_usage(opts);
        return;
    }

    println!("{}", version());
    if matches.opt_present("v") {
        return;
    }
    println!("");

    let mut cli = CLI::new();
    if matches.opt_present("c") {
        cli.game.is_colored = true;
    }
    if matches.opt_present("d") {
        cli.game.is_debug = true;
    }
    if matches.opt_present("t") {
        if let Some(size) = matches.opt_str("t") {
            let memory = size.parse::<usize>().unwrap() << 20;
            cli.game.tt_resize(memory);
        }
    }
    cli.run();
}
