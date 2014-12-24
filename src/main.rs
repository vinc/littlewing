extern crate littlewing;

use std::io;

fn cmd_usage() {
    println!("Usage:");
    println!("quit    exit this program");
}

fn cmd_perft() {
    let game = littlewing::Game::new();
    let mut i = 0u;
    loop {
        i += 1;
        let n = game.perft(i);
        println!("perft({}) -> {}", i, n);
    }
}

fn main() {
    println!("Little Wing v0.0.1");
    println!("");

    loop {
        print!("> ");
        let line = io::stdin().read_line().unwrap();
        let cmd = line.as_slice().trim();
        match cmd {
            "quit"  => break,
            "perft" => cmd_perft(),
            _       => cmd_usage()
        }
    }
}
