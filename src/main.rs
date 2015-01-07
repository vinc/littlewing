extern crate littlewing;

use littlewing::protocols::cli::CLI;

fn main() {
    println!("Little Wing v0.0.1");
    println!("");

    let mut cli = CLI::new();
    cli.run();
}
