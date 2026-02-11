use std::prelude::v1::*;

use crate::common::*;
use crate::game::Game;
use crate::piece_move::PieceMove;

pub trait HistoryHeuristic {
    fn clear_history(&mut self);
    fn get_history(&self, m: PieceMove) -> Score;
    fn set_history(&mut self, m: PieceMove, s: Score);
    fn inc_history(&mut self, m: PieceMove, d: Depth);
    fn dec_history(&mut self, m: PieceMove, d: Depth);
    fn delta(&self, depth: Depth, quadra: i32, linear: i32, offset: i32) -> Score;
}

impl HistoryHeuristic for Game {
    fn clear_history(&mut self) {
        self.history = [[[0; 64]; 64]; 2];
    }

    fn get_history(&self, m: PieceMove) -> Score {
        let a = m.to() as usize;
        let b = m.from() as usize;
        let c = self.side() as usize;
        self.history[c][b][a]
    }

    fn set_history(&mut self, m: PieceMove, s: Score) {
        // Gravity formula
        let val = s as i32;
        let max = HH_MAX as i32;
        let old = self.get_history(m) as i32;
        let new = old + val - old * val.abs() / max;
        debug_assert!(new < Score::MAX as i32);
        debug_assert!(new > Score::MIN as i32);
        debug_assert!(new <= HH_MAX as i32);
        debug_assert!(new >= -HH_MAX as i32);

        let a = m.to() as usize;
        let b = m.from() as usize;
        let c = self.side() as usize;
        self.history[c][b][a] = new as Score;
    }

    fn inc_history(&mut self, m: PieceMove, d: Depth) {
        let quadra = self.params.hhb_quadra.val;
        let linear = self.params.hhb_linear.val;
        let offset = self.params.hhb_offset.val;
        let s = self.delta(d, quadra, linear, offset);
        self.set_history(m, s);
    }

    fn dec_history(&mut self, m: PieceMove, d: Depth) {
        let quadra = self.params.hhm_quadra.val;
        let linear = self.params.hhm_linear.val;
        let offset = self.params.hhm_offset.val;
        let s = -self.delta(d, quadra, linear, offset);
        self.set_history(m, s);
    }

    fn delta(&self, depth: Depth, quadra: i32, linear: i32, offset: i32) -> Score {
        let d = depth as i32;
        let x = quadra;
        let y = linear;
        let z = offset;

        (x * d * d + y * d + z).clamp(0, self.params.hh_clamp.val) as Score
    }
}

#[cfg(test)]
mod tests {
    use std::prelude::v1::*;
    use crate::common::*;
    use crate::fen::FEN;
    use crate::game::Game;
    use crate::piece_move_notation::PieceMoveNotation;
    use super::*;

    #[test]
    fn test_delta() {
        let mut game = Game::new();
        game.load_fen(DEFAULT_FEN).unwrap();

        let bonuses = [0, 64, 272, 512, 784, 1088, 1424, 1792, 2192, 2624];
        for (depth, bonus) in bonuses.iter().enumerate() {
            assert_eq!(delta((depth + 1) as Depth, 16, 128, -256), *bonus);
        }
    }

    #[test]
    fn test_update_history() {
        let mut game = Game::new();
        game.load_fen(DEFAULT_FEN).unwrap();
        let m = game.move_from_lan("e2e4");
        assert_eq!(game.get_history(m), 0);

        for value in [512, 1008, 1489, 1955, 2406, 2843] {
            game.set_history(m, 512);
            assert_eq!(game.get_history(m), value);
        }

        for value in [2243, 1661, 1098, 552, 23, -489] {
            game.set_history(m, -512);
            assert_eq!(game.get_history(m), value);
        }
    }

    #[test]
    fn test_clear_history() {
        let mut game = Game::new();
        game.load_fen(DEFAULT_FEN).unwrap();
        let m = game.move_from_lan("e2e4");
        assert_eq!(game.get_history(m), 0);

        game.set_history(m, 512);
        assert_eq!(game.get_history(m), 512);

        game.clear_history();
        assert_eq!(game.get_history(m), 0);
    }
}
