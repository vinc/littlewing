extern crate time;

use std::io;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::path::Path;

use littlewing::common::*;
use littlewing::attack::Attack;
use littlewing::clock::Clock;
use littlewing::fen::FEN;
use littlewing::game::Game;
use littlewing::search::Search;
use littlewing::protocols::xboard::XBoard;

pub struct CLI {
    game: Game
}

impl CLI {
    pub fn new() -> CLI {
        CLI {
            game: FEN::from_fen(DEFAULT_FEN)
        }
    }
    pub fn run(&mut self) {
        loop {
            print!("> ");
            io::stdout().flush().unwrap();
            let mut line = String::new();
            io::stdin().read_line(&mut line);
            let args: Vec<&str> = line.trim().split(' ').collect();
            match args[0] {
                "quit"       => { break },
                "setboard"   => { self.cmd_setboard(args.as_slice()) },
                "print"      => { self.cmd_print() },
                "divide"     => { self.cmd_divide(args.as_slice()) },
                "perft"      => { self.cmd_perft() },
                "perftsuite" => { self.cmd_perftsuite(args.as_slice()) },
                "testsuite"  => { self.cmd_testsuite(args.as_slice()) },
                "xboard"     => { self.cmd_xboard(); break },
                "help"       => { self.cmd_usage() },
                _            => { self.error(args.as_slice()); self.cmd_usage() }
            }
        }
    }

    pub fn cmd_usage(&self) {
        println!("help                      Display this screen");
        println!("setboard <fen>            Set the board to <fen>");
        println!("print                     Print the board");
        println!("divide <depth>            Count the nodes at <depth> for each moves");
        println!("perft                     Count the nodes at each depth");
        println!("perftsuite <epd>          Compare perft results to each position of <epd>");
        println!("testsuite <epd> [<time>]  Search each position of <epd> [for <time>]");
        println!("xboard                Start XBoard mode");
        println!("quit                  Exit this program");
    }

    pub fn error(&mut self, args: &[&str]) {
        println!("Unrecognized command '{}'", args[0]);
        println!("");
    }

    pub fn cmd_xboard(&self) {
        let mut xboard = XBoard::new();
        xboard.run();
    }

    pub fn cmd_setboard(&mut self, args: &[&str]) {
        if args.len() == 1 {
            panic!("no fen given");
        }

        let s = args[1..].connect(" ");
        let fen = s.as_str();
        self.game = FEN::from_fen(fen);
    }

    pub fn cmd_print(&self) {
        println!("{}", self.game.to_string());
    }

    pub fn cmd_divide(&mut self, args: &[&str]) {
        let mut moves_count = 0u64;
        let mut nodes_count = 0u64;

        if args.len() != 2 {
            panic!("no depth given");
        }

        let d = args[1].parse::<usize>().unwrap();

        let side = self.game.positions.top().side;
        self.game.generate_moves();
        let n = self.game.moves.len();
        for i in 0..n {
            let m = self.game.moves[i];
            self.game.make_move(m);
            //println!("{}", game.to_string());
            if !self.game.is_check(side) {
                let r = self.game.perft(d);
                println!("{} {}", m.to_can(), r);
                moves_count += 1;
                nodes_count += r;
            } else {
                //println!("{} (illegal)", m.to_can());
            }
            self.game.undo_move(m);
        }

        println!("");
        println!("Moves: {}", moves_count);
        println!("Nodes: {}", nodes_count);
    }

    pub fn cmd_perft(&mut self) {
        let mut i = 0;
        loop {
            i += 1;
            let started_at = time::precise_time_s();
            let n = self.game.perft(i);
            let ended_at = time::precise_time_s();
            let s = ended_at - started_at;
            let nps = (n as f64) / s;
            println!("perft({}) -> {} ({:.2} s, {:.2e} nps)", i, n, s, nps);
        }
    }

    pub fn cmd_perftsuite(&mut self, args: &[&str]) {
        if args.len() != 2 {
            panic!("no filename given");
        }
        let path = Path::new(args[1]);
        let file = BufReader::new(File::open(&path).unwrap());
        for line in file.lines() {
            let l = line.unwrap();
            let mut fields = l.split(';');
            let fen = fields.next().unwrap().trim();
            print!("{} -> ", fen);
            self.game = FEN::from_fen(fen);
            for field in fields {
                let mut it = field.trim().split(' ');
                let d = it.next().unwrap()[1..].parse::<usize>().unwrap();
                let n = it.next().unwrap().parse::<u64>().unwrap();
                if self.game.perft(d) == n {
                    print!(".");
                    io::stdout().flush().unwrap();
                } else {
                    print!("x");
                    break;
                }
            }
            println!("");
        }
    }

    pub fn cmd_testsuite(&mut self, args: &[&str]) {
        if args.len() == 1 {
            panic!("no filename given");
        }
        let time = if args.len() == 3 {
            args[2].parse::<u16>().unwrap()
        } else {
            10
        };
        let path = Path::new(args[1]);
        let file = BufReader::new(File::open(&path).unwrap());
        let mut r = 0;
        let mut n = 0;
        for line in file.lines() {
            n += 1;
            let l = line.unwrap();
            let mut fields = l.split(';');
            let fen = fields.next().unwrap().trim();
            print!("{} -> ", fen);
            self.game = FEN::from_fen(fen);
            self.game.clock = Clock::new(1, time);

            // TODO: There can be more than one move
            let mut fields = fen.split_whitespace().rev().take(2);
            let move_str = fields.next().unwrap();
            let type_str = fields.next().unwrap();

            let best_move = self.game.root(MAX_PLY);
            let best_move_str = self.game.move_to_san(best_move);
            let found = match type_str {
                "bm" => move_str == best_move_str,
                "am" => move_str != best_move_str,
                _    => false
            };
            if found {
                r += 1;
            }
            print!("{}", best_move_str);
            println!("");
        }
        println!("Result {}/{}", r, n);
    }
}
