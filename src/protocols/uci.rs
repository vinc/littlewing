use std::prelude::v1::*;
use std::io;
use std::thread;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::color::*;
use crate::common::*;
use crate::clock::Clock;
use crate::fen::FEN;
use crate::game::Game;
use crate::piece_move_generator::PieceMoveGenerator;
use crate::piece_move_notation::PieceMoveNotation;
use crate::search::Search;
use crate::protocols::Protocol;
use crate::version;

#[derive(PartialEq)]
enum PositionArg { Cmd, Fen, Moves }

pub struct UCI {
    pub game: Game,
    max_depth: Depth,
    searcher: Option<thread::JoinHandle<()>>,
    print_bestmove: Arc<AtomicBool>,
}

impl UCI {
    pub fn new() -> UCI {
        UCI {
            game: Game::from_fen(DEFAULT_FEN).unwrap(),
            max_depth: (MAX_PLY - 10) as Depth,
            searcher: None,
            print_bestmove: Arc::new(AtomicBool::new(false))
        }
    }
    pub fn run(&mut self) {
        self.game.protocol = Protocol::UCI;
        self.game.is_search_verbose = true;
        self.cmd_uci();
        loop {
            let mut cmd = String::new();
            io::stdin().read_line(&mut cmd).unwrap();
            let args: Vec<&str> = cmd.trim().split(' ').collect();
            match args[0] {
                "quit"       => break,
                "uci"        => self.cmd_uci(),
                "stop"       => self.cmd_stop(),
                "debug"      => self.cmd_debug(&args),
                "setoption"  => self.cmd_setoption(&args),
                "isready"    => self.cmd_isready(),
                "ucinewgame" => self.cmd_ucinewgame(),
                "position"   => self.cmd_position(&args),
                "go"         => self.cmd_go(&args),
                _            => continue, // Ignore unknown commands
            }
        }
        self.abort_search();
    }

    fn cmd_uci(&mut self) {
        println!("id name {}", version());
        println!("id author Vincent Ollivier");
        println!("option name Threads type spin default 1 min 1 max 64");
        println!("option name Hash type spin default 8 min 1 max 16384");

        let params = [
            ("HistoryHeuristicBonusQuadratic", &self.game.params.hhb_quadra),
            ("HistoryHeuristicBonusLinear", &self.game.params.hhb_linear),
            ("HistoryHeuristicBonusOffset", &self.game.params.hhb_offset),
            ("HistoryHeuristicMalusQuadratic", &self.game.params.hhm_quadra),
            ("HistoryHeuristicMalusLinear", &self.game.params.hhm_linear),
            ("HistoryHeuristicMalusOffset", &self.game.params.hhm_offset),
            ("HistoryHeuristicClamp", &self.game.params.hh_clamp),
            ("DeltaPruningMargin", &self.game.params.dp_margin),
            ("FutilityPruningMargin", &self.game.params.fp_margin),
            ("LateMoveReductionHistoryMargin", &self.game.params.lmr_hm),
            ("LateMoveReductionMinimum", &self.game.params.lmr_min),
            ("LateMoveReductionDivisor", &self.game.params.lmr_div),
        ];
        for (label, param) in params {
            println!(
                "option name {} type spin default {} min {} max {}",
                label, param.val, param.min, param.max,
            );
        }

        println!("uciok");
    }

    fn cmd_setoption(&mut self, args: &[&str]) {
        let mut name = "";
        let mut i = 0;
        let n = args.len();
        while i < n {
            match args[i] {
                "name" if i + 1 < n => {
                    i += 1;
                    name = args[i];
                },
                "value" if i + 1 < n => {
                    i += 1;
                    match name {
                        "Threads" => {
                            self.game.threads_count = args[i].parse().unwrap();
                        },
                        "Hash" => {
                            let size = args[i].parse::<usize>().unwrap(); // MB
                            self.game.tt_resize(size << 20);
                        },
                        "HistoryHeuristicBonusQuadratic" => {
                            self.game.params.hhb_quadra.val = args[i].parse().unwrap();
                        },
                        "HistoryHeuristicBonusLinear" => {
                            self.game.params.hhb_linear.val = args[i].parse().unwrap();
                        },
                        "HistoryHeuristicBonusOffset" => {
                            self.game.params.hhb_offset.val = args[i].parse().unwrap();
                        },
                        "HistoryHeuristicMalusQuadratic" => {
                            self.game.params.hhm_quadra.val = args[i].parse().unwrap();
                        },
                        "HistoryHeuristicMalusLinear" => {
                            self.game.params.hhm_linear.val = args[i].parse().unwrap();
                        },
                        "HistoryHeuristicMalusOffset" => {
                            self.game.params.hhm_offset.val = args[i].parse().unwrap();
                        },
                        "HistoryHeuristicClamp" => {
                            self.game.params.hh_clamp.val = args[i].parse().unwrap();
                        },
                        "DeltaPruningMargin" => {
                            self.game.params.dp_margin.val = args[i].parse().unwrap();
                        },
                        "FutilityPruningMargin" => {
                            self.game.params.fp_margin.val = args[i].parse().unwrap();
                        },
                        "LateMoveReductionHistoryMargin" => {
                            self.game.params.lmr_hm.val = args[i].parse().unwrap();
                        },
                        "LateMoveReductionMinimum" => {
                            self.game.params.lmr_min.val = args[i].parse().unwrap();
                            self.game.params.compute_lmr();
                        },
                        "LateMoveReductionDivisor" => {
                            self.game.params.lmr_div.val = args[i].parse().unwrap();
                            self.game.params.compute_lmr();
                        },
                        _ => {}
                    }
                },
                _ => {}
            }
            i += 1;
        }
    }

    fn cmd_stop(&mut self) {
        self.stop_search();
    }

    fn cmd_debug(&mut self, args: &[&str]) {
        match args.get(1) {
            Some(&"on") => self.game.is_debug = true,
            Some(&"off") => self.game.is_debug = false,
            _ => {},
        }
    }

    fn cmd_isready(&mut self) {
        println!("readyok");
    }

    fn cmd_ucinewgame(&mut self) {
        self.abort_search();

        self.max_depth = (MAX_PLY - 10) as Depth;
        self.game.clear();
        self.game.tt.clear();
    }

    fn cmd_go(&mut self, args: &[&str]) {
        self.abort_search();
        let side = self.game.side();
        let mut moves = 0;
        let mut time = 0;
        let mut time_increment = 0;
        let mut i = 0;
        let n = args.len();
        while i < n {
            match args[i] {
                "infinite" => {
                    time = u64::MAX;
                },
                "wtime" if i + 1 < n => {
                    i += 1;
                    if side == WHITE {
                        time = args[i].parse().unwrap();
                    }
                },
                "btime" if i + 1 < n => {
                    i += 1;
                    if side == BLACK {
                        time = args[i].parse().unwrap();
                    }
                },
                "winc" if i + 1 < n => {
                    i += 1;
                    if side == WHITE {
                        time_increment = args[i].parse().unwrap();
                    }
                },
                "binc" if i + 1 < n => {
                    i += 1;
                    if side == BLACK {
                        time_increment = args[i].parse().unwrap();
                    }
                },
                "movetime" if i + 1 < n => {
                    i += 1;
                    time = args[i].parse().unwrap();
                },
                "movestogo" if i + 1 < n => {
                    i += 1;
                    moves = args[i].parse().unwrap();
                },
                _ => {}
            }
            i += 1;
        }
        self.game.clock = Clock::new(moves, time);
        self.game.clock.set_time_increment(time_increment);
        self.print_bestmove.store(true, Ordering::Relaxed);
        self.start_search();
    }

    fn cmd_position(&mut self, args: &[&str]) {
        self.abort_search();

        let mut next = PositionArg::Cmd;
        let mut fen = Vec::with_capacity(args.len());
        let mut moves = Vec::with_capacity(args.len());
        for &arg in args {
            match arg {
                "startpos" => fen.push(DEFAULT_FEN),
                "fen" => next = PositionArg::Fen,
                "moves" => next = PositionArg::Moves,
                _ if next == PositionArg::Fen => fen.push(arg),
                _ if next == PositionArg::Moves => moves.push(arg),
                _ => {},
            }
        }

        self.game.load_fen(&fen.join(" ")).unwrap();

        for s in moves {
            let m = self.game.move_from_lan(s);
            self.game.make_move(m);
            self.game.plies.push(m);
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
                    Some(m) => println!("bestmove {}", m.to_lan()),
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
