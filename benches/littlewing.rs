extern crate littlewing;
extern crate criterion;

use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;

use littlewing::color;
use littlewing::eval::Eval;
use littlewing::fen::FEN;
use littlewing::game::Game;
use littlewing::piece_move_generator::PieceMoveGenerator;
use littlewing::piece_move_notation::PieceMoveNotation;
use littlewing::search::Search;

fn bench_next_move(c: &mut Criterion) {
    let mut game = Game::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();

    c.bench_function("Game next move", |b| b.iter(|| {
        let mut n = 0;
        game.moves.clear();
        while let Some(_) = game.next_move() {
            n += 1;
        }
        n
    }));
}

fn bench_next_move_without_ordering(c: &mut Criterion) {
    let mut game = Game::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
    game.moves.skip_ordering = true;

    c.bench_function("Game next move without ordering", |b| b.iter(|| {
        let mut n = 0;
        game.moves.clear();
        while let Some(_) = game.next_move() {
            n += 1;
        }
        n
    }));
}

fn bench_eval_material(c: &mut Criterion) {
    let game = Game::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();

    c.bench_function("Game eval material", |b| b.iter(||
        game.eval_material(color::WHITE)
    ));
}

fn bench_make_undo_move(c: &mut Criterion) {
    let mut game = Game::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
    let m = game.move_from_lan("e2e4");

    c.bench_function("Game make/undo move", |b| b.iter(|| {
        game.make_move(black_box(m));
        game.undo_move(black_box(m));
    }));
}

fn bench_eval(c: &mut Criterion) {
    let game = Game::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();

    c.bench_function("Game eval", |b| b.iter(||
        game.eval()
    ));
}

fn bench_see(c: &mut Criterion) {
    let mut game = Game::from_fen("rnbqkb1r/pp2pppp/2p2n2/1B1p4/4P3/2N5/PPPP1PPP/R1BQK1NR w KQkq - 0 4").unwrap();
    let m = game.move_from_lan("c2d5");

    c.bench_function("Game see", |b| b.iter(||
        game.see(black_box(m))
    ));
}

fn bench_search(c: &mut Criterion) {
    let mut game = Game::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();

    c.bench_function("Game search", |b| b.iter(||
        game.search(black_box(1..5))
    ));
}

fn bench_perft(c: &mut Criterion) {
    let mut game = Game::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();

    c.bench_function("Game perft", |b| b.iter(||
        game.perft(black_box(3))
    ));
}

fn bench_move_from_lan(c: &mut Criterion) {
    let mut game = Game::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
    c.bench_function("Game move from LAN", |b| b.iter(||
        game.move_from_lan(black_box("e2e4"))
    ));
}

fn bench_move_from_san(c: &mut Criterion) {
    let mut game = Game::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
    c.bench_function("Game move from SAN", |b| b.iter(||
        game.move_from_san(black_box("e4"))
    ));
}

fn bench_tt_16mb_get(c: &mut Criterion) {
    let mut game = Game::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
    let m = game.move_from_lan("e2e4");
    game.tt_resize(16 << 20); // 16 MB
    game.search(1..5);
    game.make_move(m);
    let hash = game.positions.top().hash;
    c.bench_function("Game TT get (16 MB)", |b| b.iter(|| {
        if let Some(t) = game.tt.get(black_box(hash)) {
            t.score()
        } else {
            0
        }
    }));
}

fn bench_tt_256mb_get(c: &mut Criterion) {
    let mut game = Game::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
    let m = game.move_from_lan("e2e4");
    game.tt_resize(256 << 20); // 256 MB
    game.search(1..5);
    game.make_move(m);
    let hash = game.positions.top().hash;
    c.bench_function("Game TT get (256 MB)", |b| b.iter(|| {
        if let Some(t) = game.tt.get(black_box(hash)) {
            t.score()
        } else {
            0
        }
    }));
}

criterion_group!(benches,
    bench_next_move,
    bench_next_move_without_ordering,
    bench_eval_material,
    bench_make_undo_move,
    bench_eval,
    bench_see,
    bench_search,
    bench_perft,
    bench_move_from_lan,
    bench_move_from_san,
    bench_tt_16mb_get,
    bench_tt_256mb_get,
);
criterion_main!(benches);
