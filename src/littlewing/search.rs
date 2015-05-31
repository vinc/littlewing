use littlewing::common::*;
use littlewing::attack::Attack;
use littlewing::eval::Eval;
use littlewing::game::Game;
use littlewing::moves::Move;

pub trait Search {
    fn perft(&mut self, depth: usize) -> u64;
    fn search(&mut self, mut alpha: i32, beta: i32, depth: usize) -> i32;
    fn root(&mut self, max_depth: usize) -> Move;
    fn print_thinking(&mut self, depth: usize, score: i32, m: Move);
}

impl Search for Game {
    fn perft(&mut self, depth: usize) -> u64 {
        if depth == 0 {
            1
        } else {
            let side = self.positions.top().side;
            self.generate_moves();
            let n = self.moves.len();
            let mut r = 0;
            for i in 0..n {
                let m = self.moves[i];
                self.make_move(m);
                if !self.is_check(side) {
                    r += self.perft(depth - 1);
                }
                self.undo_move(m);
            }
            r
        }
    }

    fn search(&mut self, mut alpha: i32, beta: i32, depth: usize) -> i32 {
        if depth <= 0 {
            return self.eval();
        }

        let side = self.positions.top().side;
        self.generate_moves();
        let n = self.moves.len();

        for i in 0..n {
            let m = self.moves[i];
            self.make_move(m);
            if !self.is_check(side) {
                let score = -self.search(-beta, -alpha, depth - 1);
                if score >= beta {
                    self.undo_move(m);
                    return beta;
                }
                if score > alpha {
                    alpha = score;
                }
            }
            self.undo_move(m);
        }

        alpha
    }

    fn root(&mut self, max_depth: usize) -> Move {
        let side = self.positions.top().side;
        self.clock.start();
        self.generate_moves();

        let n = self.moves.len();

        if self.is_verbose {
            println!(" ply  score   time  move");
        }
        let mut best_move = Move::new_null(); // best_move.is_null() == true
        for depth in 1..max_depth {
            if self.clock.poll() {
                break;
            }
            let mut alpha = -INF;
            let beta = INF;
            for i in 0..n {
                let m = self.moves[i];
                self.make_move(m);
                if !self.is_check(side) {
                    let score = -self.search(-beta, -alpha, depth - 1);
                    if score > alpha {
                        if self.is_verbose {
                            self.print_thinking(depth, score, m);
                        }
                        alpha = score;
                        best_move = m;
                    }
                }
                self.undo_move(m);
            }
        }

        best_move
    }

    fn print_thinking(&mut self, depth: usize, score: i32, m: Move) {
        let time = (self.clock.elapsed_time() * 100.0) as u64;

        self.undo_move(m);
        let move_str = self.move_to_san(m);
        self.make_move(m);

        println!(" {:>3}  {:>5}  {:>5}  {}", depth, score, time, move_str);
    }
}

#[cfg(test)]
mod tests {
    use littlewing::common::*;
    use littlewing::fen::FEN;
    use littlewing::game::Game;
    use littlewing::search::Search;

    #[test]
    fn test_perft() {
        let mut game = Game::new();

        // Initial position
        game.load_fen(DEFAULT_FEN);
        assert_eq!(game.perft(1), 20);
        assert_eq!(game.perft(2), 400);
        assert_eq!(game.perft(3), 8902);
        assert_eq!(game.perft(4), 197281);

        let fen = "7k/8/8/p7/1P6/8/8/7K b - - 0 1";
        game.load_fen(fen);
        assert_eq!(game.perft(1), 5);

        let fen = "k6K/8/8/b6b/8/8/8/8 b - - 0 1";
        game.load_fen(fen);
        assert_eq!(game.perft(1), 17);

        let fen = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -";
        game.load_fen(fen);
        assert_eq!(game.perft(1), 14);
        assert_eq!(game.perft(2), 191);
        assert_eq!(game.perft(3), 2812);

        let fen = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1";
        game.load_fen(fen);
        assert_eq!(game.perft(1), 6);
        assert_eq!(game.perft(2), 264);
        assert_eq!(game.perft(3), 9467);

        let fen = "r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1";
        game.load_fen(fen);
        assert_eq!(game.perft(1), 6);
        assert_eq!(game.perft(2), 264);
        assert_eq!(game.perft(3), 9467);

        // Kiwipete position
        let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -";
        game.load_fen(fen);
        assert_eq!(game.perft(1), 48);
        assert_eq!(game.perft(2), 2039);
        assert_eq!(game.perft(3), 97862);

        let fen = "rnbqkb1r/pp1p1ppp/2p5/4P3/2B5/8/PPP1NnPP/RNBQK2R w KQkq - 0 6";
        game.load_fen(fen);
        assert_eq!(game.perft(1), 42);
        assert_eq!(game.perft(2), 1352);
        assert_eq!(game.perft(3), 53392);
    }
}
