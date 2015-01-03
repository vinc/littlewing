extern crate littlewing;

use std::io;

use littlewing::cmd;

fn main() {
    println!("Little Wing v0.0.1");
    println!("");

    loop {
        print!("> ");
        let line = io::stdin().read_line().unwrap();
        let args: Vec<&str> = line.as_slice().trim().split(' ').collect();
        match args[0].as_slice() {
            "quit"       => break,
            "divide"     => cmd::divide(args.as_slice()),
            "perft"      => cmd::perft(),
            "perftsuite" => cmd::perftsuite(args.as_slice()),
            _            => cmd::usage()
        }
    }
}
