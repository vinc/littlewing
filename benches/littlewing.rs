#![feature(test)]

extern crate test;
extern crate littlewing;

use test::Bencher;

use littlewing::color;
use littlewing::eval::Eval;
use littlewing::fen::FEN;
use littlewing::game::Game;
use littlewing::piece_move_generator::PieceMoveGenerator;
use littlewing::piece_move_notation::PieceMoveNotation;
use littlewing::search::Search;

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

#[bench]
fn bench_eval(b: &mut Bencher) {
    let game = Game::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");

    b.iter(|| {
        game.eval()
    })
}

#[bench]
fn bench_eval_material(b: &mut Bencher) {
    let game = Game::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");

    b.iter(|| {
        game.eval_material(color::WHITE)
    })
}

#[bench]
fn bench_see(b: &mut Bencher) {
    let mut game = Game::from_fen("rnbqkb1r/pp2pppp/2p2n2/1B1p4/4P3/2N5/PPPP1PPP/R1BQK1NR w KQkq - 0 4");
    let m = game.move_from_can("c2d5");

    b.iter(|| {
        game.see(m)
    })
}

#[bench]
fn bench_search(b: &mut Bencher) {
    let mut game = Game::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");

    b.iter(|| {
        game.search(1..5)
    })
}

#[bench]
fn bench_perft(b: &mut Bencher) {
    let mut game = Game::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    b.iter(|| {
        game.perft(3)
    });
}
