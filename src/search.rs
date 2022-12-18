use crate::std::prelude::v1::*;
use crate::std::cmp;
use crate::std::ops::Range;

#[cfg(feature = "std")]
use crate::std::thread;

use crate::color::*;
use crate::piece::*;
use crate::common::*;
use crate::attack::Attack;
use crate::bitboard::BitboardExt;
use crate::eval::Eval;
#[cfg(feature = "std")]
use crate::fen::FEN;
use crate::game::Game;
use crate::piece_move::PieceMove;
use crate::piece_move_generator::PieceMoveGenerator;
use crate::piece_move_notation::PieceMoveNotation;
use crate::transposition::Bound;
#[cfg(feature = "std")]
use crate::protocols::Protocol;

/// Search the game
pub trait Search {
    /// Search the number of legal moves at the given depth
    fn perft(&mut self, depth: Depth) -> u64;

    /// Searh the best move at the given depth range
    fn search(&mut self, depths: Range<Depth>) -> Option<PieceMove>;

    /// Searh the best move from the root position at the given depth range
    fn search_root(&mut self, depths: Range<Depth>) -> Option<PieceMove>;

    /// Searh the best score between alpha and beta from a node position at the given depth
    fn search_node(&mut self, alpha: Score, beta: Score, depth: Depth, ply: usize) -> Score;

    /// Specialized quiescence search
    fn quiescence(&mut self, alpha: Score, beta: Score, depth: Depth, ply: usize) -> Score;

    fn is_mate(&mut self) -> bool;
    fn get_moves(&mut self) -> Vec<PieceMove>;
}

trait SearchExt {
    fn get_pv(&mut self, depth: Depth) -> String;

    #[cfg(feature = "std")]
    fn print_debug_init(&self, depth: Depth);
    #[cfg(feature = "std")]
    fn print_thinking_init(&self);
    #[cfg(feature = "std")]
    fn print_thinking(&mut self, depth: Depth, score: Score, m: PieceMove);
}

impl Search for Game {
    fn perft(&mut self, depth: Depth) -> u64 {
        if depth == 0 {
            1
        } else {
            let side = self.side();
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

    fn search(&mut self, depths: Range<Depth>) -> Option<PieceMove> {
        self.nodes_count = 0;
        self.tt.reset();

        // NOTE: `clear_all()` will zero everything internally, including
        // ply counter, while `clear()` will just reset the counter for
        // the current ply.
        // By using `clear_all()` we make sure that we can always search
        // very deep, even at the end of a very long game. But we loose
        // the ability to undo moves outside of the search function unless
        // we make a special case in `undo_move` for the root. In that special
        // case we don't decrement the ply counter that is already at 0.
        self.moves.clear_all();

        self.clock.start(self.positions.len());

        let n = if cfg!(feature = "std") { self.threads_count } else { 0 };

        if self.is_debug {
            println!("# using {} threads", n);
        }

        if n == 0 {
            return self.search_root(depths);
        }

        #[cfg(not(feature = "std"))]
        unreachable!();

        #[cfg(feature = "std")]
        {
            let mut children = Vec::with_capacity(n);

            for i in 0..n {
                let mut clone = self.clone();
                if i > 0 {
                    clone.is_search_verbose = false;
                    clone.is_debug = false;
                }

                let min_depth = depths.start; // TODO: + i as usize;
                let max_depth = depths.end;

                let builder = thread::Builder::new().
                    name(format!("search_{}", i)).
                    stack_size(4 << 20);

                children.push(builder.spawn(move || {
                    clone.search_root(min_depth..max_depth)
                }).unwrap());
            }

            let mut res = Vec::with_capacity(n);
            for child in children {
                res.push(child.join().unwrap());
            }

            res[0] // best move found by the first thread
        }
    }

    fn search_root(&mut self, depths: Range<Depth>) -> Option<PieceMove> {
        let hash = self.positions.top().hash;
        let side = self.side();
        let ply = 0;

        #[cfg(feature = "std")]
        if self.is_debug {
            self.print_debug_init(depths.start);
        }

        #[cfg(feature = "std")]
        if self.is_search_verbose {
            self.print_thinking_init();
        }

        // Current best move
        #[allow(unused_assignments)]
        let mut best_score = 0; // Overwritten before being read in no_std
        let mut best_move = PieceMove::new_null();

        // Keep track of previous values at shallower depths
        let mut best_scores = [0; MAX_PLY];
        let mut best_moves = [PieceMove::new_null(); MAX_PLY];

        debug_assert!(depths.start > 0);
        for depth in depths {
            let mut alpha = -INF;
            let beta = INF;

            self.moves.clear();
            if !best_move.is_null() {
                self.moves.add_move(best_move);
            }

            // Mate pruning
            if depth > 6 {
                // Stop the search if the position was mate at the 3 previous
                // shallower depths.
                let mut is_mate = true;
                let inf = INF - (MAX_PLY as Score);
                for d in 1..4 {
                    let score = best_scores[(depth - d) as usize];
                    if -inf < score && score < inf {
                        is_mate = false;
                        break;
                    }
                }
                if is_mate {
                    break;
                }
            }

            let mut has_legal_moves = false;
            while let Some(m) = self.next_move() {
                if self.clock.poll(self.nodes_count) {
                    break; // Discard search at this depth if time is out
                }

                self.make_move(m);
                let score = -self.search_node(-beta, -alpha, depth - 1, ply + 1);
                if !self.is_check(side) {
                    has_legal_moves = true;
                    self.nodes_count += 1;
                    if score > alpha {
                        if self.is_search_verbose && !self.clock.poll(self.nodes_count) {
                            // TODO: skip the first thousand nodes to gain time?

                            self.tt.set(hash, depth, score, m, Bound::Exact);

                            // Get the PV line from the TT.
                            #[cfg(feature = "std")]
                            self.print_thinking(depth, score, m);
                        }
                        alpha = score;
                        best_scores[depth as usize] = score;
                        best_moves[depth as usize] = m;
                    }
                }
                self.undo_move(m);
            }

            // Save the best move only if we found one and if we still have
            // some time left after the search at this depth.
            if !best_moves[depth as usize].is_null() && !self.clock.poll(self.nodes_count) {
                best_move = best_moves[depth as usize];
                best_score = best_scores[depth as usize];

                self.tt.set(hash, depth, best_score, best_move, Bound::Exact);
            }

            // No need to iterate if there's no legal moves to play
            if !has_legal_moves {
                break;
            }
        }

        #[cfg(feature = "std")]
        if self.is_debug {
            let n = self.nodes_count;
            let t = self.clock.elapsed_time();
            let nps = (n as f64) / ((t as f64) / 1000.0);
            if self.is_search_verbose {
                println!();
            }
            println!("# {:15} {:>8}", "score:", best_score);
            println!("# {:15} {:>8} ms", "time:", t);
            println!("# {:15} {:>8} ({:.2e} nps)", "nodes:", n, nps);

            self.tt.print_stats();
        }

        if best_move.is_null() {
            None
        } else {
            Some(best_move)
        }
    }

    fn search_node(&mut self, mut alpha: Score, mut beta: Score, depth: Depth, ply: usize) -> Score {
        if self.clock.poll(self.nodes_count) {
            return 0;
        }

        if depth == 0 {
            return self.quiescence(alpha, beta, depth - 1, ply + 1);
        }

        // Detect draw by threefold repetitions and fifty-moves rule
        if self.positions.is_draw() {
            return 0;
        }

        let hash = self.positions.top().hash;
        let side = self.side();
        let is_null_move = !self.positions.top().null_move_right;
        let is_pv = alpha != beta - 1;

        let mut best_move = PieceMove::new_null();
        let mut best_score = alpha;
        let old_alpha = alpha; // To test if best score raise initial alpha

        // Try to get the best move from transposition_table table
        if let Some(t) = self.tt.get(hash) {
            if !is_pv && t.depth() >= depth {
                match t.bound() {
                    Bound::Exact => {
                        return t.score();
                    },
                    Bound::Lower => {
                        if t.score() > alpha {
                            alpha = t.score();
                        }
                    },
                    Bound::Upper => {
                        if t.score() < beta {
                            beta = t.score();
                        }
                    }
                }
                if alpha >= beta {
                    return t.score();
                }
            }

            best_move = t.best_move();
        }

        let is_in_check = self.is_check(side);

        // Null Move Pruning (NMP)
        let pieces_count = self.bitboard(side).count();
        let pawns_count = self.bitboard(side | PAWN).count();
        let is_pawn_ending = pieces_count == pawns_count + 1; // pawns + king

        let nmp_allowed =
            !is_in_check &&
            !is_null_move &&
            !is_pv &&
            !is_pawn_ending;

        if nmp_allowed {
            let r = cmp::min(depth - 1, 3 + depth / 4);
            let m = PieceMove::new_null();
            self.make_move(m);
            self.positions.disable_null_move();
            let score = -self.search_node(-beta, -beta + 1, depth - r - 1, ply + 1);
            self.positions.enable_null_move();
            self.undo_move(m);

            if score >= beta {
                return score;
            }
        }

        // Internal Iterative Deepening (IID)
        //
        // If we didn't get a best move from the transposition_table table,
        // get it by searching the position at a reduced depth.
        let iid_allowed = is_pv && best_move.is_null();

        if iid_allowed && depth > 3 {
            self.search_node(-beta, -alpha, depth / 2, ply + 1);

            if let Some(t) = self.tt.get(hash) {
                best_move = t.best_move();
            }
        }

        self.moves.clear();
        if !best_move.is_null() {
            self.moves.add_move(best_move);
        }

        let eval = self.eval_material(side) - self.eval_material(side ^ 1);

        let mut has_legal_moves = false;
        let mut is_first_move = true;
        while let Some(m) = self.next_move() {
            self.make_move(m);

            if self.is_check(side) {
                self.undo_move(m);
                continue;
            }

            self.nodes_count += 1;
            has_legal_moves = true;

            let mut score;
            if is_first_move {
                // Search the first move with the full window
                score = -self.search_node(-beta, -alpha, depth - 1, ply + 1);

                best_score = score;
                best_move = m;
                is_first_move = false;
            } else {
                let is_giving_check = self.is_check(side ^ 1);
                let mut r = 0; // Depth reduction

                // Futility Pruning (FP)
                let fp_allowed =
                    !is_pv &&
                    !is_in_check &&
                    !is_giving_check &&
                    !m.is_capture() &&
                    !m.is_promotion();

                if fp_allowed && depth < 6 {
                    let margin = 100 * depth as Score;
                    if eval + margin < alpha {
                        self.undo_move(m);
                        continue;
                    }
                }

                // Late Move Reduction (LMR)
                let lmr_allowed =
                    !is_pv &&
                    !is_in_check &&
                    !is_giving_check &&
                    !m.is_capture() &&
                    !m.is_promotion();

                if lmr_allowed && depth > 2 {
                    r += 1; // Do the search at a reduced depth
                    if depth > 4 {
                        r += depth / 4;
                    }
                }

                // Search the other moves with the reduced window
                score = -self.search_node(-alpha - 1, -alpha, depth - r - 1, ply + 1);

                // LMR re-search
                if r > 0 && score > alpha {
                    score = -self.search_node(-alpha - 1, -alpha, depth - 1, ply + 1);
                }

                // Re-search with the full window
                if alpha < score && score < beta {
                    score = -self.search_node(-beta, -alpha, depth - 1, ply + 1);
                }
            }

            self.undo_move(m);

            if score > alpha {
                if score >= beta {
                    if !m.is_capture() {
                        self.moves.add_killer_move(m);
                    }
                    self.tt.set(hash, depth, score, m, Bound::Lower);
                    return score;
                }

                alpha = score;
                best_score = score;
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
            let bound = if best_score > old_alpha {
                Bound::Exact
            } else {
                Bound::Upper
            };
            self.tt.set(hash, depth, best_score, best_move, bound);
        }

        alpha
    }

    fn quiescence(&mut self, mut alpha: Score, mut beta: Score, depth: Depth, ply: usize) -> Score {
        // Time limit abort
        if self.clock.poll(self.nodes_count) {
            return 0;
        }

        // Static evaluation
        let eval = self.eval();

        // Maximum depth abort
        if ply >= MAX_PLY {
            return eval;
        }

        // Delta pruning
        let delta = 1000; // Queen value
        if eval < alpha - delta {
            return alpha;
        }

        // Stand pat pruning
        if eval > alpha {
            if eval >= beta {
                return eval;
            }

            alpha = eval;
        }

        let hash = self.positions.top().hash;
        let side = self.side();
        let old_alpha = alpha;
        let mut best_move = PieceMove::new_null();

        if let Some(t) = self.tt.get(hash) {
            if t.depth() >= depth { // This node has already been searched
                match t.bound() {
                    Bound::Exact => {
                        return t.score();
                    },
                    Bound::Lower => {
                        if t.score() > alpha {
                            alpha = t.score();
                        }
                    },
                    Bound::Upper => {
                        if t.score() < beta {
                            beta = t.score();
                        }
                    }
                }
                if alpha >= beta {
                    return t.score();
                }
            }

            best_move = t.best_move();
        }

        self.moves.clear();
        if !best_move.is_null() {
            self.moves.add_move(best_move);
        }
        while let Some(m) = self.next_capture() {
            self.make_move(m);

            if self.is_check(side) {
                self.undo_move(m);
                continue;
            }
            self.nodes_count += 1;

            let score = -self.quiescence(-beta, -alpha, depth - 1, ply + 1);

            self.undo_move(m);

            if score > alpha {
                if score >= beta {
                    self.tt.set(hash, depth, score, m, Bound::Lower);
                    return score;
                }
                alpha = score;
                best_move = m;
            }
        }

        if !best_move.is_null() {
            let bound = if alpha > old_alpha {
                Bound::Exact
            } else {
                Bound::Upper
            };
            self.tt.set(hash, depth, alpha, best_move, bound);
        }

        alpha
    }

    fn is_mate(&mut self) -> bool {
        let side = self.side();
        self.moves.clear();
        while let Some(m) = self.next_move() {
            self.make_move(m);
            let is_legal = !self.is_check(side);
            self.undo_move(m);
            if is_legal {
                return false;
            }
        }
        true
    }

    fn get_moves(&mut self) -> Vec<PieceMove> {
        let mut res = Vec::new();
        let side = self.side();
        self.moves.clear();
        while let Some(m) = self.next_move() {
            self.make_move(m);
            if !self.is_check(side) {
                res.push(m);
            }
            self.undo_move(m);
        }
        res
    }
}

impl SearchExt for Game {
    #[cfg(feature = "std")]
    fn print_debug_init(&self, depth: Depth) {
        println!("# FEN {}", self.to_fen());
        println!("# allocating {} ms to move", self.clock.allocated_time());
        println!("# starting search at depth {}", depth);
        println!();
    }

    #[cfg(feature = "std")]
    fn print_thinking_init(&self) {
        if self.protocol != Protocol::UCI {
            println!("  {:>3}  {:>5}  {:>6}  {:>9}  {}", "ply", "score", "time", "nodes", "pv");
        }
    }

    #[cfg(feature = "std")]
    fn print_thinking(&mut self, depth: Depth, score: Score, m: PieceMove) {
        self.undo_move(m);

        let time = self.clock.elapsed_time();
        let nodes = self.nodes_count;
        let mut pv = self.get_pv(depth);

        match self.protocol {
            Protocol::UCI => {
                println!("info depth {} score cp {} time {} nodes {} pv {}", depth, score, time, nodes, pv);
            },
            Protocol::XBoard | Protocol::CLI => {
                if self.side() == BLACK {
                    let fm = self.positions.fullmoves();
                    pv = format!("{}. ... {}", fm, pv);
                }

                // Split PV over multiple lines of 80 chars max in CLI mode
                if self.protocol == Protocol::CLI {
                    let mut width = 34;
                    let mut lines = Vec::new();
                    let mut line = Vec::new();
                    for chunk in pv.trim().split(' ').collect::<Vec<&str>>().chunks(3) {
                        let s = chunk.join(" ");
                        if width + s.len() >= 80 {
                            width = 34;
                            lines.push(line.join(" "));
                            line.clear();
                        }
                        width += s.len() + 1;
                        line.push(s);
                    }
                    if line.len() > 0 {
                        lines.push(line.join(" "));
                    }
                    pv = lines.join(&format!("{:<34}", "\n"));
                }

                println!("  {:>3}  {:>5}  {:>6}  {:>9}  {}", depth, score, time / 10, nodes, pv);
            }
        }

        self.make_move(m);
    }

    fn get_pv(&mut self, depth: Depth) -> String {
        #[cfg(feature = "std")]
        let is_san_format = self.protocol != Protocol::UCI;
        #[cfg(not(feature = "std"))]
        let is_san_format = false;

        if depth == 0 {
            return String::new();
        }

        let mut res = vec![];
        let mut m = PieceMove::new_null();

        let side = self.side();
        let hash = self.positions.top().hash;
        if let Some(t) = self.tt.get(hash) {
            m = t.best_move();

            if is_san_format && side == WHITE {
                let fm = self.positions.fullmoves();
                res.push(format!("{}.", fm));
            }

            // TODO: put the rest of the code here (if the compiler allow it)
        }

        if !m.is_null() {
            let cur = if is_san_format {
                self.move_to_san(m)
            } else {
                m.to_lan()
            };
            self.make_move(m);

            let pv = &self.get_pv(depth - 1);
            let sep = if is_san_format && self.is_check(side ^ 1) {
                if pv == "#" { "" } else { "+ " }
            } else {
                " "
            };
            res.push(format!("{}{}{}", cur, sep, pv));

            self.undo_move(m);
        } else if self.is_check(side) {
            if is_san_format {
                res.push("#".into());
            }
        }

        res.join(" ")
    }
}

#[cfg(test)]
mod tests {
    use crate::std::prelude::v1::*;

    use crate::color::*;
    use crate::piece::*;
    use crate::square::*;
    use crate::common::*;
    use crate::bitboard::BitboardExt;
    use crate::clock::Clock;
    use crate::eval;
    use crate::fen::FEN;
    use crate::game::Game;
    use crate::piece_move::PieceMove;
    use crate::piece_move_generator::PieceMoveGenerator;
    use crate::piece_move_notation::PieceMoveNotation;
    use crate::search::Search;

    #[test]
    fn test_perft() {
        let mut game = Game::new();

        // Initial position
        game.load_fen(DEFAULT_FEN).unwrap();
        assert_eq!(game.perft(1), 20);
        assert_eq!(game.perft(2), 400);
        assert_eq!(game.perft(3), 8902);
        assert_eq!(game.perft(4), 197281);

        let fen = "7k/8/8/p7/1P6/8/8/7K b - - 0 1";
        game.load_fen(fen).unwrap();
        assert_eq!(game.perft(1), 5);

        let fen = "k6K/8/8/b6b/8/8/8/8 b - - 0 1";
        game.load_fen(fen).unwrap();
        assert_eq!(game.perft(1), 17);

        let fen = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -";
        game.load_fen(fen).unwrap();
        assert_eq!(game.perft(1), 14);
        assert_eq!(game.perft(2), 191);
        assert_eq!(game.perft(3), 2812);

        let fen = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1";
        game.load_fen(fen).unwrap();
        assert_eq!(game.perft(1), 6);
        assert_eq!(game.perft(2), 264);
        assert_eq!(game.perft(3), 9467);

        let fen = "r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1";
        game.load_fen(fen).unwrap();
        assert_eq!(game.perft(1), 6);
        assert_eq!(game.perft(2), 264);
        assert_eq!(game.perft(3), 9467);

        // Kiwipete position
        let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -";
        game.load_fen(fen).unwrap();
        assert_eq!(game.perft(1), 48);
        assert_eq!(game.perft(2), 2039);
        assert_eq!(game.perft(3), 97862);

        let fen = "rnbqkb1r/pp1p1ppp/2p5/4P3/2B5/8/PPP1NnPP/RNBQK2R w KQkq - 0 6";
        game.load_fen(fen).unwrap();
        assert_eq!(game.perft(1), 42);
        assert_eq!(game.perft(2), 1352);
        assert_eq!(game.perft(3), 53392);
    }

    #[test]
    fn test_search_node() {
        let fen = "4k3/8/4q3/8/8/4Q3/8/4K3 w - - 0 1";
        let mut game = Game::from_fen(fen).unwrap();

        game.nodes_count = 0;
        game.clock = Clock::new(1, 5 * 1000); // 5 seconds
        game.clock.start(game.positions.len());

        let ply = 0;
        let alpha = -INF;
        let beta = INF;

        for depth in 1..5 {
            let score = game.search_node(alpha, beta, depth, ply + 1);

            assert!(score >= eval::QUEEN_VALUE);
        }
    }

    #[test]
    fn test_stalemate() {
        let mut game = Game::from_fen("4k3/4P3/4K3/8/8/8/8/ b - - 0 1").unwrap();

        game.nodes_count = 0;
        game.clock = Clock::new(1, 5 * 1000); // 5 seconds
        game.clock.start(game.positions.len());

        let ply = 0;
        let alpha = -INF;
        let beta = INF;

        for depth in 1..5 {
            let score = game.search_node(alpha, beta, depth, ply + 1);
            assert_eq!(score, 0);
        }
    }

    #[test]
    fn test_threefold_repetition() {
        // Fischer vs Petrosian (1971)
        let mut game = Game::from_fen("8/pp3p1k/2p2q1p/3r1P1Q/5R2/7P/P1P2P2/7K w - - 1 30").unwrap();
        let moves = vec!["h5e2", "f6e5", "e2h5", "e5f6", "h5e2", "d5e5", "e2d3", "e5d5", "d3e2"];
        for s in moves {
            let m = game.move_from_lan(s);
            game.make_move(m);
            game.history.push(m);
        }

        game.nodes_count = 0;
        game.clock = Clock::new(1, 5 * 1000); // 5 seconds
        game.clock.start(game.positions.len());

        let ply = 0;
        let alpha = -INF;
        let beta = INF;

        for depth in 1..5 {
            let score = game.search_node(alpha, beta, depth, ply + 1);
            assert_eq!(score, 0);
        }
    }

    #[test]
    fn test_fifty_move_rule() {
        // Timman vs Lutz (1995)
        let mut game = Game::from_fen("8/7k/8/1r3KR1/5B2/8/8/8 w - - 105 122").unwrap();

        game.nodes_count = 0;
        game.clock = Clock::new(1, 5 * 1000); // 5 seconds
        game.clock.start(game.positions.len());

        let ply = 0;
        let alpha = -INF;
        let beta = INF;

        for depth in 1..5 {
            let score = game.search_node(alpha, beta, depth, ply + 1);
            assert_eq!(score, 0);
        }
    }

    #[test]
    fn test_search() {
        let fen = "2k4r/ppp3pp/8/2b2p1P/PPP2p2/N4P2/3r2K1/1q5R w - - 4 29";
        let best_move = PieceMove::new(G2, H3, QUIET_MOVE);
        let mut game = Game::from_fen(fen).unwrap();
        game.clock = Clock::new(1, 5 * 1000); // 5 seconds
        let m = game.search(1..10).unwrap();
        assert_eq!(m.to_string(), best_move.to_string());


        let fen = "r1bq2rk/pp3pbp/2p1p1pQ/7P/3P4/2PB1N2/PP3PPR/2KR4 w - -";
        let best_move = PieceMove::new(H6, H7, CAPTURE);
        let mut game = Game::from_fen(fen).unwrap();
        game.clock = Clock::new(1, 5 * 1000); // 5 seconds
        let m = game.search(1..10).unwrap();
        assert_eq!(m.to_string(), best_move.to_string());


        let fen = "1n6/2rp3p/5Bpk/2p1P3/p1P2P2/5K2/PPB3P1/R6R b - - 0 1";
        let mut game = Game::from_fen(fen).unwrap();
        game.clock = Clock::new(1, 1 * 1000); // 1 seconds
        assert_eq!(game.search(1..10), None);
    }

    #[test]
    fn test_bug_promotion() {
        let fen = "5n2/1k4P1/8/8/8/8/6K1/8 w - - 0 1";
        let mut game = Game::from_fen(fen).unwrap();

        let m = PieceMove::new(G7, G8, KNIGHT_PROMOTION);
        game.make_move(m);
        //assert!(!game.bitboards[WHITE as usize].get(G7));
        game.bitboards[(WHITE | PAWN) as usize].debug();
        assert!(!game.bitboards[(WHITE | PAWN) as usize].get(G8));
        game.undo_move(m);
        game.bitboards[(WHITE | PAWN) as usize].debug();
        assert!(!game.bitboards[(WHITE | PAWN) as usize].get(G8));

        let m = PieceMove::new(G7, F8, KNIGHT_PROMOTION_CAPTURE);
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
        let mut game = Game::from_fen(fen).unwrap();

        let m1 = PieceMove::new(F2, G1, QUIET_MOVE);
        let m2 = PieceMove::new(A7, B7, QUIET_MOVE);
        let m3 = PieceMove::new(G1, F2, QUIET_MOVE);
        let m4 = PieceMove::new(B7, A7, QUIET_MOVE);
        game.make_move(m1);
        game.make_move(m2);
        game.make_move(m3);
        game.make_move(m4);
        game.make_move(m1);
        game.make_move(m2);
        game.make_move(m3);
        game.make_move(m4);
        game.clock = Clock::new(1, 1000); // 1 second
        let m = game.search(1..10).unwrap();
        assert!(m != m1);
    }

    #[test]
    fn test_null_move_pruning() {
        // Zugzwang #1
        let fen = "1q1k4/2Rr4/8/2Q3K1/8/8/8/8 w - - 0 1";
        let mut game = Game::from_fen(fen).unwrap();
        game.clock = Clock::new(1, 5000); // 1 second
        let m = game.search(1..100).unwrap();
        assert_eq!(m, PieceMove::new(G5, H6, QUIET_MOVE));

        // Zugzwang #2
        /*
        //FIXME: this position takes too long at the moment
        let fen = "8/8/p1p5/1p5p/1P5p/8/PPP2K1p/4R1rk w - - 0 1";
        let mut game = Game::from_fen(fen).unwrap();
        game.clock = Clock::new(1, 1000); // 1 second
        let m = game.search(1..100).unwrap();
        assert_eq!(m, PieceMove::new(E1, F1, QUIET_MOVE));
        */
    }

    #[test]
    fn test_is_mate() {
        let fen = "rnbqkbnr/pppp1ppp/8/4p3/6P1/5P2/PPPPP2P/RNBQKBNR b KQkq g3 0 2";
        let mut game = Game::from_fen(fen).unwrap();
        assert!(!game.is_mate());
        game.make_move(PieceMove::new(D8, H4, QUIET_MOVE));
        assert!(game.is_mate());
    }

    #[test]
    fn test_get_moves() {
        let mut game = Game::from_fen("8/8/8/8/r7/1k6/8/K7 w - - 0 1").unwrap();
        assert_eq!(game.get_moves(), vec![PieceMove::new(A1, B1, QUIET_MOVE)])
    }
}
