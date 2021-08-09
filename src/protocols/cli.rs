use rustyline::{Context, Editor};
use rustyline::completion::Completer;
use rustyline::error::ReadlineError;
use rustyline_derive::{Helper, Validator, Highlighter, Hinter};

use std::prelude::v1::*;
use std::io;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::error::Error;

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

#[derive(PartialEq)]
enum State {
    Running,
    Stopped
}

impl CLI {
    pub fn new() -> CLI {
        // Load startup position
        let mut game = Game::from_fen(DEFAULT_FEN).unwrap();

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
        // Setup line editor
        let mut rl = Editor::new();
        if let Some(path) = history_path() {
            let _ = rl.load_history(&path);
        }
        let helper = CommandHelper {
            move_params: Vec::new()
        };
        rl.set_helper(Some(helper));

        // Execute commands
        let mut state = State::Running;
        while state != State::Stopped {
            // Add chess moves currently available to the autocomplete
            if let Some(helper) = rl.helper_mut() {
                helper.move_params = self.game.get_moves().into_iter().
                    map(|m| if self.show_san { self.game.move_to_san(m) } else { m.to_lan() }).collect();
            }

            state = match rl.readline(&self.prompt) {
                Ok(line) => {
                    rl.add_history_entry(&line);
                    self.exec(&line)
                },
                Err(_) => {
                    State::Stopped
                }
            };

            if let Some(path) = history_path() {
                if fs::create_dir_all(path.parent().unwrap()).is_ok() {
                    rl.save_history(&path).unwrap();
                }
            }
        }
    }

    fn exec(&mut self, line: &str) -> State {
        let mut state = State::Running;

        // A line can have multiple commands separated by semicolons
        for cmd in line.split(';') {
            if state == State::Stopped {
                break
            }

            let args: Vec<&str> = cmd.trim().split(' ').collect();

            let res = match args[0] {
                "init" | "i"           => self.cmd_init(),
                "load" | "l"           => self.cmd_load(&args),
                "save" | "s"           => self.cmd_save(&args),
                "play" | "p"           => self.cmd_play(&args),
                "hint"                 => self.cmd_hint(),
                "eval" | "e"           => self.cmd_eval(),
                "undo" | "u"           => self.cmd_undo(),
                "move" | "m"           => self.cmd_move(&args),
                "time" | "t" | "level" => self.cmd_time(&args),
                "show"                 => self.cmd_config(true, &args),
                "hide"                 => self.cmd_config(false, &args),
                "core" | "threads"     => self.cmd_threads(&args),
                "hash" | "memory"      => self.cmd_memory(&args),
                "perft"                => self.cmd_perft(&args),
                "perftsuite"           => self.cmd_perftsuite(&args),
                "testsuite"            => self.cmd_testsuite(&args),
                "divide"               => self.cmd_divide(&args),
                "uci"                  => self.cmd_uci(),
                "xboard"               => self.cmd_xboard(),
                "help" | "h"           => self.cmd_usage("help"),
                "quit" | "q" | "exit"  => Ok(State::Stopped),
                ""                     => Ok(State::Running),
                _                      => Err(format!("unknown command '{}'", args[0]).into()),
            };

            state = match res {
                Ok(state) => {
                    state
                },
                Err(e) => {
                    print_error(&e.to_string().to_lowercase());
                    match args[0] {
                        "move" | "m" => Ok(State::Running), // Skip usage on common errors
                        "load" | "l" => self.cmd_load_usage(),
                        "save" | "s" => self.cmd_save_usage(),
                        "show"       => self.cmd_config_usage(true),
                        "hide"       => self.cmd_config_usage(false),
                        _            => self.cmd_usage(args[0]),
                    }.unwrap_or(State::Stopped)
                }
            }
        }

        state
    }

    fn cmd_usage(&self, cmd: &str) -> Result<State, Box<dyn Error>> {
        let lines = vec![
            "",
            "Commands:",
            "",
            "  quit                      Exit this program",
            "  help                      Display this screen",
            "  init                      Initialize a new game",
            "  load <options>            Load game from <options>",
            "  save <options>            Save game to <options>",
            "  hint                      Search the best move",
            "  play [<color>]            Search and play [<color>] move[s]",
            "  undo                      Undo the last move",
            "  move <move>               Play <move> on the board",
            "",
            "  show <feature>            Show <feature>",
            "  hide <feature>            Hide <feature>",
            "  time <moves> <time>       Set clock to <moves> in <time> (in seconds)",
            "  hash <size>               Set the <size> of the memory (in MB)",
            "  core <number>             Set the <number> of threads",
            "",
            "  perft [<depth>]           Count the nodes at each depth",
            "  perftsuite <epd>          Compare perft results to each position of <epd>",
            "  testsuite <epd> [<time>]  Search each position of <epd> [for <time>]",
            "  divide <depth>            Count the nodes at <depth> for each moves",
            "",
            "  uci                       Start UCI mode",
            "  xboard                    Start XBoard mode",
            "",
            "Made with <3 in 2014-2019 by Vincent Ollivier <v@vinc.cc>",
            "",
            "Report bugs to https://github.com/vinc/littlewing/issues",
            "",
        ];
        for line in lines {
            if line.starts_with(&format!("  {} ", cmd)) {
                println!("{}", bold(line));
            } else {
                println!("{}", line);
            }
        }
        Ok(State::Running)
    }

    fn cmd_config_usage(&self, value: bool) -> Result<State, Box<dyn Error>> {
        let cmds = [
            ["board", "board"],
            ["color", "terminal colors"],
            ["coord", "board coordinates"],
            ["debug", "debug output"],
            ["think", "search output"],
            ["san  ", "standard algebraic notation"],
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
        Ok(State::Running)
    }

    fn cmd_load_usage(&self) -> Result<State, Box<dyn Error>> {
        println!();
        println!("Subcommands:");
        println!();
        println!("  load fen <string>         Load game from FEN <string>");
        println!("  load pgn <file>           Load game from PGN <file>");
        println!();
        Ok(State::Running)
    }

    fn cmd_save_usage(&self) -> Result<State, Box<dyn Error>> {
        println!();
        println!("Subcommands:");
        println!();
        println!("  save fen                  Save game to FEN <string>");
        println!("  save pgn <file>           Save game to PGN <file>");
        println!();
        Ok(State::Running)
    }

    fn cmd_uci(&self) -> Result<State, Box<dyn Error>> {
        let mut uci = UCI::new();
        uci.game.is_debug = self.game.is_debug;
        uci.game.threads_count = self.game.threads_count;
        uci.game.tt = self.game.tt.clone();
        uci.run();
        Ok(State::Stopped)
    }

    fn cmd_xboard(&self) -> Result<State, Box<dyn Error>> {
        let mut xboard = XBoard::new();
        xboard.game.is_debug = self.game.is_debug;
        xboard.game.threads_count = self.game.threads_count;
        xboard.game.tt = self.game.tt.clone();
        xboard.run();
        Ok(State::Stopped)
    }

    fn cmd_init(&mut self) -> Result<State, Box<dyn Error>> {
        self.max_depth = (MAX_PLY - 10) as Depth;
        self.game.clear();
        self.game.load_fen(DEFAULT_FEN)?;

        if self.show_board {
            println!();
            println!("{}", self.game);
        }
        Ok(State::Running)
    }

    fn cmd_load(&mut self, args: &[&str]) -> Result<State, Box<dyn Error>> {
        if args.len() == 1 {
            return Err("no subcommand given".into());
        }

        match args[1] {
            "fen" => {
                if args.len() == 2 {
                    return Err("no fen string given".into());
                }
                let fen = args[2..].join(" ");
                self.game.load_fen(&fen)?;
            },
            "pgn" => {
                if args.len() == 2 {
                    return Err("no filename given".into());
                }
                let path = Path::new(args[2]);
                let pgn_str = fs::read_to_string(path)?;
                // TODO: Add cmd arg to select which game to load in PGN file
                // that have more than one game. Right now the last one will
                // be loaded.
                let pgn = PGN::from(pgn_str);
                self.game.load_pgn(pgn);
            }
            "help" => {
                return self.cmd_load_usage();
            }
            _ => {
                return Err(format!("unknown subcommand '{}'", args[1]).into());
            }
        }

        if self.show_board {
            println!();
            println!("{}", self.game);
        }

        Ok(State::Running)
    }

    fn cmd_save(&mut self, args: &[&str]) -> Result<State, Box<dyn Error>> {
        if args.len() == 1 {
            return Err("no subcommand given".into());
        }

        match args[1] {
            "fen" => {
                println!("{}", self.game.to_fen());
            },
            "pgn" => {
                if args.len() == 2 {
                    return Err("no filename given".into());
                }
                let path = Path::new(args[2]);
                let mut buffer = File::create(&path)?;
                let mut pgn = self.game.to_pgn();
                if self.play_side == Some(WHITE) {
                    pgn.set_white(&version());
                }
                if self.play_side == Some(BLACK) {
                    pgn.set_black(&version());
                }
                write!(buffer, "{}", pgn)?;
            }
            "help" => {
                return self.cmd_save_usage();
            }
            _ => {
                return Err(format!("unknown subcommand '{}'", args[1]).into());
            }
        }

        Ok(State::Running)
    }

    fn cmd_config(&mut self, value: bool, args: &[&str]) -> Result<State, Box<dyn Error>> {
        if args.len() != 2 {
            return Err("no subcommand given".into());
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
                colorize(value);
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
                return self.cmd_config_usage(value);
            }
            _ => {
                return Err(format!("unknown subcommand '{}'", args[1]).into());
            }
        }

        Ok(State::Running)
    }

    fn cmd_play(&mut self, args: &[&str]) -> Result<State, Box<dyn Error>> {
        if args.len() > 1 {
            self.play_side = match args[1] {
                "white" => Some(WHITE),
                "black" => Some(BLACK),
                "none"  => None,
                _ => {
                    return Err("<color> should be either 'white', 'black', or 'none'".into());
                }
            };

            if self.play_side != Some(self.game.side()) {
                return Ok(State::Running);
            }
        }

        if self.game.is_debug || self.game.is_search_verbose {
            println!("");
        }

        self.think(true);

        if self.game.is_mate() {
            self.print_result(true);
        }

        Ok(State::Running)
    }

    fn cmd_hint(&mut self) -> Result<State, Box<dyn Error>> {
        self.think(false);
        Ok(State::Running)
    }

    fn cmd_eval(&mut self) -> Result<State, Box<dyn Error>> {
        let c = self.game.side();
        println!("Static evaluation of the current position:");
        println!();
        self.game.is_eval_verbose = true;
        self.game.eval();
        self.game.is_eval_verbose = false;
        println!();
        println!("(score in pawn, relative to {})", if c == WHITE { "white" } else { "black"});
        Ok(State::Running)
    }

    fn cmd_undo(&mut self) -> Result<State, Box<dyn Error>> {
        if self.game.history.len() > 0 {
            if let Some(m) = self.game.history.pop() {
                self.game.undo_move(m);
            }
        }

        if self.show_board {
            println!();
            println!("{}", self.game);
        }
        Ok(State::Running)
    }

    fn cmd_move(&mut self, args: &[&str]) -> Result<State, Box<dyn Error>> {
        if args.len() < 2 {
            return Err("no <move> given".into());
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
                return Err(format!("move '{}' is not valid", args[1]).into());
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
            Ok(State::Running)
        } else {
            Err(format!("could not parse move '{}'", args[1]).into())
        }
    }

    fn cmd_time(&mut self, args: &[&str]) -> Result<State, Box<dyn Error>> {
        match args.len() {
            1 => { return Err("no <moves> and <time> given".into()) },
            2 => { return Err("no <time> given".into()) },
            _ => {}
        }
        let moves = args[1].parse::<u16>()?;
        let time = args[2].parse::<f64>()?;
        self.game.clock = Clock::new(moves, (time * 1000.0).round() as u64);
        Ok(State::Running)
    }

    fn cmd_divide(&mut self, args: &[&str]) -> Result<State, Box<dyn Error>> {
        if args.len() != 2 {
            return Err("no <depth> given".into());
        }
        let d = args[1].parse::<Depth>()?;

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
        Ok(State::Running)
    }

    fn cmd_threads(&mut self, args: &[&str]) -> Result<State, Box<dyn Error>> {
        if args.len() < 2 {
            return Err("no <number> given".into());
        }
        self.game.threads_count = args[1].parse::<usize>()?;
        Ok(State::Running)
    }

    fn cmd_memory(&mut self, args: &[&str]) -> Result<State, Box<dyn Error>> {
        if args.len() < 2 {
            return Err("no <size> given".into());
        }
        let memory = args[1].parse::<usize>()?; // In MB
        self.game.tt_resize(memory << 20);
        Ok(State::Running)
    }

    fn cmd_perft(&mut self, args: &[&str]) -> Result<State, Box<dyn Error>> {
        let mut depth = if args.len() == 2 {
            args[1].parse::<Depth>()?
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
            let started_at = (self.game.clock.system_time)();
            let n = self.game.perft(depth);
            let s = (((self.game.clock.system_time)() - started_at) as f64) / 1000.0;
            let nps = (n as f64) / s;
            println!("perft {} -> {} ({:.2} s, {:.2e} nps)", depth, n, s, nps);

            if args.len() == 2 {
                break;
            } else {
                depth += 1;
            }
        }
        Ok(State::Running)
    }

    fn cmd_perftsuite(&mut self, args: &[&str]) -> Result<State, Box<dyn Error>> {
        self.game.moves.skip_ordering = true;
        self.game.moves.skip_killers = true;

        if args.len() == 1 {
            return Err("no <epd> given".into());
        }
        let path = Path::new(args[1]);
        let file = fs::read_to_string(&path)?;
        for line in file.lines() {
            let mut fields = line.split(';');
            let fen = fields.next().unwrap().trim();
            print!("{} -> ", fen);
            self.game.load_fen(fen)?;
            for field in fields {
                let field = field.trim();
                if !field.starts_with("D") {
                    println!("");
                    return Err("invalid perftsuite epd format".into());
                }
                let mut it = field.split(' ');
                let d = it.next().unwrap()[1..].parse::<Depth>()?;
                let n = it.next().unwrap().parse::<u64>()?;
                if self.game.perft(d) == n {
                    print!("{}", bold_green("."));
                    io::stdout().flush().unwrap();
                } else {
                    print!("{}", bold_red("x"));
                    break;
                }
            }
            println!();
        }
        Ok(State::Running)
    }

    fn cmd_testsuite(&mut self, args: &[&str]) -> Result<State, Box<dyn Error>> {
        if args.len() == 1 {
            return Err("no <epd> given".into());
        }
        let time = if args.len() > 2 {
            args[2].parse::<u64>()? // `time` is given in seconds
        } else {
            10
        };
        let path = Path::new(args[1]);
        let file = fs::read_to_string(&path)?;
        let mut found_count = 0;
        let mut total_count = 0;
        for mut line in file.lines() {
            if let Some(i) = line.find(";") {
                line = &line[0..i];
            }
            if !line.contains(" am ") && !line.contains(" bm ") {
                return Err("invalid testsuite epd format".into());
            }

            let i = line.find("m ").unwrap() - 1;
            let (fen, rem) = line.split_at(i);
            let (mt, moves) = rem.split_at(2);

            print!("{}{}{} -> ", fen, mt, moves);

            self.game.load_fen(fen)?;
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
                println!("{}", bold_green(&best_move_str));
            } else {
                println!("{}", bold_red(&best_move_str));
            }
            total_count += 1;
        }
        println!("Result {}/{}", found_count, total_count);
        Ok(State::Running)
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
}

fn print_error(msg: &str) {
    println!("# {} {}", bold_red("error:"), msg);
}

fn history_path() -> Option<PathBuf> {
    if let Some(data_dir) = dirs::data_dir() {
        Some(data_dir.join("littlewing").join("history"))
    } else {
        None
    }
}

#[derive(Helper, Validator, Highlighter, Hinter)]
struct CommandHelper {
    move_params: Vec<String>
}

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
        cli.cmd_play(&[]).unwrap();
        cli.cmd_undo().unwrap();

        // Undo 2 moves
        cli.cmd_play(&[]).unwrap();
        cli.cmd_play(&[]).unwrap();
        cli.cmd_undo().unwrap();
        cli.cmd_undo().unwrap();

        // Undo 0 moves
        cli.cmd_undo().unwrap();

        assert!(true);
    }

    #[test]
    fn test_divide() {
        let mut cli = CLI::new();

        cli.cmd_divide(&["divide", "2"]).unwrap();
        assert!(true);
    }
}
