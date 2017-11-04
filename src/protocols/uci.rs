use std::io;
use std::thread;

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
    max_depth: Depth,
    commands: Vec<String>
}

impl UCI {
    pub fn new() -> UCI {
        UCI {
            game: Game::from_fen(DEFAULT_FEN),
            max_depth: (MAX_PLY - 10) as Depth,
            commands: Vec::new()
        }
    }
    pub fn run(&mut self) {
        self.game.protocol = Protocol::UCI;
        self.game.is_search_verbose = true;
        println!("id name {}", version());
        println!("id author Vincent Ollivier");
        println!("uciok");
        loop {
            let mut cmd = String::new();
            if self.commands.is_empty() {
                io::stdin().read_line(&mut cmd).unwrap();
            } else {
                // For commands received while thinking
                cmd = self.commands.pop().unwrap();
            }
            let args: Vec<&str> = cmd.trim().split(' ').collect();
            match args[0] {
                "quit"       => break,
                "isready"    => self.cmd_isready(),
                "ucinewgame" => self.cmd_ucinewgame(),
                "position"   => self.cmd_position(&args),
                "go"         => self.cmd_go(&args),
                _            => continue, // Ignore unknown commands
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
                        moves = arg.parse::<u16>().unwrap();
                        is_moves = false;
                    }
                }
            }
        }
        // FIXME: time increment is ignored
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
        // Searcher thread
        let n = self.max_depth;
        let mut game = self.game.clone();
        let builder = thread::Builder::new().
            name(String::from("searcher")).
            stack_size(4 << 20);
        let searcher = builder.spawn(move || {
            game.search(1..n)
        }).unwrap();

        // Stopper thread
        let mut game = self.game.clone();
        let builder = thread::Builder::new().
            name(String::from("stopper")).
            stack_size(4 << 20);
        let stopper = builder.spawn(move || {
            loop {
                let mut cmd = String::new();
                io::stdin().read_line(&mut cmd).unwrap();
                match cmd.trim() {
                    "isready" => {
                        println!("readyok");
                    },
                    "stop" => {
                        game.clock.stop();
                        return cmd;
                    },
                    _ => {
                        return cmd;
                    }
                }
            }
        }).unwrap();

        let best_move = searcher.join().unwrap();
        match best_move {
            Some(m) => println!("bestmove {}", m.to_can()),
            None    => println!("bestmove 0000")
        }

        // If the stopper thread receives a `stop` command it will stop the
        // searcher thread, otherwise it will keep listening until the next
        // command sent to the engine, and this command must be sent back to
        // the main input loop.
        let cmd = stopper.join().unwrap();
        self.commands.push(cmd);
    }
}
