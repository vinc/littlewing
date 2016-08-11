use common::*;
use clock::Clock;
use moves::{Move, Moves};
use position::Positions;
use transpositions::Transpositions;
use zobrist::Zobrist;

pub struct Game {
    pub is_verbose: bool,
    pub nodes_count: u64,
    pub clock: Clock,
    pub bitboards: [Bitboard; 14],
    pub board: [Piece; 64],
    pub moves: Moves,
    pub positions: Positions,
    pub zobrist: Zobrist,
    pub history: Vec<Move>,
    pub tt: Transpositions
}

impl Game {
    pub fn new() -> Game {
        Game {
            is_verbose: false,
            nodes_count: 0,
            clock: Clock::new(40, 5 * 60),
            bitboards: [0; 14],
            board: [EMPTY; 64],
            moves: Moves::new(),
            positions: Positions::new(),
            zobrist: Zobrist::new(),
            history: Vec::new(),
            tt: Transpositions::with_capacity(100000)
        }
    }

    pub fn clear(&mut self) {
        self.bitboards = [0; 14];
        self.board = [EMPTY; 64];
        self.moves.clear_all();
        self.positions.clear();
        self.history.clear();
    }

    pub fn to_string(&self) -> String {
        // FIXME: Testing `map` and `fold` for the lulz

        let sep = (0..8)
            .map(|_| "+---")
            .fold(String::new(), |r, s| r + s) + "+\n";

        /*
        String::new() + &*sep + (0..8).map(|i| {
            (0..8)
                .map(|j| {
                    let c = (self.board[8 * (7 - i) + j as usize]).to_char();
                    String::from("| ") + &*(c.to_string()) + " "
                })
                .concat() + "|\n" + &*sep
        }).concat()
        */

        sep
    }
}
