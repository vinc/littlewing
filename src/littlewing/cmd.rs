extern crate time;

use std::io::BufferedReader;
use std::io::File;

use littlewing::common::*;
use littlewing::game::Game;
use littlewing::attack::Attack;

const DEFAULT_FEN: &'static str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

pub fn usage() {
    println!("help                  Display this screen");
    println!("divide <d> [<fen>]    Count the nodes [from <fen>] at <d> for each moves");
    println!("perft [<fen>]         Count the nodes [from <fen>] at each depth");
    println!("perftsuite <epd>      Compare perft results to each position of <epd>");
    println!("quit                  Exit this program");
}

pub fn divide(args: &[&str]) {
    let mut moves_count = 0u64;
    let mut nodes_count = 0u64;

    if args.len() == 1 {
        panic!("no depth given");
    }

    let s = if args.len() == 2 {
        DEFAULT_FEN.to_string()
    } else {
        args.slice_from(2).connect(" ")
    };
    let fen = s.as_slice();
    let d = args[1].parse::<uint>().unwrap();

    let mut game = Game::from_fen(fen);

    game.generate_moves();
    let n = game.moves.len();
    for i in range(0u, n) {
        let m = game.moves.get(i);
        game.make_move(m);
        if !game.is_check() {
            let r = game.perft(d);
            println!("{} {}", m.to_can(), r);
            //println!("{}", game.to_string());
            moves_count += 1;
            nodes_count += r;
        }
        game.undo_move(m);
    }

    println!("");
    println!("Moves: {}", moves_count);
    println!("Nodes: {}", nodes_count);
}

pub fn perft(args: &[&str]) {
    let s = if args.len() == 1 {
        DEFAULT_FEN.to_string()
    } else {
        args.slice_from(1).connect(" ")
    };
    let fen = s.as_slice();
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
            if game.perft(d) == n {
                print!(".");
            } else {
                print!("x");
                break;
            }
        }
        println!("");
    }
}

