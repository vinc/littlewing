extern crate time;

use std::io;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::path::Path;

use regex::Regex;

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
    max_depth: usize,
    show_board: bool
}

impl CLI {
    pub fn new(args: Vec<String>) -> CLI {
        let mut game = Game::from_fen(DEFAULT_FEN);

        for arg in &args {
            match arg.as_str() {
                "-c" | "--color" => { game.is_colored = true; }
                _                => { }
            }
        }

        CLI {
            game: game,
            max_depth: MAX_PLY - 10,
            show_board: false
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
                "help"       => { self.cmd_usage() },
                "play"       => { self.cmd_play() },
                "undo"       => { self.cmd_undo() },
                "move"       => { self.cmd_move(&*args) },
                "time"       => { self.cmd_time(&*args) },
                "show"       => { self.cmd_show(&*args) },
                "hide"       => { self.cmd_hide(&*args) },
                "setboard"   => { self.cmd_setboard(&*args) },
                "perft"      => { self.cmd_perft() },
                "perftsuite" => { self.cmd_perftsuite(&*args) },
                "testsuite"  => { self.cmd_testsuite(&*args) },
                "divide"     => { self.cmd_divide(&*args) },
                "xboard"     => { self.cmd_xboard(); break },
                _            => { self.cmd_error(&*args); self.cmd_usage() }
            }
        }
    }

    pub fn cmd_usage(&self) {
        println!("quit                      Exit this program");
        println!("help                      Display this screen");
        println!("play                      Search and play a move");
        println!("undo                      Undo the last move");
        println!("move <move>               Play <move> on the board");
        println!("show board|think          Show board/think");
        println!("hide board|think          Hide board/think");
        println!("time <moves> <time>       Set clock to <moves> in <time> (in seconds)");
        println!("setboard <fen>            Set the board to <fen>");
        println!("perft                     Count the nodes at each depth");
        println!("perftsuite <epd>          Compare perft results to each position of <epd>");
        println!("testsuite <epd> [<time>]  Search each position of <epd> [for <time>]");
        println!("divide <depth>            Count the nodes at <depth> for each moves");
        println!("xboard                    Start XBoard mode");
    }

    pub fn cmd_xboard(&self) {
        let mut xboard = XBoard::new();
        xboard.run();
    }

    pub fn cmd_setboard(&mut self, args: &[&str]) {
        if args.len() == 1 {
            self.print_error(format!("no fen given"));
            return;
        }

        let s = args[1..].join(" ");
        let fen = &*s;
        self.game.load_fen(fen);
    }

    pub fn cmd_show(&mut self, args: &[&str]) {
        if args.len() != 2 {
            self.print_error(format!("no subcommand given"));
            return;
        }

        match args[1] {
            "board" => { self.show_board = true; }
            "think" => { self.game.is_verbose = true; }
            _       => { self.print_error(format!("unrecognized subcommand '{}'", args[1])); }
        }

        if self.show_board {
            println!("{}", self.game.to_string());
        }
    }

    pub fn cmd_hide(&mut self, args: &[&str]) {
        if args.len() != 2 {
            format!("no subcommand given");
            return;
        }

        match args[1] {
            "board" => { self.show_board = false; }
            "think" => { self.game.is_verbose = false; }
            _       => { self.print_error(format!("unrecognized subcommand '{}'", args[1])); }
        }
    }

    pub fn cmd_play(&mut self) {
        match self.game.root(self.max_depth) {
            None => {
                if self.game.is_check(WHITE) {
                    println!("black mates");
                } else if self.game.is_check(BLACK) {
                    println!("white mates");
                } else {
                    println!("draw");
                }
            },
            Some(m) => {
                self.game.make_move(m);
                self.game.history.push(m);

                println!("move {}", m.to_can());
            }
        }

        if self.show_board {
            println!("{}", self.game.to_string());
        }
    }

    pub fn cmd_undo(&mut self) {
        let m = self.game.history.pop().unwrap();
        self.game.undo_move(m);

        if self.show_board {
            println!("{}", self.game.to_string());
        }
    }

    pub fn cmd_move(&mut self, args: &[&str]) {
        let re = Regex::new(r"^[a-h][0-9][a-h][0-9][nbrq]?$").unwrap();
        if !re.is_match(args[1]) {
            self.print_error(format!("could not parse move '{}'", args[1]));
            return;
        }
        let parsed_move = self.game.move_from_can(&args[1]);

        let mut is_valid = false;
        let side = self.game.positions.top().side;
        self.game.moves.clear();
        while let Some(m) = self.game.next_move() {
            if m == parsed_move {
                self.game.make_move(m);
                if !self.game.is_check(side) {
                    is_valid = true;
                }
                self.game.undo_move(m);
                break;
            }
        }
        if !is_valid {
            self.print_error(format!("move '{}' is not a valid move", args[1]));
            return;
        }

        self.game.make_move(parsed_move);
        self.game.history.push(parsed_move);

        if self.show_board {
            println!("{}", self.game.to_string());
        }
    }

    pub fn cmd_time(&mut self, args: &[&str]) {
        let moves = args[1].parse::<u16>().unwrap();
        let time = args[2].parse::<u64>().unwrap();
        self.game.clock = Clock::new(moves, time * 1000);
    }

    pub fn cmd_divide(&mut self, args: &[&str]) {
        self.game.moves.skip_moves_ordering = true;
        let mut moves_count = 0u64;
        let mut nodes_count = 0u64;

        if args.len() != 2 {
            self.print_error(format!("no depth given"));
            return;
        }

        let d = args[1].parse::<usize>().unwrap();

        let side = self.game.positions.top().side;
        self.game.moves.clear();
        while let Some(m) = self.game.next_move() {
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
        self.game.moves.skip_moves_ordering = true;
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
        if args.len() == 1 {
            self.print_error(format!("no filename given"));
            return;
        }
        let path = Path::new(args[1]);
        let file = BufReader::new(File::open(&path).unwrap());
        for line in file.lines() {
            let l = line.unwrap();
            let mut fields = l.split(';');
            let fen = fields.next().unwrap().trim();
            print!("{} -> ", fen);
            self.game.load_fen(fen);
            self.game.moves.skip_moves_ordering = true;
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
            self.print_error(format!("no filename given"));
            return;
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
            self.game.load_fen(fen);
            self.game.clock = Clock::new(1, time * 1000);

            // TODO: There can be more than one move
            let mut fields = fen.split_whitespace().rev().take(2);
            let move_str = fields.next().unwrap();
            let type_str = fields.next().unwrap();

            let best_move = self.game.root(MAX_PLY).unwrap();
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

    pub fn cmd_error(&mut self, args: &[&str]) {
        self.print_error(format!("unrecognized command '{}'", args[0]));
        println!("");
    }

    fn print_error(&self, msg: String) {
        let err = if self.game.is_colored {
            self.colorize_red("error:".into())
        } else {
            "error:".into()
        };
        println!("{} {}", err, msg);
    }

    fn colorize_red(&self, text: String) -> String {
        if self.game.is_colored {
            format!("\x1B[31m{}\x1B[0m", text)
        } else {
            text
        }
    }

    fn colorize_green(&self, text: String) -> String {
        if self.game.is_colored {
            format!("\x1B[32m{}\x1B[0m", text)
        } else {
            text
        }
    }
}
