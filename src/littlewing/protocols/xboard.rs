use std::io;

use littlewing::common::*;
use littlewing::attack::Attack;
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
                "setboard" => self.cmd_setboard(args.as_slice()),
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
        self.game.clear();
    }

    pub fn cmd_setboard(&mut self, args: &[&str]) {
        if args.len() == 1 {
            panic!("no fen given");
        }

        let s = args.slice_from(1).connect(" ");
        let fen = s.as_slice();

        self.game = FEN::from_fen(fen);
    }

    pub fn cmd_protover(&mut self, args: &[&str]) {
        println!("feature sigint=0");
    }

    pub fn parse_move(&mut self, args: &[&str]) {
        let cmd = args[0];
        let from: Square = SquareString::from_square_string(String::from_str(cmd.slice(0, 2)));
        let to: Square = SquareString::from_square_string(String::from_str(cmd.slice(2, 4)));

        if from > 63 || to > 63 {
            return;
        }

        let m = Move::new(from, to, QUIET_MOVE);
        self.game.make_move(m);
        
        self.game.generate_moves();
        let n = self.game.moves.len();
        let m = self.game.moves[0];
        self.game.make_move(m);
        println!("move {}", m.to_can());
    }
}
