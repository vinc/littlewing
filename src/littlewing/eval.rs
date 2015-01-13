use std::num::Int;

use littlewing::game::Game;

pub trait Eval {
    fn eval(&mut self) -> i32;
}

impl Eval for Game {
    fn eval(&mut self) -> i32 {
        let side = self.positions.top().side;
        let mut score = 0;

        score += self.bitboards[side].count_ones() as i32;
        score -= self.bitboards[side ^ 1].count_ones() as i32;

        score
    }
}
