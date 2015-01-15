use std::io;

use littlewing::common::*;
use littlewing::attack::Attack;
use littlewing::clock::Clock;
use littlewing::fen::FEN;
use littlewing::game::Game;
use littlewing::search::Search;
use littlewing::square::SquareString;
use littlewing::moves::Move;

pub struct XBoard {
    game: Game
}

impl XBoard {
    pub fn new() -> XBoard {
        XBoard {
            game: FEN::from_fen(DEFAULT_FEN)
        }
    }
    pub fn run(&mut self) {
        println!(""); // Acknowledge XBoard mode
        loop {
            let line = io::stdin().read_line().unwrap();
            let args: Vec<&str> = line.as_slice().trim().split(' ').collect();
            match args[0].as_slice() {
                "quit"     => break,
                "new"      => self.cmd_new(),
                "go"       => self.cmd_go(),
                "post"     => self.cmd_post(),
                "nopost"   => self.cmd_nopost(),
                "setboard" => self.cmd_setboard(args.as_slice()),
                "level"    => self.cmd_level(args.as_slice()),
                "protover" => self.cmd_protover(args.as_slice()),
                _          => self.parse_move(args.as_slice())
            }
        }
    }

    pub fn cmd_new(&mut self) {
        self.game.clear();
        self.game.load_fen(DEFAULT_FEN);
    }

    pub fn cmd_go(&mut self) {
        self.think();
    }

    pub fn cmd_post(&mut self) {
        self.game.is_verbose = true;
    }

    pub fn cmd_nopost(&mut self) {
        self.game.is_verbose = false;
    }

    pub fn cmd_setboard(&mut self, args: &[&str]) {
        if args.len() == 1 {
            panic!("no fen given");
        }

        let s = args.slice_from(1).connect(" ");
        let fen = s.as_slice();

        self.game = FEN::from_fen(fen);
    }

    pub fn cmd_level(&mut self, args: &[&str]) {
        let moves = args[1].parse::<u8>().unwrap();
        let time = args[2].parse::<u16>().unwrap();

        self.game.clock = Clock::new(moves, time);
    }

    pub fn cmd_protover(&mut self, args: &[&str]) {
        println!("feature sigint=0 done=1");
    }

    pub fn parse_move(&mut self, args: &[&str]) {
        let side = self.game.positions.top().side;
        let cmd = args[0];
        let from: Square = SquareString::from_square_string(String::from_str(cmd.slice(0, 2)));
        let to: Square = SquareString::from_square_string(String::from_str(cmd.slice(2, 4)));

        if from > 63 || to > 63 {
            return;
        }

        let mt = if from == E1 ^ 56 * side && to == G1 ^ 56 * side {
            KING_CASTLE
        } else if from == E1 ^ 56 * side && to == C1 ^ 56 * side {
            QUEEN_CASTLE
        } else if self.game.board[to] == EMPTY {
            QUIET_MOVE
        } else {
            CAPTURE
        };

        let m = Move::new(from, to, mt);
        self.game.make_move(m);

        self.think();
    }

    pub fn think(&mut self) {
        let m = self.game.root(256);
        self.game.make_move(m);

        println!("move {}", m.to_can());
    }
}
