extern crate time;

use std::io;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::path::Path;

use common::*;
use attack::Attack;
use clock::Clock;
use fen::FEN;
use game::Game;
use moves_generator::MovesGenerator;
use protocols::xboard::XBoard;
use search::Search;

pub struct CLI {
    game: Game,
    is_colored: bool
}

impl CLI {
    pub fn new(args: Vec<String>) -> CLI {
        let mut is_colored = false;

        for arg in &args {
            match arg.as_str() {
                "-c" | "--color" => { is_colored = true; }
                _                => { }
            }
        }

        CLI {
            game: FEN::from_fen(DEFAULT_FEN),
            is_colored: is_colored
        }
    }
    pub fn run(&mut self) {
        loop {
            print!("> ");
            io::stdout().flush().unwrap();
            let mut line = String::new();
            let _ = io::stdin().read_line(&mut line);
            let args: Vec<&str> = line.trim().split(' ').collect();
            match args[0] {
                "quit"       => { break },
                "setboard"   => { self.cmd_setboard(&*args) },
                "print"      => { self.cmd_print() },
                "divide"     => { self.cmd_divide(&*args) },
                "perft"      => { self.cmd_perft() },
                "perftsuite" => { self.cmd_perftsuite(&*args) },
                "testsuite"  => { self.cmd_testsuite(&*args) },
                "xboard"     => { self.cmd_xboard(); break },
                "help"       => { self.cmd_usage() },
                _            => { self.error(&*args); self.cmd_usage() }
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
        println!("xboard                    Start XBoard mode");
        println!("quit                      Exit this program");
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

        let s = args[1..].join(" ");
        let fen = &*s;
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
        self.game.moves.clear();
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
                    print!("{}", self.colorize_green(".".into()));
                    io::stdout().flush().unwrap();
                } else {
                    print!("{}", self.colorize_red("x".into()));
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
            args[2].parse::<u64>().unwrap() // `time` is given in seconds
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
            self.game.clock = Clock::new(1, time * 1000);

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
                println!("{}", self.colorize_green(best_move_str));
            } else {
                println!("{}", self.colorize_red(best_move_str));
            }
        }
        println!("Result {}/{}", r, n);
    }

    fn colorize_red(&self, text: String) -> String {
        if self.is_colored {
            format!("\x1B[31m{}\x1B[0m", text)
        } else {
            text
        }
    }

    fn colorize_green(&self, text: String) -> String {
        if self.is_colored {
            format!("\x1B[32m{}\x1B[0m", text)
        } else {
            text
        }
    }
}
