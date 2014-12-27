extern crate test;
extern crate littlewing;

use test::Bencher;
use littlewing::game::Game;

#[bench]
fn bench_game_perft(b: &mut Bencher) {
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w";
    let mut game = Game::from_fen(fen);

    b.iter(|| {
        game.perft(4);
    })
}

#[bench]
fn bench_game_generate_moves(b: &mut Bencher) {
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w";
    let mut game = Game::from_fen(fen);

    b.iter(|| {
        game.generate_moves();
    })
}
