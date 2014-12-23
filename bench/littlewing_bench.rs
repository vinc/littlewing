extern crate littlewing;
extern crate test;

use test::Bencher;

#[bench]
fn game_perft_bench(b: &mut Bencher) {
    let game = littlewing::Game;

    b.iter(|| {
        game.perft(5);
    })
}
