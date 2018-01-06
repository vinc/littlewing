use std::io;
use std::thread;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use color::*;
use common::*;
use clock::Clock;
use fen::FEN;
use game::Game;
use piece_move_generator::PieceMoveGenerator;
use search::Search;
use protocols::Protocol;
use version;

pub struct UCI {
    pub game: Game,
    max_depth: Depth,
    searcher: Option<thread::JoinHandle<()>>,
    print_bestmove: Arc<AtomicBool>,
}

impl UCI {
    pub fn new() -> UCI {
        UCI {
            game: Game::from_fen(DEFAULT_FEN),
            max_depth: (MAX_PLY - 10) as Depth,
            searcher: None,
            print_bestmove: Arc::new(AtomicBool::new(false))
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
            io::stdin().read_line(&mut cmd).unwrap();
            let args: Vec<&str> = cmd.trim().split(' ').collect();
            match args[0] {
                "quit"       => break,
                "stop"       => self.cmd_stop(),
                "isready"    => self.cmd_isready(),
                "ucinewgame" => self.cmd_ucinewgame(),
                "position"   => self.cmd_position(&args),
                "go"         => self.cmd_go(&args),
                _            => continue, // Ignore unknown commands
            }
        }
        self.abort_search();
    }

    fn cmd_stop(&mut self) {
        self.stop_search();
    }

    fn cmd_isready(&mut self) {
        println!("readyok");
    }

    fn cmd_ucinewgame(&mut self) {
        self.abort_search();

        self.max_depth = (MAX_PLY - 10) as Depth;
        self.game.clear();
    }

    fn cmd_go(&mut self, args: &[&str]) {
        self.abort_search();

        let side = self.game.positions.top().side;
        let mut is_time = false;
        let mut is_moves = false;
        let mut time = u64::max_value(); // Infinite time
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
        self.print_bestmove.store(true, Ordering::Relaxed);
        self.start_search();
    }

    fn cmd_position(&mut self, args: &[&str]) {
        self.abort_search();

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

    fn start_search(&mut self) {
        let n = self.max_depth;
        let mut game = self.game.clone();
        let print_bestmove = self.print_bestmove.clone();

        let builder = thread::Builder::new().
            name(String::from("searcher")).
            stack_size(4 << 20);

        self.searcher = Some(builder.spawn(move || {
            let res = game.search(1..n);

            if print_bestmove.load(Ordering::Relaxed) {
                match res {
                    Some(m) => println!("bestmove {}", m.to_can()),
                    None    => println!("bestmove 0000")
                }
            }
        }).unwrap());
    }

    fn stop_search(&mut self) {
        self.game.clock.stop();

        // Wait for current search to end
        if let Some(searcher) = self.searcher.take() {
            searcher.join().unwrap();
        }
    }

    fn abort_search(&mut self) {
        self.print_bestmove.store(false, Ordering::Relaxed);
        self.stop_search();
    }
}
