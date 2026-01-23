use std::prelude::v1::*;
use std::fs;
use std::path::Path;

use crate::color::*;
use crate::piece::*;
use crate::eval::*;
use crate::bitboard::{BitboardExt, BitboardIterator};
use crate::game::Game;
use crate::eval::Eval;
use crate::search::Search;
use crate::fen::FEN;
use crate::piece_square_table::PST;

const P: usize = 0;
const N: usize = 1;
const B: usize = 2;
const R: usize = 3;
const Q: usize = 4;
const BP: usize = 5;
const PST_INDEX: usize = 6;
const MAX_PARAMS: usize = 6 + (6 * 64 * 2);

#[derive(Clone)]
pub struct EvaluatedPosition {
    pub trace: Trace,
    pub wdl: f64, // 1.0 = win, 0.5 = draw, 0.0 = loss
}

#[derive(Clone)]
pub struct Trace {
    // Material counts [white, black]
    pub pawns: [i32; 2],
    pub knights: [i32; 2],
    pub bishops: [i32; 2],
    pub rooks: [i32; 2],
    pub queens: [i32; 2],
    pub bishop_pair: [i32; 2],

    // PST: [kind][square][color]
    pub pst: [[[i32; 2]; 64]; 6],

    pub piece_count: i32,
}

impl Trace {
    pub fn eval(&self, params: &[f64; MAX_PARAMS]) -> f64 {
        let mut score = 0.0;
        score += (self.pawns[0] - self.pawns[1]) as f64 * params[P];
        score += (self.knights[0] - self.knights[1]) as f64 * params[N];
        score += (self.bishops[0] - self.bishops[1]) as f64 * params[B];
        score += (self.rooks[0] - self.rooks[1]) as f64 * params[R];
        score += (self.queens[0] - self.queens[1]) as f64 * params[Q];
        score += (self.bishop_pair[0] - self.bishop_pair[1]) as f64 * params[BP];

        // PST with phase interpolation
        let x = self.piece_count as f64;
        let x0 = 32.0; // Opening (max pieces)
        let x1 = 2.0;  // Endgame (min pieces)

        for kind in 0..6 {
            for sq in 0..64 {
                for c in 0..2 {
                    if self.pst[kind][sq][c] > 0 {
                        // Get PST parameters for opening and endgame
                        let pst_idx_opening = PST_INDEX + kind * 64 * 2 + sq;
                        let pst_idx_endgame = pst_idx_opening + 64;

                        let opening_val = params[pst_idx_opening];
                        let endgame_val = params[pst_idx_endgame];

                        // Linear interpolation
                        let interpolated = (opening_val * (x1 - x) + endgame_val * (x - x0)) / (x1 - x0);

                        let count = self.pst[kind][sq][c] as f64;

                        if c == 0 {
                            score += count * interpolated;
                        } else {
                            score -= count * interpolated;
                        }
                    }
                }
            }
        }

        score
    }
}

impl Default for Trace {
    fn default() -> Self {
        Self {
            pawns: [0; 2],
            knights: [0; 2],
            bishops: [0; 2],
            rooks: [0; 2],
            queens: [0; 2],
            bishop_pair: [0; 2],
            pst: [[[0; 2]; 64]; 6],
            piece_count: 0,
        }
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

        for kind in 0..6 {
            let piece = (kind + 1) * 2;
            for phase in 0..2 {
                let offset = PST_INDEX + kind * 64 * 2 + phase * 64;
                for sq in 0..64 {
                    params[offset + sq] = PST[piece][sq][phase] as f64;
                }
            }
        }

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

                game.tt.clear();
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
    pub fn compute_gradient(&self) -> [f64; MAX_PARAMS] {
        let mut gradient = [0.0; MAX_PARAMS];
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


            // PST gradients
            let x = pos.trace.piece_count as f64;
            let x0 = 32.0;
            let x1 = 2.0;

            for kind in 0..6 {
                for sq in 0..64 {
                    for c in 0..2 {
                        let count = pos.trace.pst[kind][sq][c] as f64;
                        if count == 0.0 {
                            continue;
                        }

                        let pst_idx_opening = PST_INDEX + kind * 64 * 2 + sq;
                        let pst_idx_endgame = pst_idx_opening + 64;

                        let sign = if c == 0 { 1.0 } else { -1.0 };

                        // d(interpolated)/d(opening) = (x1 - x) / (x1 - x0)
                        gradient[pst_idx_opening] += coefficient * sign * count * (x1 - x) / (x1 - x0);

                        // d(interpolated)/d(endgame) = (x - x0) / (x1 - x0)
                        gradient[pst_idx_endgame] += coefficient * sign * count * (x - x0) / (x1 - x0);
                    }
                }
            }
        }

        gradient
    }

    /// Tune using gradient descent with Adam optimizer
    pub fn tune(&mut self, iterations: usize, learning_rate: f64) {
        let initial_error = self.compute_error();
        println!("Tuning {} parameters... (iterations={}, learning_rate={})", MAX_PARAMS, iterations, learning_rate);
        println!();
        println!("Iteration,Error");
        println!("{},{:.6}", 0, initial_error);

        // Adam optimizer state
        let mut m = [0.0; MAX_PARAMS]; // First moment estimate
        let mut v = [0.0; MAX_PARAMS]; // Second moment estimate
        let beta1 = 0.9;
        let beta2 = 0.999;
        let epsilon = 1e-8;

        for iter in 0..iterations {
            let gradient = self.compute_gradient();

            // Update parameters using Adam
            for i in 1..MAX_PARAMS {
                m[i] = beta1 * m[i] + (1.0 - beta1) * gradient[i];
                v[i] = beta2 * v[i] + (1.0 - beta2) * gradient[i] * gradient[i];

                let m_hat = m[i] / (1.0 - beta1.powi((iter + 1) as i32));
                let v_hat = v[i] / (1.0 - beta2.powi((iter + 1) as i32));

                self.params[i] -= learning_rate * m_hat / (v_hat.sqrt() + epsilon);
            }

            if (iter + 1) % 50 == 0 {
                let error = self.compute_error();
                println!("{},{:.6}", iter + 1, error);
            }
        }
        println!();
    }

    /// Find optimal K value for sigmoid function
    pub fn tune_k(&mut self) {
        println!("Tuning K parameter...");
        println!();
        println!("K,Error");

        let mut best_k = self.k;
        let mut best_error = self.compute_error();

        // Try K values from 0.0 to 2.5 in steps of 0.1
        for i in 0..25 {
            self.k = i as f64 / 10.0;
            let error = self.compute_error();
            println!("{:.1},{:.6}", self.k, error);

            if error < best_error {
                best_error = error;
                best_k = self.k;
            }
        }

        self.k = best_k;
        println!();
        println!("Optimal K={:.1} (error={:.6})", best_k, best_error);
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

            // Trace PST for each piece
            for &p in &PIECES {
                let mut pieces = game.bitboards[(c | p) as usize];
                let kind = (p as usize / 2) - 1;

                while let Some(sq) = pieces.next() {
                    trace.pst[kind][sq as usize][ci] = 1;
                    trace.piece_count += 1;
                }
            }
        }

        trace
    }

    pub fn print_params(&self) {
        println!("Result:");
        println!();
        println!("pub const PAWN_VALUE:   Score = {:>6.0}", self.params[0]);
        println!("pub const KNIGHT_VALUE: Score = {:>6.0}", self.params[1]);
        println!("pub const BISHOP_VALUE: Score = {:>6.0}", self.params[2]);
        println!("pub const ROOK_VALUE:   Score = {:>6.0}", self.params[3]);
        println!("pub const QUEEN_VALUE:  Score = {:>6.0}", self.params[4]);
        println!("pub const BISHOP_PAIR:  Score = {:>6.0}", self.params[5]);

        let piece_names = ["PAWN", "KNIGHT", "BISHOP", "ROOK", "QUEEN", "KING"];
        let phase_names = ["OPENING", "ENDGAME"];
        for kind in 0..6 {
            for phase in 0..2 {
                println!();
                println!(
                    "const {}_{}: [Score; 64] = [",
                    piece_names[kind],
                    phase_names[phase],
                );
                let offset = PST_INDEX + kind * 64 * 2 + phase * 64;
                for i in 0..64 {
                    print!("{:>4.0}, ", self.params[offset + i]);
                    if i % 8 == 7 {
                        println!();
                    }
                }
                println!("];");
            }
        }
    }
}
