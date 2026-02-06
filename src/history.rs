use std::prelude::v1::*;

use crate::common::*;
use crate::game::Game;
use crate::piece_move::PieceMove;

// TODO: Tune this
const HH_BONUS_SCALE: Score = 300;
const HH_BONUS_OFFSET: Score = 250;
const HH_BONUS_CLAMP: Score = HH_MAX;

// TODO: Tune this
const HH_MALUS_SCALE: Score = 300;
const HH_MALUS_OFFSET: Score = 250;
const HH_MALUS_CLAMP: Score = HH_MAX;

pub trait HistoryHeuristic {
    fn clear_history(&mut self);
    fn get_history(&self, m: PieceMove) -> Score;
    fn inc_history(&mut self, m: PieceMove, d: Depth);
    fn dec_history(&mut self, m: PieceMove, d: Depth);
    fn set_history(&mut self, m: PieceMove, s: Score);
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

    fn inc_history(&mut self, m: PieceMove, d: Depth) {
        let d = d as i32;
        let x = HH_BONUS_SCALE as i32;
        let y = HH_BONUS_OFFSET as i32;
        let z = HH_BONUS_CLAMP as i32;
        let bonus = (d * x - y).clamp(0, z);
        //let bonus = (d * d).min(z);

        // Gravity formula
        let old = self.get_history(m) as i32;
        let new = old + bonus - old * bonus / (HH_MAX as i32);
        debug_assert!(new < Score::MAX as i32);
        debug_assert!(new <= HH_MAX as i32);

        self.set_history(m, new as Score);
    }

    fn dec_history(&mut self, m: PieceMove, d: Depth) {
        let d = d as i32;
        let x = HH_MALUS_SCALE as i32;
        let y = HH_MALUS_OFFSET as i32;
        let z = HH_MALUS_CLAMP as i32;
        let malus = -(d * x - y).clamp(0, z);
        //let malus = -(d * d).min(z);

        // Gravity formula
        let old = self.get_history(m) as i32;
        let new = old + malus - old * malus.abs() / (HH_MAX as i32);
        debug_assert!(new < Score::MAX as i32);
        debug_assert!(new > Score::MIN as i32);
        debug_assert!(new <= HH_MAX as i32);
        debug_assert!(new >= -HH_MAX as i32);
        
        self.set_history(m, new as Score);
    }

    fn set_history(&mut self, m: PieceMove, s: Score) {
        let a = m.to() as usize;
        let b = m.from() as usize;
        let c = self.side() as usize;
        self.history[c][b][a] = s;
    }
}
