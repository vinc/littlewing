use colored::Colorize;
use rustyline::{Context, Editor, Helper};
use rustyline::hint::Hinter;
use rustyline::highlight::Highlighter;
use rustyline::completion::Completer;
use rustyline::error::ReadlineError;
use time::precise_time_s;

use std::io;
use std::fs;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::path::{Path, PathBuf};

use version;
use color::*;
use common::*;
use attack::Attack;
use clock::Clock;
use eval::Eval;
use fen::FEN;
use game::Game;
use piece_move_generator::PieceMoveGenerator;
use piece_move_notation::PieceMoveNotation;
use pgn::*;
use protocols::xboard::XBoard;
use protocols::uci::UCI;
use search::Search;

#[derive(Clone)]
pub struct CLI {
    pub game: Game,
    max_depth: Depth,
    play_side: Option<Color>,
    pub show_board: bool,
    pub show_san: bool,
    pub prompt: String,
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
            show_board: false,
            show_san: true,
            prompt: "> ".to_string(),
        }
    }

    pub fn run(&mut self) {
        let mut rl = Editor::new();
        if let Some(path) = history_path() {
            let _ = rl.load_history(&path);
        }
        let helper = CommandHelper {
            move_params: Vec::new()
        };
        rl.set_helper(Some(helper));

        loop {
            if let Some(helper) = rl.helper_mut() {
                helper.move_params = self.game.get_moves().into_iter().
                    map(|m| if self.show_san { self.game.move_to_san(m) } else { m.to_lan() }).collect();
            }

            match rl.readline(&self.prompt) {
                Ok(line) => {
                    rl.add_history_entry(line.as_str());

                    let args: Vec<&str> = line.trim().split(' ').collect();
                    match args[0] {
                        ""                         => (),
                        "quit" | "q" | "exit"      => { break },
                        "help" | "h"               => { self.cmd_usage() },
                        "init" | "i"               => { self.cmd_init() },
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

            if let Some(path) = history_path() {
                if fs::create_dir_all(path.parent().unwrap()).is_ok() {
                    rl.save_history(&path).unwrap();
                }
            }
        }
    }

    fn cmd_usage(&self) {
        println!();
        println!("Commands:");
        println!();
        println!("  quit                      Exit this program");
        println!("  help                      Display this screen");
        println!("  init                      Initialize a new game");
        println!("  load <options>            Load game from <options>");
        println!("  save <options>            Save game to <options>");
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
        println!("Made with <3 in 2014-2019 by Vincent Ollivier <v@vinc.cc>");
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
            ["think", "search output"],
            ["san", "  standard algebraic notation"],
        ];

        println!();
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
        println!();
        println!("Subcommands:");
        println!();
        println!("  load fen <string>         Load game from FEN <string>");
        println!("  load pgn <file>           Load game from PGN <file>");
        println!();
    }

    fn cmd_save_usage(&self) {
        println!();
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

    fn cmd_init(&mut self) {
        self.max_depth = (MAX_PLY - 10) as Depth;
        self.game.clear();
        self.game.load_fen(DEFAULT_FEN);

        if self.show_board {
            println!();
            println!("{}", self.game);
        }
    }

    fn cmd_load(&mut self, args: &[&str]) {
        if args.len() == 1 {
            print_error("no subcommand given");
            self.cmd_load_usage();
            return;
        }

        match args[1] {
            "fen" => {
                let fen = args[2..].join(" ");
                self.game.load_fen(&fen);
            },
            "pgn" => {
                if args.len() == 2 {
                    print_error("no filename given");
                    return;
                }
                let path = Path::new(args[2]);
                let pgn_str = match fs::read_to_string(path) {
                    Ok(pgn_str) => pgn_str,
                    Err(error) => {
                        print_error(&format!("{}", error).to_lowercase());
                        return;
                    }
                };
                // TODO: Add cmd arg to select which game to load in PGN file
                // that have more than one game. Right now the last one will
                // be loaded.
                let pgn = PGN::from(pgn_str);
                self.game.load_pgn(pgn);
            }
            "help" => {
                self.cmd_load_usage();
                return;
            }
            _ => {
                print_error(&format!("unrecognized subcommand '{}'", args[1]));
                self.cmd_load_usage();
                return;
            }
        }

        if self.show_board {
            println!();
            println!("{}", self.game);
        }
    }

    fn cmd_save(&mut self, args: &[&str]) {
        if args.len() == 1 {
            print_error("no subcommand given");
            self.cmd_save_usage();
            return;
        }

        match args[1] {
            "fen" => {
                println!("{}", self.game.to_fen());
            },
            "pgn" => {
                if args.len() == 2 {
                    print_error("no filename given");
                    return;
                }
                let path = Path::new(args[2]);
                let mut buffer = match File::create(&path) {
                    Ok(buffer) => buffer,
                    Err(error) => {
                        print_error(&format!("{}", error).to_lowercase());
                        return;
                    }
                };

                let mut pgn = self.game.to_pgn();
                if self.play_side == Some(WHITE) {
                    pgn.set_white(&version());
                }
                if self.play_side == Some(BLACK) {
                    pgn.set_black(&version());
                }
                write!(buffer, "{}", pgn).unwrap();
            }
            "help" => {
                self.cmd_save_usage();
            }
            _ => {
                print_error(&format!("unrecognized subcommand '{}'", args[1]));
                self.cmd_save_usage();
            }
        }
    }

    fn cmd_config(&mut self, value: bool, args: &[&str]) {
        if args.len() != 2 {
            print_error("no subcommand given");
            self.cmd_config_usage(value);
            return;
        }

        match args[1] {
            "board" => {
                self.show_board = value;
                if value {
                    println!();
                    println!("{}", self.game);
                }
            }
            "color" | "colors" => {
                colored::control::set_override(value);
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
            "san" => {
                self.show_san = value;
            }
            "help" => {
                self.cmd_config_usage(value);
            }
            _ => {
                print_error(&format!("unrecognized subcommand '{}'", args[1]));
                self.cmd_config_usage(value);
            }
        }
    }

    fn cmd_play(&mut self, args: &[&str]) {
        if args.len() > 1 {
            self.play_side = match args[1] {
                "white" => Some(WHITE),
                "black" => Some(BLACK),
                "none"  => None,
                _ => {
                    print_error("<color> should be either 'white', 'black', or 'none'");
                    self.cmd_usage();
                    return;
                }
            };

            if self.play_side != Some(self.game.side()) {
                return;
            }
        }

        if self.game.is_debug || self.game.is_search_verbose {
            println!("");
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
        let c = if play { "<" } else { "#" };
        let n = self.max_depth;
        let r = self.game.search(1..n);
        if self.game.is_debug || self.game.is_search_verbose {
            println!();
        }
        if let Some(m) = r {
            println!("{} move {}", c, if self.show_san { self.game.move_to_san(m) } else { m.to_lan() });

            if play {
                self.game.make_move(m);
                self.game.history.push(m);

                if self.show_board {
                    println!();
                    println!("{}", self.game);
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
            println!("{}", self.game);
        }
    }

    fn cmd_move(&mut self, args: &[&str]) {
        if args.len() < 2 {
            print_error("no <move> given");
            self.cmd_usage();
            return;
        }
        if let Some(parsed_move) = self.game.parse_move(args[1]) {
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
                print_error(&format!("move '{}' is not valid", args[1]));
                return;
            }

            self.game.make_move(parsed_move);
            self.game.history.push(parsed_move);

            if self.show_board {
                println!();
                println!("{}", self.game);
            } else if self.game.is_debug || self.game.is_search_verbose {
                println!("");
            }

            if self.play_side == Some(self.game.side()) {
                self.think(true);
            }

            if self.game.is_mate() {
                self.print_result(true);
            }
        } else {
            print_error(&format!("could not parse move '{}'", args[1]));
        }
    }

    fn cmd_time(&mut self, args: &[&str]) {
        if args.len() < 3 {
            print_error("no <time> given");
            if args.len() < 2 {
                print_error("no <moves> given");
            }
            self.cmd_usage();
            return;
        }
        let moves = args[1].parse::<u16>().unwrap();
        let time = args[2].parse::<f64>().unwrap();
        self.game.clock = Clock::new(moves, (time * 1000.0).round() as u64);
    }

    fn cmd_divide(&mut self, args: &[&str]) {
        if args.len() != 2 {
            print_error("no <depth> given");
            self.cmd_usage();
            return;
        }
        let d = args[1].parse::<Depth>().unwrap();

        self.game.moves.skip_ordering = true;
        self.game.moves.skip_killers = true;
        let mut moves_count = 0u64;
        let mut nodes_count = 0u64;

        let side = self.game.side();
        self.game.moves.clear();
        while let Some(m) = self.game.next_move() {
            let move_str = if self.show_san { self.game.move_to_san(m) } else { m.to_lan() };
            self.game.make_move(m);
            if !self.game.is_check(side) {
                let r = self.game.perft(d);
                println!("{} {}", move_str, r);
                moves_count += 1;
                nodes_count += r;
            }
            self.game.undo_move(m);
        }

        println!();
        println!("Moves: {}", moves_count);
        println!("Nodes: {}", nodes_count);
    }

    fn cmd_threads(&mut self, args: &[&str]) {
        if args.len() < 2 {
            print_error("no <number> given");
            self.cmd_usage();
            return;
        }
        self.game.threads_count = args[1].parse::<usize>().unwrap();
    }

    fn cmd_memory(&mut self, args: &[&str]) {
        if args.len() < 2 {
            print_error("no <size> given");
            self.cmd_usage();
            return;
        }
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
            print_error("no <epd> given");
            self.cmd_usage();
            return;
        }
        let path = Path::new(args[1]);
        let file = match File::open(&path) {
            Ok(file) => BufReader::new(file),
            Err(error) => {
                print_error(&format!("{}", error).to_lowercase());
                return;
            }
        };
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
                    print!("{}", ".".bold().green());
                    io::stdout().flush().unwrap();
                } else {
                    print!("{}", "x".bold().red());
                    break;
                }
            }
            println!();
        }
    }

    fn cmd_testsuite(&mut self, args: &[&str]) {
        if args.len() == 1 {
            print_error("no <epd> given");
            self.cmd_usage();
            return;
        }
        let time = if args.len() == 3 {
            args[2].parse::<u64>().unwrap() // `time` is given in seconds
        } else {
            10
        };
        let path = Path::new(args[1]);
        let file = match File::open(&path) {
            Ok(file) => BufReader::new(file),
            Err(error) => {
                print_error(&format!("{}", error).to_lowercase());
                return;
            }
        };
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
                println!("{}", best_move_str.bold().green());
            } else {
                println!("{}", best_move_str.bold().red());
            }
            total_count += 1;
        }
        println!("Result {}/{}", found_count, total_count);
    }

    fn cmd_error(&mut self, args: &[&str]) {
        print_error(&format!("unrecognized command '{}'", args[0]));
    }
}

fn print_error(msg: &str) {
    println!("# {} {}", "error:".bold().red(), msg);
}

fn history_path() -> Option<PathBuf> {
    if let Some(data_dir) = dirs::data_dir() {
        Some(data_dir.join("littlewing").join("history"))
    } else {
        None
    }
}

struct CommandHelper {
    move_params: Vec<String>
}

impl Helper for CommandHelper {}
impl Hinter for CommandHelper {}
impl Highlighter for CommandHelper {}
impl Completer for CommandHelper {
    type Candidate = String;

    fn complete(&self, line: &str, _pos: usize, _ctx: &Context<'_>) -> Result<(usize, Vec<String>), ReadlineError> {
        let move_params = self.move_params.iter().map(AsRef::as_ref).collect();
        let play_params = vec!["black", "white", "none"];
        let conf_params = vec!["board", "color", "coord", "debug", "think", "san"];
        let load_params = vec!["fen", "pgn", "help"];
        let save_params = vec!["fen", "pgn", "help"];
        let commands = vec![
            "help", "quit", "init", "load", "save", "play", "hint", "eval",
            "undo", "move", "time", "show", "hide", "core", "hash", "perft",
            "perftsuite", "testsuite", "divide", "xboard", "uci"
        ];

        let mut options = Vec::new();
        options.push(("move", &move_params));
        options.push(("play", &play_params));
        options.push(("show", &conf_params));
        options.push(("hide", &conf_params));
        options.push(("load", &load_params));
        options.push(("save", &save_params));
        options.push(("", &commands));

        let mut candidates = Vec::new();
        for (command, params) in options {
            if line.starts_with(command) {
                for param in params {
                    let command_line = format!("{} {}", command, param);
                    if command_line.trim().starts_with(line) {
                        candidates.push(command_line.trim().to_owned());
                    }
                }
            }
        }
        Ok((0, candidates))
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

    #[test]
    fn test_divide() {
        let mut cli = CLI::new();

        cli.cmd_divide(&["divide", "2"]);
        assert!(true);
    }
}
