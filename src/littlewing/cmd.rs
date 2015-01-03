extern crate time;

use std::io::BufferedReader;
use std::io::File;

use littlewing::common::*;
use littlewing::game::Game;

pub fn usage() {
    println!("help                  Display this screen");
    println!("perft                 Count the nodes at each depth from the starting position");
    println!("perftsuite <epd>      Compare perft results to each position of <epd>");
    println!("quit                  Exit this program");
}

pub fn perft() {
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

pub fn perftsuite(args: &[&str]) {
    if args.len() != 2 {
        panic!("no filename given");
    }
    let path = Path::new(args[1]);
    let mut file = BufferedReader::new(File::open(&path));
    for line in file.lines() {
        let l = line.unwrap();
        let mut fields = l.split(';');
        let fen = fields.next().unwrap().trim();
        print!("{} -> ", fen);
        let mut game = Game::from_fen(fen);
        for field in fields {
            let mut it = field.trim().split(' ');
            let d = it.next().unwrap().slice_from(1).parse::<uint>().unwrap();
            let n = it.next().unwrap().parse::<u64>().unwrap();
            if d > 3 { break }
            if game.perft(d) == n {
                print!(".");
            } else {
                print!("x");
            }
        }
        println!("");
    }
}

