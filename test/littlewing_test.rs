extern crate littlewing;

#[test]
fn game_perft_test() {
    let game = littlewing::Game;
    assert!(game.perft(1) == 20u);
    assert!(game.perft(2) == 400u);
    assert!(game.perft(3) == 8902u);
    assert!(game.perft(4) == 197281u);
}
