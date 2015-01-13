extern crate time;

use std::io;
use std::io::BufferedReader;
use std::io::File;

use littlewing::common::*;
use littlewing::attack::Attack;
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
            let line = io::stdin().read_line().unwrap();
            let args: Vec<&str> = line.as_slice().trim().split(' ').collect();
            match args[0].as_slice() {
                "quit"       => { break },
                "setboard"   => { self.cmd_setboard(args.as_slice()) },
                "divide"     => { self.cmd_divide(args.as_slice()) },
                "perft"      => { self.cmd_perft() },
                "perftsuite" => { self.cmd_perftsuite(args.as_slice()) },
                "xboard"     => { self.cmd_xboard(); break },
                "help"       => { self.cmd_usage() },
                _            => { self.error(args.as_slice()); self.cmd_usage() }
            }
        }
    }

    pub fn cmd_usage(&self) {
        println!("help                  Display this screen");
        println!("setboard <fen>        Set the board to <fen>");
        println!("divide <depth>        Count the nodes at <depth> for each moves");
        println!("perft                 Count the nodes at each depth");
        println!("perftsuite <epd>      Compare perft results to each position of <epd>");
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

        let s = args.slice_from(1).connect(" ");
        let fen = s.as_slice();

        self.game = FEN::from_fen(fen);
    }

    pub fn cmd_divide(&mut self, args: &[&str]) {
        let mut moves_count = 0u64;
        let mut nodes_count = 0u64;

        if args.len() != 2 {
            panic!("no depth given");
        }

        let d = args[1].parse::<uint>().unwrap();

        self.game.generate_moves();
        let n = self.game.moves.len();
        for i in range(0, n) {
            let m = self.game.moves[i];
            self.game.make_move(m);
            //println!("{}", game.to_string());
            if !self.game.is_check() {
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
        let mut i = 0u;
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
        let mut file = BufferedReader::new(File::open(&path));
        for line in file.lines() {
            let l = line.unwrap();
            let mut fields = l.split(';');
            let fen = fields.next().unwrap().trim();
            print!("{} -> ", fen);
            self.game = FEN::from_fen(fen);
            for field in fields {
                let mut it = field.trim().split(' ');
                let d = it.next().unwrap().slice_from(1).parse::<uint>().unwrap();
                let n = it.next().unwrap().parse::<u64>().unwrap();
                if self.game.perft(d) == n {
                    print!(".");
                } else {
                    print!("x");
                    break;
                }
            }
            println!("");
        }
    }
}
