use std::io;

use littlewing::common::*;
use littlewing::attack::Attack;
use littlewing::clock::Clock;
use littlewing::fen::FEN;
use littlewing::game::Game;
use littlewing::piece::PieceAttr;
use littlewing::search::Search;
use littlewing::square::SquareString;
use littlewing::moves::Move;

pub struct XBoard {
    game: Game,
    force: bool
}

impl XBoard {
    pub fn new() -> XBoard {
        XBoard {
            game: FEN::from_fen(DEFAULT_FEN),
            force: true
        }
    }
    pub fn run(&mut self) {
        println!(""); // Acknowledge XBoard mode
        loop {
            let line = io::stdin().read_line().unwrap();
            let args: Vec<&str> = line.as_slice().trim().split(' ').collect();
            match args[0].as_slice() {
                "quit"     => break,
                "force"    => self.cmd_force(),
                "new"      => self.cmd_new(),
                "go"       => self.cmd_go(),
                "post"     => self.cmd_post(),
                "nopost"   => self.cmd_nopost(),
                "ping"     => self.cmd_ping(args.as_slice()),
                "setboard" => self.cmd_setboard(args.as_slice()),
                "level"    => self.cmd_level(args.as_slice()),
                "protover" => self.cmd_protover(args.as_slice()),
                _          => self.parse_move(args.as_slice())
            }
        }
    }

    pub fn cmd_force(&mut self) {
        self.force = true;
    }

    pub fn cmd_new(&mut self) {
        self.game.clear();
        self.game.load_fen(DEFAULT_FEN);
    }

    pub fn cmd_go(&mut self) {
        self.force = false;
        self.think();
    }

    pub fn cmd_post(&mut self) {
        self.game.is_verbose = true;
    }

    pub fn cmd_nopost(&mut self) {
        self.game.is_verbose = false;
    }

    pub fn cmd_ping(&mut self, args: &[&str]) {
        println!("pong {}", args[1].parse::<usize>().unwrap());
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
        println!("feature myname=\"Little Wing v0.0.1\"");
        println!("feature sigint=0 ping=1 setboard=1 done=1");
    }

    pub fn parse_move(&mut self, args: &[&str]) {
        let side = self.game.positions.top().side;
        let from: Square = SquareString::from_coord(String::from_str(args[0].slice(0, 2)));
        let to: Square = SquareString::from_coord(String::from_str(args[0].slice(2, 4)));

        if from > 63 || to > 63 {
            return; // TODO
        }

        let mt = if args[0].len() == 5 {
            let promotion = match args[0].as_slice().char_at(4) {
                'n' => KNIGHT_PROMOTION,
                'b' => BISHOP_PROMOTION,
                'r' => ROOK_PROMOTION,
                'q' => QUEEN_PROMOTION,
                _   => NULL_MOVE // FIXME
            };
            if self.game.board[to] == EMPTY {
                promotion
            } else {
                promotion & CAPTURE
            }
        } else if from == E1 ^ 56 * side && to == G1 ^ 56 * side {
            KING_CASTLE
        } else if from == E1 ^ 56 * side && to == C1 ^ 56 * side {
            QUEEN_CASTLE
        } else if self.game.board[to] == EMPTY {
            let kind = self.game.board[from].kind();
            let rank = (from ^ 56 * side).rank();
            if kind == PAWN && rank == 1 {
                DOUBLE_PAWN_PUSH
            } else if kind == PAWN && to == self.game.positions.top().en_passant {
                EN_PASSANT
            } else {
                QUIET_MOVE
            }
        } else {
            CAPTURE
        };

        let m = Move::new(from, to, mt);
        println!("parsed: {}", self.game.move_to_san(m));
        self.game.make_move(m);

        if !self.force {
            self.think();
        }
    }

    pub fn think(&mut self) {
        let m = self.game.root(256);
        self.game.make_move(m);

        println!("move {}", m.to_can());
    }
}
