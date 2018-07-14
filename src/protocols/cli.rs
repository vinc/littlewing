use regex::Regex;
use rustyline::Editor;
use time::precise_time_s;

use std::io;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::path::Path;

use color::*;
use common::*;
use attack::Attack;
use clock::Clock;
use eval::Eval;
use fen::FEN;
use game::Game;
use piece_move_generator::PieceMoveGenerator;
use piece_move_notation::PieceMoveNotation;
use protocols::xboard::XBoard;
use protocols::uci::UCI;
use search::Search;

#[derive(Clone)]
pub struct CLI {
    pub game: Game,
    max_depth: Depth,
    play_side: Option<Color>,
    pub show_board: bool
}

impl CLI {
    pub fn new() -> CLI {
        // Load startup position
        let mut game = Game::from_fen(DEFAULT_FEN);

        // Set default clock to 40 moves in 5 minutes
        game.clock = Clock::new(40, 5 * 60 * 1000);

        CLI {
            game,
            max_depth: (MAX_PLY - 10) as Depth,
            play_side: None,
            show_board: false
        }
    }

    pub fn run(&mut self) {
        let mut rl = Editor::<()>::new();

        loop {
            let readline = rl.readline("> ");

            match readline {
                Ok(line) => {
                    rl.add_history_entry(&line);

                    let args: Vec<&str> = line.trim().split(' ').collect();
                    match args[0] {
                        ""                         => (),
                        "quit" | "q" | "exit"      => { break },
                        "help" | "h"               => { self.cmd_usage() },
                        "load" | "l"               => { self.cmd_load(&args) },
                        "save" | "s"               => { self.cmd_save(&args) },
                        "play" | "p" | "go"        => { self.cmd_play(&args) },
                        "hint"                     => { self.cmd_hint() },
                        "eval" | "e"               => { self.cmd_eval() },
                        "undo" | "u"               => { self.cmd_undo() },
                        "move" | "m"               => { self.cmd_move(&args) },
                        "time" | "t" | "level"     => { self.cmd_time(&args) },
                        "show"                     => { self.cmd_config(true, &args) },
                        "hide"                     => { self.cmd_config(false, &args) },
                        "core" | "threads"         => { self.cmd_threads(&args) },
                        "hash" | "memory"          => { self.cmd_memory(&args) },
                        "perft"                    => { self.cmd_perft(&args) },
                        "perftsuite"               => { self.cmd_perftsuite(&args) },
                        "testsuite"                => { self.cmd_testsuite(&args) },
                        "divide"                   => { self.cmd_divide(&args) },
                        "uci"                      => { self.cmd_uci(); break },
                        "xboard"                   => { self.cmd_xboard(); break },
                        _                          => { self.cmd_error(&args); self.cmd_usage() }
                    }
                },
                Err(_) => { break }
            }
        }
    }

    fn cmd_usage(&self) {
        println!();
        println!("Commands:");
        println!();
        println!("  quit                      Exit this program");
        println!("  help                      Display this screen");
        println!("  load [<options>]          Load game from <options>");
        println!("  save [<options>]          Save game to <options>");
        println!("  hint                      Search the best move");
        println!("  play [<color>]            Search and play [<color>] move[s]");
        println!("  undo                      Undo the last move");
        println!("  move <move>               Play <move> on the board");
        println!();
        println!("  show <feature>            Show <feature>");
        println!("  hide <feature>            Hide <feature>");
        println!("  time <moves> <time>       Set clock to <moves> in <time> (in seconds)");
        println!("  hash <size>               Set the <size> of the memory (in MB)");
        println!("  core <number>             Set the <number> of threads");
        println!();
        println!("  perft [<depth>]           Count the nodes at each depth");
        println!("  perftsuite <epd>          Compare perft results to each position of <epd>");
        println!("  testsuite <epd> [<time>]  Search each position of <epd> [for <time>]");
        println!("  divide <depth>            Count the nodes at <depth> for each moves");
        println!();
        println!("  uci                       Start UCI mode");
        println!("  xboard                    Start XBoard mode");
        println!();
        println!("Made with <3 in 2014-2018 by Vincent Ollivier <v@vinc.cc>");
        println!();
        println!("Report bugs to https://github.com/vinc/littlewing/issues");
        println!();
    }

    fn cmd_config_usage(&self, value: bool) {
        let cmds = [
            ["board", "board"],
            ["color", "terminal colors"],
            ["coord", "board coordinates"],
            ["debug", "debug output"],
            ["think", "search output"]
        ];

        println!("Subcommands:");
        println!();
        for args in &cmds {
            if value {
                println!("  show {}     Show {}", args[0], args[1]);
            } else {
                println!("  hide {}     Hide {}", args[0], args[1]);
            }
        }
        println!();
    }

    fn cmd_load_usage(&self) {
        println!("Subcommands:");
        println!();
        println!("  load fen <string>         Load game from FEN <string>");
        //println!("  load pgn <file>           Load game from PGN <file>"); // TODO
        println!();
    }

    fn cmd_save_usage(&self) {
        println!("Subcommands:");
        println!();
        println!("  save fen                  Save game to FEN <string>");
        println!("  save pgn <file>           Save game to PGN <file>");
        println!();
    }

    fn cmd_uci(&self) {
        let mut uci = UCI::new();
        uci.game.is_debug = self.game.is_debug;
        uci.game.threads_count = self.game.threads_count;
        uci.game.tt = self.game.tt.clone();
        uci.run();
    }

    fn cmd_xboard(&self) {
        let mut xboard = XBoard::new();
        xboard.game.is_debug = self.game.is_debug;
        xboard.game.threads_count = self.game.threads_count;
        xboard.game.tt = self.game.tt.clone();
        xboard.run();
    }

    fn cmd_load(&mut self, args: &[&str]) {
        if args.len() == 1 {
            self.print_error(format!("no subcommand given"));
            println!();
            self.cmd_load_usage();
            return;
        }

        match args[1] {
            "fen" => {
                let fen = args[2..].join(" ");
                self.game.load_fen(&fen);
            },
            "pgn" => {
                self.print_error(format!("not implemented yet")); // TODO
                println!();
            }
            _ => {
                self.print_error(format!("unrecognized subcommand '{}'", args[1]));
                println!();
                self.cmd_load_usage();
            }
        }
    }

    fn cmd_save(&mut self, args: &[&str]) {
        if args.len() == 1 {
            self.print_error(format!("no subcommand given"));
            println!();
            self.cmd_save_usage();
            return;
        }

        match args[1] {
            "fen" => {
                println!("{}", self.game.to_fen());
            },
            "pgn" => {
                let starting_fen = self.game.starting_fen.clone();

                if args.len() == 2 {
                    self.print_error(format!("no filename given"));
                    return;
                }
                let path = Path::new(args[2]);
                let mut buffer = File::create(&path).unwrap();

                let mut version = ::version();
                let result = if self.game.is_mate() {
                    if self.game.is_check(WHITE) {
                        "0-1"
                    } else if self.game.is_check(BLACK) {
                        "1-0"
                    } else {
                        "1/2-1/2"
                    }
                } else {
                    "*"
                };

                writeln!(buffer, "[Event \"?\"]").unwrap();
                writeln!(buffer, "[Site \"?\"]").unwrap();
                if self.play_side == Some(WHITE) {
                    writeln!(buffer, "[White \"{}\"]", version).unwrap();
                } else {
                    writeln!(buffer, "[White \"?\"]").unwrap();
                }
                if self.play_side == Some(BLACK) {
                    writeln!(buffer, "[Black \"{}\"]", version).unwrap();
                } else {
                    writeln!(buffer, "[Black \"?\"]").unwrap();
                }
                writeln!(buffer, "[Result \"{}\"]", result).unwrap();
                if starting_fen != String::from(DEFAULT_FEN) {
                    writeln!(buffer, "[FEN \"{}\"]", starting_fen).unwrap();
                    writeln!(buffer, "[SetUp \"1\"]").unwrap();
                }
                writeln!(buffer, "").unwrap();

                let moves = self.game.history.clone();
                self.game.load_fen(&starting_fen);
                let mut first_move = true;
                let mut line = String::new();
                for m in moves {
                    let fm = self.game.positions.fullmoves();
                    if self.game.side() == WHITE {
                        line.push_str(&format!("{}. ", fm));
                    } else if first_move {
                        line.push_str(&format!("{}. ... ", fm));
                    }
                    first_move = false;

                    line.push_str(&self.game.move_to_san(m));

                    self.game.make_move(m);
                    self.game.history.push(m);

                    if self.game.is_mate() {
                        line.push('#');
                    } else if self.game.is_check(self.game.side()) {
                        line.push('+');
                    }

                    if line.len() > 70 {
                        writeln!(buffer, "{}", line).unwrap();
                        line = String::new();
                    } else {
                        line.push(' ');
                    }
                }
                writeln!(buffer, "{}{}", line, result).unwrap();
            }
            _ => {
                self.print_error(format!("unrecognized subcommand '{}'", args[1]));
                println!();
                self.cmd_save_usage();
            }
        }
    }

    fn cmd_config(&mut self, value: bool, args: &[&str]) {
        if args.len() != 2 {
            self.print_error(format!("no subcommand given"));
            println!();
            self.cmd_config_usage(value);
            return;
        }

        match args[1] {
            "board" => {
                self.show_board = value;
                if value {
                    println!();
                    println!("{}", self.game.to_string());
                    println!();
                }
            }
            "color" | "colors" => {
                self.game.is_colored = value;
            }
            "debug" => {
                self.game.is_debug = value;
            }
            "think" | "thinking" => {
                self.game.is_search_verbose = value;
            }
            "coord" | "coords" | "coordinates" => {
                self.game.show_coordinates = value;
            }
            "help" => {
                self.cmd_config_usage(value);
            }
            _ => {
                self.print_error(format!("unrecognized subcommand '{}'", args[1]));
                println!();
                self.cmd_config_usage(value);
            }
        }
    }

    fn cmd_play(&mut self, args: &[&str]) {
        if args.len() > 1 {
            self.play_side = match args[1] {
                "white" => Some(WHITE),
                "black" => Some(BLACK),
                _       => None
            };

            if self.play_side != Some(self.game.side()) {
                return;
            }
        }

        self.think(true);

        if self.game.is_mate() {
            self.print_result(true);
        }
    }

    fn cmd_hint(&mut self) {
        self.think(false);
    }

    fn think(&mut self, play: bool) {
        if self.game.is_debug || self.game.is_search_verbose {
            println!();
        }
        let c = if play { "<" } else { "#" };
        let n = self.max_depth;
        let r = self.game.search(1..n);
        if self.game.is_debug || self.game.is_search_verbose {
            println!();
        }
        if let Some(m) = r {
            println!("{} move {}", c, m.to_can());

            if play {
                self.game.make_move(m);
                self.game.history.push(m);

                if self.show_board {
                    println!();
                    println!("{}", self.game.to_string());
                    println!();
                }
            }
        }
    }

    fn print_result(&self, play: bool) {
        let c = if play { "<" } else { "#" };
        if self.game.is_check(WHITE) {
            println!("{} black mates", c);
        } else if self.game.is_check(BLACK) {
            println!("{} white mates", c);
        } else {
            println!("{} draw", c);
        }
    }

    fn cmd_eval(&mut self) {
        let c = self.game.side();

        println!("Static evaluation of the current position:");
        println!();
        self.game.is_eval_verbose = true;
        self.game.eval();
        self.game.is_eval_verbose = false;
        println!();
        println!("(score in pawn, relative to {})", if c == WHITE { "white" } else { "black"});
    }

    fn cmd_undo(&mut self) {
        if self.game.history.len() > 0 {
            let m = self.game.history.pop().unwrap();
            self.game.undo_move(m);
        }

        if self.show_board {
            println!();
            println!("{}", self.game.to_string());
            println!();
        }
    }

    fn cmd_move(&mut self, args: &[&str]) {
        let re = Regex::new(r"^[a-h][0-9][a-h][0-9][nbrq]?$").unwrap();
        if !re.is_match(args[1]) {
            self.print_error(format!("could not parse move '{}'", args[1]));
            return;
        }
        let parsed_move = self.game.move_from_can(args[1]);

        let mut is_valid = false;
        let side = self.game.side();
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
            println!();
            println!("{}", self.game.to_string());
            if self.play_side == None || (!self.game.is_debug && !self.game.is_search_verbose) {
                println!();
            }
        }

        if self.play_side == Some(self.game.side()) {
            self.think(true);
        }

        if self.game.is_mate() {
            self.print_result(true);
        }
    }

    fn cmd_time(&mut self, args: &[&str]) {
        let moves = args[1].parse::<u16>().unwrap();
        let time = args[2].parse::<f64>().unwrap();
        self.game.clock = Clock::new(moves, (time * 1000.0).round() as u64);
    }

    fn cmd_divide(&mut self, args: &[&str]) {
        self.game.moves.skip_ordering = true;
        self.game.moves.skip_killers = true;
        let mut moves_count = 0u64;
        let mut nodes_count = 0u64;

        if args.len() != 2 {
            self.print_error(format!("no depth given"));
            return;
        }

        let d = args[1].parse::<Depth>().unwrap();

        let side = self.game.side();
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

        println!();
        println!("Moves: {}", moves_count);
        println!("Nodes: {}", nodes_count);
    }

    fn cmd_threads(&mut self, args: &[&str]) {
        self.game.threads_count = args[1].parse::<usize>().unwrap();
    }

    fn cmd_memory(&mut self, args: &[&str]) {
        let memory = args[1].parse::<usize>().unwrap(); // In MB
        self.game.tt_resize(memory << 20);
    }

    fn cmd_perft(&mut self, args: &[&str]) {
        let mut depth = if args.len() == 2 {
            args[1].parse::<Depth>().unwrap()
        } else {
            1
        };

        if self.game.is_debug {
            println!("# FEN {}", self.game.to_fen());
            println!("# starting perft at depth {}", depth);
        }

        self.game.moves.skip_ordering = true;
        self.game.moves.skip_killers = true;

        loop {
            let started_at = precise_time_s();
            let n = self.game.perft(depth);
            let ended_at = precise_time_s();
            let s = ended_at - started_at;
            let nps = (n as f64) / s;
            println!("perft {} -> {} ({:.2} s, {:.2e} nps)", depth, n, s, nps);

            if args.len() == 2 {
                break;
            } else {
                depth += 1;
            }
        }
    }

    fn cmd_perftsuite(&mut self, args: &[&str]) {
        self.game.moves.skip_ordering = true;
        self.game.moves.skip_killers = true;

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
            for field in fields {
                let mut it = field.trim().split(' ');
                let d = it.next().unwrap()[1..].parse::<Depth>().unwrap();
                let n = it.next().unwrap().parse::<u64>().unwrap();
                if self.game.perft(d) == n {
                    print!("{}", self.colorize_green(".".into()));
                    io::stdout().flush().unwrap();
                } else {
                    print!("{}", self.colorize_red("x".into()));
                    break;
                }
            }
            println!();
        }
    }

    fn cmd_testsuite(&mut self, args: &[&str]) {
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
        let mut found_count = 0;
        let mut total_count = 0;
        for line in file.lines() {
            let line = line.unwrap();
            let line = line.split(";").next().unwrap();

            let i = line.find("m ").unwrap() - 1;
            let (fen, rem) = line.split_at(i);
            let (mt, moves) = rem.split_at(2);

            print!("{}{}{} -> ", fen, mt, moves);

            self.game.load_fen(fen);
            self.game.clock = Clock::new(1, time * 1000);

            let n = self.max_depth;
            let best_move = self.game.search(1..n).unwrap();
            let mut best_move_str = self.game.move_to_san(best_move);

            // Add `+` to move in case of check
            let side = self.game.side();
            self.game.make_move(best_move);
            if self.game.is_check(side ^ 1) {
                best_move_str.push('+');
            }
            self.game.undo_move(best_move);

            let found = match mt {
                "bm" => moves.contains(&best_move_str),
                "am" => !moves.contains(&best_move_str),
                _    => unreachable!()
            };
            if found {
                found_count += 1;
                println!("{}", self.colorize_green(best_move_str));
            } else {
                println!("{}", self.colorize_red(best_move_str));
            }
            total_count += 1;
        }
        println!("Result {}/{}", found_count, total_count);
    }

    fn cmd_error(&mut self, args: &[&str]) {
        self.print_error(format!("unrecognized command '{}'", args[0]));
    }

    fn print_error(&self, msg: String) {
        let err = if self.game.is_colored {
            self.colorize_red("error:".into())
        } else {
            "error:".into()
        };
        println!("# {} {}", err, msg);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_undo() {
        let mut cli = CLI::new();

        // Undo 1 move
        cli.cmd_play(&[]);
        cli.cmd_undo();

        // Undo 2 moves
        cli.cmd_play(&[]);
        cli.cmd_play(&[]);
        cli.cmd_undo();
        cli.cmd_undo();

        // Undo 0 moves
        cli.cmd_undo();

        assert!(true);
    }
}
