extern crate test;
extern crate littlewing;

use test::Bencher;
use littlewing::game::Game;

#[bench]
fn bench_game_perft(b: &mut Bencher) {
    let game = Game::new();

    b.iter(|| {
        game.perft(5);
    })
}
