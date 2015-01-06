use littlewing::game::Game;
use littlewing::attack::Attack;

pub trait Search {
    fn perft(&mut self, depth: uint) -> u64;
}

impl Search for Game {
    fn perft(&mut self, depth: uint) -> u64 {
        if depth == 0 {
            1
        } else {
            self.generate_moves();
            let n = self.moves.len();
            let mut r = 0;
            for i in range(0u, n) {
                let m = self.moves.get(i);
                self.make_move(m);
                if !self.is_check() {
                    r += self.perft(depth - 1);
                }
                self.undo_move(m);
            }
            r
        }
    }
}
