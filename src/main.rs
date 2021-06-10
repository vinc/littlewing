extern crate littlewing;
extern crate colored;
extern crate getopts;
extern crate atty;

use std::env;
use atty::Stream;
use getopts::Options;
use colored::Colorize;
use littlewing::protocols::cli::CLI;

fn print_usage(opts: Options) {
    let brief = format!("Usage: littlewing [options]");
    print!("{}", opts.usage(&brief));
}

fn print_banner(mut board: String) {
    let author = "Vincent Ollivier";
    let mut version = littlewing::version();
    println!("                                      _,;");
    println!("               ,       .--.       _,-'.-;");
    println!("                \\`-, <) o  `._ ,-' ,'`_7");
    println!("                <_  `-\\ _       _,' _.'");
    println!("                  <_`\".| `\\    `  _.>");
    println!("                    <_ ;   \\     _>");
    println!("                     `\"     ;  ``");
    if version.len() < 19 {
        version = format!("{}    \\   |   \\", version.bold());
    } else {
        version = format!("{}", version.bold());
    }
    println!("  {}", version);
    println!("                         '|-. _  \\");
    println!("  by {}  _/ /     \\ '.", author.bold());
    board.replace_range(23..35, "\"-\"`---+--'\\_>");
    println!("{}", board);
}

fn main() {
    let mut cli = CLI::new();

    if !atty::is(Stream::Stdout) {
        colored::control::set_override(false);
    }

    let mut opts = Options::new();
    opts.optopt("t",  "tt",      "set transposition table size (in MB)", "SIZE");
    opts.optflag("d", "debug",   "enable debug output");
    opts.optflag("h", "help",    "print this message");
    opts.optflag("s", "silent",  "display less output");
    opts.optflag("v", "version", "print version");

    let args: Vec<String> = env::args().collect();
    let matches = match opts.parse(&args) {
        Ok(m) => { m }
        Err(f) => { panic!("{}", f.to_string()) }
    };

    if matches.opt_present("h") {
        print_usage(opts);
        return;
    }

    if matches.opt_present("v") {
        println!("{}", littlewing::version());
        return;
    }

    if !matches.opt_present("s") {
        cli.show_board = true;
        cli.game.show_coordinates = true;
        print_banner(cli.game.to_string());
    }

    if matches.opt_present("d") {
        cli.game.is_debug = true;
    }

    if matches.opt_present("t") {
        if let Some(size) = matches.opt_str("t") {
            let memory = size.parse::<usize>().unwrap() << 20;
            cli.game.tt_resize(memory);
        }
    }

    cli.run();
}
