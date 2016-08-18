use common::*;
use attack::Attack;
use eval::Eval;
use fen::FEN;
use game::Game;
use moves::{Move, MovesState};
use moves_generator::MovesGenerator;

pub trait Search {
    fn perft(&mut self, depth: usize) -> u64;
    fn quiescence(&mut self, mut alpha: Score, beta: Score, ply: usize) -> Score;
    fn search(&mut self, mut alpha: Score, beta: Score, depth: usize, ply: usize) -> Score;
    fn root(&mut self, max_depth: usize) -> Option<Move>;
    fn print_thinking(&mut self, depth: usize, score: Score, m: Move);
    fn get_pv(&mut self, depth: usize) -> String;
}

impl Search for Game {
    fn perft(&mut self, depth: usize) -> u64 {
        if depth == 0 {
            1
        } else {
            let side = self.positions.top().side;
            self.moves.clear();
            let mut r = 0;
            while let Some(m) = self.next_move() {
                self.make_move(m);
                if !self.is_check(side) {
                    r += self.perft(depth - 1);
                }
                self.undo_move(m);
            }
            r
        }
    }

    fn quiescence(&mut self, mut alpha: Score, beta: Score, ply: usize) -> Score {
        if self.clock.poll(self.nodes_count) {
            return 0
        }

        let stand_path = self.eval();
        if ply >= MAX_PLY {
            return stand_path;
        }
        if stand_path >= beta {
            return beta
        }
        if alpha < stand_path {
            alpha = stand_path;
        }

        let side = self.positions.top().side;

        self.moves.clear();
        while let Some(m) = self.next_move() {
            if !m.is_capture() {
                continue;
            }

            let old_fen = self.to_fen();
            self.make_move(m);

            if self.is_check(side) {
                self.undo_move(m);
                let new_fen = self.to_fen();
                debug_assert_eq!(old_fen, new_fen);
                continue;
            }
            self.nodes_count += 1;

            let score = -self.quiescence(-beta, -alpha, ply + 1);

            self.undo_move(m);
            let new_fen = self.to_fen();
            debug_assert_eq!(old_fen, new_fen);

            if score >= beta {
                return beta
            }
            if alpha < score {
                alpha = score;
            }
        }

        alpha
    }

    fn search(&mut self, mut alpha: Score, beta: Score, depth: usize, ply: usize) -> Score {
        if self.clock.poll(self.nodes_count) {
            return 0;
        }

        if depth == 0 {
            //return self.eval();
            return self.quiescence(alpha, beta, ply + 1);
        }

        let hash = self.positions.top().hash;
        let side = self.positions.top().side;

        // Detect draw by threefold repetitions and fifty-moves rule
        if self.positions.is_draw() {
            return 0;
        }

        let is_in_check = self.is_check(side);
        let mut has_legal_moves = false;


        let mut best_move = match self.tt.get(&hash) {
            None    => Move::new_null(),
            Some(t) => {
                if t.depth() >= depth { // This node has already been searched
                    return t.score()
                }

                t.best_move()
            }
        };

        self.moves.clear();
        if !best_move.is_null() {
            self.moves.add_move(best_move, MovesState::BestMove);
        }

        while let Some(m) = self.next_move() {
            self.make_move(m);

            if self.is_check(side) {
                self.undo_move(m);
                continue;
            }

            self.nodes_count += 1;
            has_legal_moves = true;

            let score = -self.search(-beta, -alpha, depth - 1, ply + 1);

            self.undo_move(m);

            if score >= beta {
                return beta;
            }

            if score > alpha {
                alpha = score;
                best_move = m;
            }
        }

        // TODO: could we just use `best_move.is_null()` ?
        if !has_legal_moves { // End of game
            if is_in_check {
                return -INF + (ply as Score); // Checkmate
            } else {
                return 0; // Stalemate
            }
        }

        if !best_move.is_null() {
            self.tt.set(hash, best_move, alpha, depth);
        }

        alpha
    }

    fn root(&mut self, max_depth: usize) -> Option<Move> {
        let hash = self.positions.top().hash;
        let side = self.positions.top().side;
        let ply = 0;
        self.nodes_count = 0;

        // NOTE: `clear_all()` will zero everything internally, including
        // ply counter, while `clear()` will just reset the counter for
        // the current ply.
        // By using `clear_all()` we make sure that we can always search
        // very deep, even at the end of a very long game. But we loose
        // the ability to undo moves outside of the search function unless
        // we make a special case in `undo_move` for the root. In that special
        // case we don't decrement the ply counter that is already at 0.
        self.moves.clear_all();

        self.tt.clear();
        self.clock.start(self.positions.len());

        let old_fen = self.to_fen();
        if self.is_verbose {
            println!("# FEN {}", old_fen);
            println!("# allocating {} ms to move", self.clock.allocated_time());
        }

        if self.is_verbose {
            println!(" ply   score   time     nodes  pv");
        }

        // Current best move
        let mut best_move = Move::new_null();

        // Keep track of previous values at shallower depths
        let mut best_moves = [Move::new_null(); MAX_PLY];
        let mut best_scores = [0; MAX_PLY];

        for depth in 1..max_depth {
            let mut alpha = -INF;
            let beta = INF;

            self.moves.clear();
            if !best_move.is_null() {
                self.moves.add_move(best_move, MovesState::BestMove);
            }

            // Mate pruning
            if depth > 6 {
                // Stop the search if the position was mate at the 3 previous
                // shallower depths.
                let mut is_mate = true;
                let inf = INF - (MAX_PLY as Score);
                for d in 1..4 {
                    let score = best_scores[depth - d];
                    if -inf < score && score < inf {
                        is_mate = false;
                        break;
                    }
                }
                if is_mate {
                    break;
                }
            }

            while let Some(m) = self.next_move() {
                if self.clock.poll(self.nodes_count) {
                    break; // Discard search at this depth if time is out
                }

                self.make_move(m);
                let score = -self.search(-beta, -alpha, depth - 1, ply + 1);
                if !self.is_check(side) {
                    self.nodes_count += 1;
                    if score > alpha {
                        if self.is_verbose { // && self.nodes_count > 1000 {
                            // We skip the first thousand nodes to gain time
                            // TODO: do we need this?

                            // Get the PV line from the TT.
                            if depth == 1 {
                                self.tt.set(hash, m, score, depth);
                            }
                            self.print_thinking(depth, score, m);
                        }
                        alpha = score;
                        best_scores[depth] = score;
                        best_moves[depth] = m;
                    }
                }
                self.undo_move(m);
            }

            // Save the best move only if we found one and if we still have
            // some time left after the search at this depth.
            if !best_moves[depth].is_null() && !self.clock.poll(self.nodes_count) {
                best_move = best_moves[depth];

                self.tt.set(hash, best_move, alpha, depth);
            }
        }

        let new_fen = self.to_fen();
        if self.is_verbose {
            let n = self.nodes_count;
            let t = self.clock.elapsed_time();
            let nps = (n as f64) / ((t as f64) / 1000.0);
            println!("# {} ms used in search", t);
            println!("# {} nodes visited ({:.2e} nps)", n, nps);
            self.tt.print_stats();
        }
        debug_assert_eq!(old_fen, new_fen);

        if best_move.is_null() {
            None
        } else {
            Some(best_move)
        }
    }

    fn print_thinking(&mut self, depth: usize, score: Score, m: Move) {
        let time = self.clock.elapsed_time() / 10; // In centiseconds

        self.undo_move(m);
        let mut pv = self.get_pv(depth);

        if self.positions.top().side == BLACK {
            let ply = self.positions.len();
            pv = format!("{}. ... {}", ply / 2, pv);
        }
        self.make_move(m);

        println!(" {:>3}  {:>6}  {:>5}  {:>8}  {}", depth, score, time, self.nodes_count, pv);
    }

    fn get_pv(&mut self, depth: usize) -> String {
        if depth == 0 {
            return String::new()
        }

        let mut res = vec![];
        let mut m = Move::new_null();

        let hash = self.positions.top().hash;
        if let Some(t) = self.tt.get(&hash) {
            m = t.best_move();

            if self.positions.top().side == WHITE {
                let ply = self.positions.len();
                res.push(format!("{}.", 1 + (ply / 2)));
            }

            // TODO: put the rest of the code here (if the compiler allow it)
        }

        if !m.is_null() {
            res.push(self.move_to_san(m));
            self.make_move(m);
            res.push(self.get_pv(depth - 1));
            self.undo_move(m);
        }

        res.join(" ")
    }
}

#[cfg(test)]
mod tests {
    use common::*;
    use bitboard::BitboardExt;
    use clock::Clock;
    use eval;
    use fen::FEN;
    use game::Game;
    use moves::Move;
    use moves_generator::MovesGenerator;
    use search::Search;

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
        let fen = "4k3/8/4q3/8/8/4Q3/8/4K3 w - - 0 1";
        let mut game = Game::from_fen(fen);

        game.nodes_count = 0;
        game.clock = Clock::new(1, 5 * 1000); // 5 seconds
        game.clock.start(game.positions.len());

        let ply = 0;
        let alpha = -INF;
        let beta = INF;

        for depth in 1..5 {
            let score = game.search(alpha, beta, depth, ply + 1);

            assert!(score >= eval::QUEEN_VALUE);
        }
    }

    #[test]
    fn test_root() {
        let fen = "2k4r/ppp3pp/8/2b2p1P/PPP2p2/N4P2/3r2K1/1q5R w - - 4 29";
        let best_move = Move::new(G2, H3, QUIET_MOVE);
        let mut game = Game::from_fen(fen);
        game.clock = Clock::new(1, 5 * 1000); // 5 seconds
        let m = game.root(10).unwrap();
        assert_eq!(m.to_string(), best_move.to_string());


        let fen = "r1bq2rk/pp3pbp/2p1p1pQ/7P/3P4/2PB1N2/PP3PPR/2KR4 w - -";
        let best_move = Move::new(H6, H7, CAPTURE);
        let mut game = Game::from_fen(fen);
        game.clock = Clock::new(1, 5 * 1000); // 5 seconds
        let m = game.root(10).unwrap();
        assert_eq!(m.to_string(), best_move.to_string());


        let fen = "1n6/2rp3p/5Bpk/2p1P3/p1P2P2/5K2/PPB3P1/R6R b - - 0 1";
        let mut game = Game::from_fen(fen);
        game.clock = Clock::new(1, 1 * 1000); // 1 seconds
        assert_eq!(game.root(10), None);
    }

    #[test]
    fn test_bug_promotion() {
        let fen = "5n2/1k4P1/8/8/8/8/6K1/8 w - - 0 1";
        let mut game = Game::from_fen(fen);

        let m = Move::new(G7, G8, KNIGHT_PROMOTION);
        game.make_move(m);
        //assert!(!game.bitboards[WHITE as usize].get(G7));
        game.bitboards[(WHITE | PAWN) as usize].debug();
        assert!(!game.bitboards[(WHITE | PAWN) as usize].get(G8));
        game.undo_move(m);
        game.bitboards[(WHITE | PAWN) as usize].debug();
        assert!(!game.bitboards[(WHITE | PAWN) as usize].get(G8));

        let m = Move::new(G7, F8, KNIGHT_PROMOTION_CAPTURE);
        game.make_move(m);
        game.bitboards[(WHITE | PAWN) as usize].debug();
        assert!(!game.bitboards[(WHITE | PAWN) as usize].get(F8));
        game.undo_move(m);
        game.bitboards[(WHITE | PAWN) as usize].debug();
        assert!(!game.bitboards[(WHITE | PAWN) as usize].get(F8));
    }

    #[test]
    fn test_repetitions() {
        let fen = "7r/k7/7p/r2p3P/p2PqB2/2R3P1/5K2/3Q3R w - - 25 45";
        let mut game = Game::from_fen(fen);

        let m1 = Move::new(F2, G1, QUIET_MOVE);
        let m2 = Move::new(A7, B7, QUIET_MOVE);
        let m3 = Move::new(G1, F2, QUIET_MOVE);
        let m4 = Move::new(B7, A7, QUIET_MOVE);
        game.make_move(m1);
        game.make_move(m2);
        game.make_move(m3);
        game.make_move(m4);
        game.make_move(m1);
        game.make_move(m2);
        game.make_move(m3);
        game.make_move(m4);
        game.clock = Clock::new(1, 1000); // 1 second
        let m = game.root(10).unwrap();
        assert!(m != m1);
    }
}
