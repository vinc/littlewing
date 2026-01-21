use std::prelude::v1::*;
use std::fs;
use std::path::Path;

use crate::color::*;
use crate::piece::*;
use crate::eval::*;
use crate::bitboard::BitboardExt;
use crate::game::Game;
use crate::eval::Eval;
use crate::search::Search;
use crate::fen::FEN;

const P: usize = 0;
const N: usize = 1;
const B: usize = 2;
const R: usize = 3;
const Q: usize = 4;
const BP: usize = 5;
const MAX_PARAMS: usize = 6;

#[derive(Clone)]
pub struct EvaluatedPosition {
    pub trace: Trace,
    pub wdl: f64, // 1.0 = win, 0.5 = draw, 0.0 = loss
}

#[derive(Clone, Default)]
pub struct Trace {
    // Material counts [white, black]
    pub pawns: [i32; 2],
    pub knights: [i32; 2],
    pub bishops: [i32; 2],
    pub rooks: [i32; 2],
    pub queens: [i32; 2],
    pub bishop_pair: [i32; 2],
}

impl Trace {
    pub fn eval(&self, params: &[f64; 6]) -> f64 {
        let mut score = 0.0;
        score += (self.pawns[0] - self.pawns[1]) as f64 * params[P];
        score += (self.knights[0] - self.knights[1]) as f64 * params[N];
        score += (self.bishops[0] - self.bishops[1]) as f64 * params[B];
        score += (self.rooks[0] - self.rooks[1]) as f64 * params[R];
        score += (self.queens[0] - self.queens[1]) as f64 * params[Q];
        score += (self.bishop_pair[0] - self.bishop_pair[1]) as f64 * params[BP];
        score
    }
}

pub struct Tuner {
    pub positions: Vec<EvaluatedPosition>,
    pub params: [f64; MAX_PARAMS],
    pub k: f64, // Scaling constant
}

impl Tuner {
    pub fn new() -> Self {
        let mut params = [0.0; MAX_PARAMS];
        params[P] = PAWN_VALUE as f64;
        params[N] = KNIGHT_VALUE as f64;
        params[B] = BISHOP_VALUE as f64;
        params[R] = ROOK_VALUE as f64;
        params[Q] = QUEEN_VALUE as f64;
        params[BP] = BONUS_BISHOP_PAIR as f64;

        Self {
            positions: Vec::new(),
            params,
            k: 1.0,
        }
    }

    pub fn load_epd(&mut self, path: &Path, game: &mut Game) -> std::io::Result<()> {
        println!("Loading EPD file...");
        let mut loaded = 0;
        let mut skipped = 0;
        let file = fs::read_to_string(path)?;
        for line in file.lines() {
            let args: Vec<_> = line.split(';').collect();
            if args.len() > 1 {
                let fen = args[0].trim();
                let wdl = match args[1].trim() {
                    "1-0"     => 1.0,
                    "1/2-1/2" => 0.5,
                    "0-1"     => 0.0,
                    _         => continue,
                };

                if game.load_fen(fen).is_err() {
                    continue;
                }

                let e = game.eval();
                let q = game.quiescence(e - 1, e + 1, 0, 0);

                if (e - q).abs() < 50 {
                    let trace = self.compute_trace(&game);
                    self.positions.push(EvaluatedPosition { trace, wdl });
                    loaded += 1;
                } else {
                    skipped += 1;
                }
            }
        }
        println!("Loaded {} quiet positions", loaded);
        println!("Skipped {} noisy positions", skipped);
        println!();
        Ok(())
    }

    /// Convert evaluation to win probability using sigmoid
    fn sigmoid(&self, eval: f64) -> f64 {
        1.0 / (1.0 + 10_f64.powf(-self.k * eval / 400.0))
    }

    /// Compute mean squared error (MSE)
    pub fn compute_error(&self) -> f64 {
        let mut total_error = 0.0;

        for pos in &self.positions {
            let eval = pos.trace.eval(&self.params);
            let predicted = self.sigmoid(eval);
            let error = pos.wdl - predicted;
            total_error += error * error;
        }

        total_error / self.positions.len() as f64
    }

    /// Compute gradient of error with respect to parameters
    pub fn compute_gradient(&self) -> [f64; 6] {
        let mut gradient = [0.0; 6];
        let n = self.positions.len() as f64;

        for pos in &self.positions {
            let eval = pos.trace.eval(&self.params);
            let sigmoid = self.sigmoid(eval);

            let error = pos.wdl - sigmoid;
            let dsigmoid_deval = sigmoid * (1.0 - sigmoid) * self.k * 10_f64.ln() / 400.0;
            let coefficient = -2.0 * error * dsigmoid_deval / n;

            // Material gradients
            gradient[0] += coefficient * (pos.trace.pawns[0] - pos.trace.pawns[1]) as f64;
            gradient[1] += coefficient * (pos.trace.knights[0] - pos.trace.knights[1]) as f64;
            gradient[2] += coefficient * (pos.trace.bishops[0] - pos.trace.bishops[1]) as f64;
            gradient[3] += coefficient * (pos.trace.rooks[0] - pos.trace.rooks[1]) as f64;
            gradient[4] += coefficient * (pos.trace.queens[0] - pos.trace.queens[1]) as f64;
            gradient[5] += coefficient * (pos.trace.bishop_pair[0] - pos.trace.bishop_pair[1]) as f64;
        }

        gradient
    }

    /// Tune using gradient descent with Adam optimizer
    pub fn tune(&mut self, iterations: usize, learning_rate: f64) {
        let initial_error = self.compute_error();
        println!("Initial error: {:.6}", initial_error);
        println!();

        // Adam optimizer state
        let mut m = [0.0; 6]; // First moment estimate
        let mut v = [0.0; 6]; // Second moment estimate
        let beta1 = 0.9;
        let beta2 = 0.999;
        let epsilon = 1e-8;

        for iter in 0..iterations {
            let gradient = self.compute_gradient();

            // Update parameters using Adam
            for i in 0..6 {
                m[i] = beta1 * m[i] + (1.0 - beta1) * gradient[i];
                v[i] = beta2 * v[i] + (1.0 - beta2) * gradient[i] * gradient[i];

                let m_hat = m[i] / (1.0 - beta1.powi((iter + 1) as i32));
                let v_hat = v[i] / (1.0 - beta2.powi((iter + 1) as i32));

                self.params[i] -= learning_rate * m_hat / (v_hat.sqrt() + epsilon);
            }

            if (iter + 1) % 50 == 0 {
                let error = self.compute_error();
                println!("Iteration {:3}: error = {:.6}", iter + 1, error);
                self.print_params();
                println!();
            }
        }

        println!("Final tuned parameters:");
        self.print_params();
    }

    /// Find optimal K value for sigmoid function
    pub fn tune_k(&mut self) {
        println!("Tuning K parameter...");

        let mut best_k = self.k;
        let mut best_error = self.compute_error();

        // Try K values from 0.5 to 2.5 in steps of 0.1
        for i in 1..25 {
            self.k = i as f64 / 10.0;
            let error = self.compute_error();
            println!("K = {:.1}: error = {:.6}", self.k, error);

            if error < best_error {
                best_error = error;
                best_k = self.k;
            }
        }

        self.k = best_k;
        println!();
        println!("Optimal K = {:.1}: error = {:.6}", best_k, best_error);
        println!();
    }

    fn compute_trace(&self, game: &Game) -> Trace {
        let mut trace = Trace::default();

        for &c in &COLORS {
            let ci = c as usize;

            trace.pawns[ci] = game.bitboards[(c | PAWN) as usize].count() as i32;
            trace.knights[ci] = game.bitboards[(c | KNIGHT) as usize].count() as i32;
            trace.bishops[ci] = game.bitboards[(c | BISHOP) as usize].count() as i32;
            trace.rooks[ci] = game.bitboards[(c | ROOK) as usize].count() as i32;
            trace.queens[ci] = game.bitboards[(c | QUEEN) as usize].count() as i32;

            // Bishop pair bonus
            if trace.bishops[ci] >= 2 {
                trace.bishop_pair[ci] = 1;
            }
        }

        trace
    }

    pub fn print_params(&self) {
        println!("  PAWN_VALUE:   {:>6.1}", self.params[0]);
        println!("  KNIGHT_VALUE: {:>6.1}", self.params[1]);
        println!("  BISHOP_VALUE: {:>6.1}", self.params[2]);
        println!("  ROOK_VALUE:   {:>6.1}", self.params[3]);
        println!("  QUEEN_VALUE:  {:>6.1}", self.params[4]);
        println!("  BISHOP_PAIR:  {:>6.1}", self.params[5]);
    }
}
