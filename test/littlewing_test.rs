extern crate littlewing;

#[test]
fn game_fen_test() {
    let fens = [
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR",
        "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR",
        "rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R"
    ];
    for &fen in fens.iter() {
        let game = littlewing::Game::from_fen(fen);
        assert!(game.to_fen().as_slice() == fen);
    }
}

#[test]
fn game_perft_test() {
    let game = littlewing::Game::new();
    assert!(game.perft(1) == 20u);
    assert!(game.perft(2) == 400u);
    assert!(game.perft(3) == 8902u);
}
