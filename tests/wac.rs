extern crate littlewing;

use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;

use littlewing::clock::Clock;
use littlewing::fen::FEN;
use littlewing::game::Game;
use littlewing::piece_move_generator::PieceMoveGenerator;
use littlewing::search::Search;

#[test]
fn test_wac() {
    let mut game = Game::new();

    // Test some WAC positions
    let path = Path::new("tests/wac.epd");
    let file = BufReader::new(File::open(&path).unwrap());
    let lines = [1, 3, 4, 5, 6, 7, 8, 9, 37, 199, 255];
    let mut l = 0;
    for line in file.lines() {
        l += 1;
        if !lines.contains(&l) {
            continue;
        }
        let line = line.unwrap();
        let line = line.split(";").next().unwrap();

        let i = line.find("m ").unwrap() - 1;
        let (fen, rem) = line.split_at(i);
        let (mt, moves) = rem.split_at(2); // Extract the best move list
        assert_eq!(mt, "bm");

        game.load_fen(fen);
        game.clock = Clock::new(1, 1000); // search for 1 second

        let m = game.search(1..99).unwrap();
        let best_move = game.move_to_san(m);

        println!("{} <- {}", moves, best_move);

        assert!(moves.contains(&best_move));
    }
}
