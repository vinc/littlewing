extern crate time;
extern crate littlewing;

use std::io;
use littlewing::game::Game;

fn cmd_usage() {
    println!("Usage:");
    println!("quit    exit this program");
}

fn cmd_perft() {
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w";
    let mut game = Game::from_fen(fen);
    let mut i = 0u;
    loop {
        i += 1;
        let started_at = time::precise_time_s();
        let n = game.perft(i);
        let ended_at = time::precise_time_s();
        let s = ended_at - started_at;
        let nps = (n as f64) / s;
        println!("perft({}) -> {} ({:.2} s, {:.2e} nps)", i, n, s, nps);
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
