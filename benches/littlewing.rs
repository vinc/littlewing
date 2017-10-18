#![feature(test)]

extern crate test;
extern crate littlewing;

use test::Bencher;

use littlewing::fen::FEN;
use littlewing::game::Game;
use littlewing::moves_generator::MovesGenerator;
use littlewing::search::Search;

#[bench]
fn bench_perft(b: &mut Bencher) {
    let mut game = Game::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    b.iter(|| {
        game.perft(3)
    });
}

#[bench]
fn bench_next_move(b: &mut Bencher) {
    let mut game = Game::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");

    b.iter(|| {
        let mut n = 0;
        game.moves.clear();
        while let Some(_) = game.next_move() {
            n += 1;
        }
        n
    })
}

#[bench]
fn bench_next_move_without_ordering(b: &mut Bencher) {
    let mut game = Game::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    game.moves.skip_ordering = true;

    b.iter(|| {
        let mut n = 0;
        game.moves.clear();
        while let Some(_) = game.next_move() {
            n += 1;
        }
        n
    })
}

#[bench]
fn bench_make_undo_move(b: &mut Bencher) {
    let mut game = Game::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    let m = game.move_from_can("e2e4");

    b.iter(|| {
        game.make_move(m);
        game.undo_move(m);
    })
}
