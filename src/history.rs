use std::prelude::v1::*;

use crate::common::*;
use crate::game::Game;
use crate::piece_move::PieceMove;

// TODO: Tune this
const HH_BONUS_QUADRA: Score = 16;
const HH_BONUS_LINEAR: Score = 128;
const HH_BONUS_OFFSET: Score = -256;
const HH_BONUS_CLAMP: Score = HH_MAX / 4;

// TODO: Use separate values
const HH_MALUS_QUADRA: Score = HH_BONUS_QUADRA;
const HH_MALUS_LINEAR: Score = HH_BONUS_LINEAR;
const HH_MALUS_OFFSET: Score = HH_BONUS_OFFSET;
//const HH_MALUS_CLAMP: Score = HH_BONUS_CLAMP;

pub trait HistoryHeuristic {
    fn clear_history(&mut self);
    fn get_history(&self, m: PieceMove) -> Score;
    fn set_history(&mut self, m: PieceMove, s: Score);
    fn inc_history(&mut self, m: PieceMove, d: Depth);
    fn dec_history(&mut self, m: PieceMove, d: Depth);
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
        let s = delta(d, HH_BONUS_QUADRA, HH_BONUS_LINEAR, HH_BONUS_OFFSET);
        self.set_history(m, s);
    }

    fn dec_history(&mut self, m: PieceMove, d: Depth) {
        let s = -delta(d, HH_MALUS_QUADRA, HH_MALUS_LINEAR, HH_MALUS_OFFSET);
        self.set_history(m, s);
    }
}

fn delta(depth: Depth, quadra: Score, linear: Score, offset: Score) -> Score {
    let d = depth as i32;
    let x = quadra as i32;
    let y = linear as i32;
    let z = offset as i32;

    (x * d * d + y * d + z).clamp(0, HH_BONUS_CLAMP as i32) as Score
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
