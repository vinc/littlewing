use common::*;
use attack::Attack;
use eval::Eval;
use game::Game;
use moves::Move;

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
        if self.clock.poll(self.nodes_count) {
            return 0;
        }

        if depth == 0 {
            return self.eval();
        }

        let side = self.positions.top().side;
        self.nodes_count += 1;
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
        self.nodes_count = 0;
        self.moves.clear_all();
        self.clock.start(self.positions.len());
        self.generate_moves();

        let n = self.moves.len();

        if self.is_verbose {
            println!(" ply  score   time  nodes  pv");
        }
        let mut best_move = Move::new_null(); // best_move.is_null() == true
        for depth in 1..max_depth {
            let mut bm = Move::new_null(); // Best move at the current depth
            let mut alpha = -INF;
            let beta = INF;
            for i in 0..n {

                if self.clock.poll(self.nodes_count) {
                    break; // Discard search at this depth if time is out
                }

                let m = self.moves[i];
                self.make_move(m);
                let score = -self.search(-beta, -alpha, depth - 1);
                if !self.is_check(side) {
                    if score > alpha {
                        if self.is_verbose && self.nodes_count > 1000 {
                            // We skip the first thousand nodes to gain time
                            // TODO: do we need this?
                            self.print_thinking(depth, score, m);
                        }
                        alpha = score;
                        bm = m;
                    }
                }
                self.undo_move(m);
            }

            // Save the best move only if we found one and if we still have
            // some time left after the search at this depth.
            if !bm.is_null() && !self.clock.poll(self.nodes_count) {
                best_move = bm;
            }
        }

        //println!("DEBUG: used  {} ms to move", self.clock.elapsed_time());
        best_move
    }

    fn print_thinking(&mut self, depth: usize, score: i32, m: Move) {
        let time = self.clock.elapsed_time() / 10; // In centiseconds

        self.undo_move(m);
        let move_str = self.move_to_san(m);
        self.make_move(m);

        println!(" {:>3}  {:>5}  {:>5}  {:>5}  {}", depth, score, time, self.nodes_count, move_str);
    }
}

#[cfg(test)]
mod tests {
    use common::*;
    use fen::FEN;
    use game::Game;
    use search::Search;
    use moves::Move;

    use clock::Clock;
    use eval;

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

    #[test]
    fn test_search() {
        let mut game = Game::new();
        let fen = "4k3/8/4q3/8/8/4Q3/8/4K3 w - - 0 1";
        game.load_fen(fen);

        game.nodes_count = 0;
        game.clock = Clock::new(1, 5 * 1000); // 5 seconds
        self.clock.start(game.positions.len());

        let alpha = -INF;
        let beta = INF;

        for depth in 0..5 {
            let score = game.search(alpha, beta, depth);
            if depth == 0 {
                assert!(score == 0);
            } else {
                assert!(score >= eval::QUEEN_VALUE);
            }
        }
    }

    #[test]
    fn test_root() {
        let mut game = Game::new();
        let fen = "2k4r/ppp3pp/8/2b2p1P/PPP2p2/N4P2/3r2K1/1q5R w - - 4 29";
        game.load_fen(fen);
        //game.is_verbose = true;

        let max_depth = 10;
        let best_move = Move::new(G2, H3, QUIET_MOVE);
        let m = game.root(max_depth);
        assert_eq!(m.to_string(), best_move.to_string());
    }
}
