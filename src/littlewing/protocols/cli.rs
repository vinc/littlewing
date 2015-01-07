extern crate time;

use std::io;
use std::io::BufferedReader;
use std::io::File;

use littlewing::common::*;
use littlewing::attack::Attack;
use littlewing::fen::FEN;
use littlewing::game::Game;
use littlewing::search::Search;

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
                "quit"       => break,
                "setboard"   => self.setboard(args.as_slice()),
                "divide"     => self.divide(args.as_slice()),
                "perft"      => self.perft(),
                "perftsuite" => self.perftsuite(args.as_slice()),
                _            => self.usage()
            }
        }
    }

    pub fn usage(&self) {
        println!("help                  Display this screen");
        println!("setboard <fen>        Set the board to <fen>");
        println!("divide <depth>        Count the nodes at <depth> for each moves");
        println!("perft                 Count the nodes at each depth");
        println!("perftsuite <epd>      Compare perft results to each position of <epd>");
        println!("quit                  Exit this program");
    }

    pub fn setboard(&mut self, args: &[&str]) {
        if args.len() == 1 {
            panic!("no fen given");
        }

        let s = args.slice_from(1).connect(" ");
        let fen = s.as_slice();

        self.game = FEN::from_fen(fen);
    }

    pub fn divide(&mut self, args: &[&str]) {
        let mut moves_count = 0u64;
        let mut nodes_count = 0u64;

        if args.len() != 2 {
            panic!("no depth given");
        }

        let d = args[1].parse::<uint>().unwrap();

        self.game.generate_moves();
        let n = self.game.moves.len();
        for i in range(0u, n) {
            let m = self.game.moves.get(i);
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

    pub fn perft(&mut self) {
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

    pub fn perftsuite(&mut self, args: &[&str]) {
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
