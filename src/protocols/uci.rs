use std::io;

use color::*;
use common::*;
use clock::Clock;
use fen::FEN;
use game::Game;
use moves_generator::MovesGenerator;
use search::Search;
use protocols::Protocol;
use version;

pub struct UCI {
    pub game: Game,
    max_depth: Depth
}

impl UCI {
    pub fn new() -> UCI {
        UCI {
            game: Game::from_fen(DEFAULT_FEN),
            max_depth: (MAX_PLY - 10) as Depth
        }
    }
    pub fn run(&mut self) {
        self.game.protocol = Protocol::UCI;
        self.game.is_verbose = true;
        println!("id name {}", version());
        println!("id author Vincent Ollivier");
        println!("uciok");
        loop {
            let mut line = String::new();
            io::stdin().read_line(&mut line).unwrap();
            let args: Vec<&str> = line.trim().split(' ').collect();
            match args[0] {
                "quit"       => break,
                "isready"    => self.cmd_isready(),
                "ucinewgame" => self.cmd_ucinewgame(),
                "position"   => self.cmd_position(&args),
                "go"         => self.cmd_go(&args),
                _            => panic!("nope nope nope")
            }
        }
    }

    fn cmd_isready(&mut self) {
        println!("readyok");
    }

    fn cmd_ucinewgame(&mut self) {
        self.max_depth = (MAX_PLY - 10) as Depth;
        self.game.clear();
    }

    fn cmd_go(&mut self, args: &[&str]) {
        let side = self.game.positions.top().side;
        let mut is_time = false;
        let mut is_moves = false;
        let mut time = 100000000; // Infinity
        let mut moves = 1;
        for &arg in args {
            match arg {
                "wtime" => {
                    if side == WHITE {
                        is_time = true;
                    }
                },
                "btime" => {
                    if side == BLACK {
                        is_time = true;
                    }
                },
                "movestogo" => {
                    is_moves = true;
                },
                _ => {
                    if is_time {
                        time = arg.parse::<u64>().unwrap();
                        is_time = false;
                    } else if is_moves {
                        moves = args[6].parse::<u16>().unwrap();
                        is_moves = false;
                    }
                }
            }
        }
        self.game.clock = Clock::new(moves, time);
        self.game.clock.disable_level();
        self.think();
    }

    fn cmd_position(&mut self, args: &[&str]) {
        let mut is_fen = false;
        let mut is_move = false;
        let mut fen = Vec::with_capacity(args.len());
        let mut moves = Vec::with_capacity(args.len());
        for &arg in args {
            match arg {
                "startpos" => {
                    fen.push(DEFAULT_FEN);
                },
                "fen" => { // Next args will form the fen string
                    is_fen = true;
                    is_move = false;
                },
                "moves" => { // Next args will form the moves list
                    is_fen = false;
                    is_move = true;
                },
                _ => {
                    if is_fen {
                        fen.push(arg);
                    } else if is_move {
                        moves.push(arg);
                    }
                }
            }
        }

        self.game.load_fen(&fen.join(" "));

        for s in moves {
            let m = self.game.move_from_can(s);
            self.game.make_move(m);
            self.game.history.push(m);
        }
    }

    fn think(&mut self) {
        let n = self.max_depth;
        match self.game.search(1..n) {
            Some(m) => println!("bestmove {}", m.to_can()),
            None    => println!("bestmove 0000")
        }
    }
}
