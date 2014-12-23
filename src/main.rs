use std::io;

fn usage() {
    println!("Usage:");
    println!("quit    exit this program");
}

fn main() {
    println!("Little Wing v0.0.1");
    println!("");

    loop {
        print!("> ");
        let line = io::stdin().read_line().unwrap();
        let cmd = line.as_slice().trim();
        match cmd {
            "quit" => break,
            _      => usage()
        }
    }
}
