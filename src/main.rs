extern crate littlewing;

use std::io;

use littlewing::protocols::cli::CLI;

fn main() {
    println!("Little Wing v0.0.1");
    println!("");

    let mut cli = CLI::new();
    loop {
        print!("> ");
        let line = io::stdin().read_line().unwrap();
        let args: Vec<&str> = line.as_slice().trim().split(' ').collect();
        match args[0].as_slice() {
            "quit"       => break,
            "setboard"   => cli.setboard(args.as_slice()),
            "divide"     => cli.divide(args.as_slice()),
            "perft"      => cli.perft(),
            "perftsuite" => cli.perftsuite(args.as_slice()),
            _            => cli.usage()
        }
    }
}
