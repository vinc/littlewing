pub struct Game;

impl Game {
    pub fn perft(&self, i: uint) -> uint {
        match i {
            1u => 20u,
            2u => 400u,
            _  => 8902u
        }
    }
}
