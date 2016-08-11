use std::io;
use regex::Regex;

use common::*;
use clock::Clock;
use fen::FEN;
use game::Game;
use piece::PieceAttr;
use search::Search;
use square::SquareString;
use moves::Move;
use version;

pub struct XBoard {
    game: Game,
    max_depth: usize,
    force: bool
}

impl XBoard {
    pub fn new() -> XBoard {
        XBoard {
            game: FEN::from_fen(DEFAULT_FEN),
            max_depth: MAX_PLY - 10,
            force: false
        }
    }
    pub fn run(&mut self) {
        println!(""); // Acknowledge XBoard mode
        loop {
            let mut line = String::new();
            io::stdin().read_line(&mut line).unwrap();
            let args: Vec<&str> = line.trim().split(' ').collect();
            match args[0] {
                "quit"     => break,
                "force"    => self.cmd_force(),
                "new"      => self.cmd_new(),
                "go"       => self.cmd_go(),
                "post"     => self.cmd_post(),
                "nopost"   => self.cmd_nopost(),
                "undo"     => self.cmd_undo(),
                "remove"   => self.cmd_remove(),
                "time"     => self.cmd_time(&*args),
                "ping"     => self.cmd_ping(&*args),
                "setboard" => self.cmd_setboard(&*args),
                "sd"       => self.cmd_depth(&*args),
                "level"    => self.cmd_level(&*args),
                "protover" => self.cmd_protover(&*args),
                _          => self.parse_move(&*args)
            }
        }
    }

    pub fn cmd_force(&mut self) {
        self.force = true;
    }

    pub fn cmd_new(&mut self) {
        self.max_depth = MAX_PLY - 10;
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

    pub fn cmd_undo(&mut self) {
        let m = self.game.history.pop().unwrap();
        self.game.undo_move(m);
    }

    pub fn cmd_remove(&mut self) {
        let m = self.game.history.pop().unwrap();
        self.game.undo_move(m);

        let m = self.game.history.pop().unwrap();
        self.game.undo_move(m);
    }

    pub fn cmd_time(&mut self, args: &[&str]) {
        // `time` is given in centiseconds
        let time = args[1].parse::<u64>().unwrap();
        self.game.clock.set_time(time * 10);
    }

    pub fn cmd_ping(&mut self, args: &[&str]) {
        println!("pong {}", args[1].parse::<usize>().unwrap());
    }

    pub fn cmd_setboard(&mut self, args: &[&str]) {
        if args.len() == 1 {
            panic!("no fen given");
        }

        let s = args[1..].join(" ");
        let fen = &*s;
        self.game = Game::from_fen(fen);
    }

    pub fn cmd_level(&mut self, args: &[&str]) {
        let moves = args[1].parse::<u16>().unwrap();

        // `time` is given in `mm:ss` or `ss`.
        let time = match args[2].find(':') {
            Some(i) => args[2][0..i].parse::<u64>().unwrap() * 60 +
                       args[2][(i + 1)..].parse::<u64>().unwrap(),
            None    => args[2].parse::<u64>().unwrap()
        };

        self.game.clock = Clock::new(moves, time * 1000);
    }

    pub fn cmd_depth(&mut self, args: &[&str]) {
        self.max_depth = args[1].parse::<usize>().unwrap() + 1;
    }

    #[allow(unused_variables)]
    pub fn cmd_protover(&mut self, args: &[&str]) { // FIXME
        println!("feature myname=\"{}\"", version());
        println!("feature sigint=0 ping=1 setboard=1 done=1");
    }

    // TODO: move the code doing the actual parsing to `Move::from()`
    pub fn parse_move(&mut self, args: &[&str]) {
        let re = Regex::new(r"^[a-h][0-9][a-h][0-9][nbrq]?$").unwrap();

        if !re.is_match(args[0]) {
            return;
        }

        let side = self.game.positions.top().side;
        let from: Square = SquareString::from_coord(String::from(&args[0][0..2]));
        let to: Square = SquareString::from_coord(String::from(&args[0][2..4]));

        let piece = self.game.board[from as usize];
        let capture = self.game.board[to as usize];

        let mt = if args[0].len() == 5 {
            let promotion = match args[0].chars().nth(4) {
                Some('n') => KNIGHT_PROMOTION,
                Some('b') => BISHOP_PROMOTION,
                Some('r') => ROOK_PROMOTION,
                Some('q') => QUEEN_PROMOTION,
                _         => panic!("could not parse promotion")
            };
            if capture == EMPTY {
                promotion
            } else {
                promotion | CAPTURE
            }
        } else if piece.kind() == KING && from == E1 ^ 56 * side && to == G1 ^ 56 * side {
            KING_CASTLE
        } else if piece.kind() == KING && from == E1 ^ 56 * side && to == C1 ^ 56 * side {
            QUEEN_CASTLE
        } else if capture == EMPTY {
            let d = (to ^ 56 * side) as Direction - (from ^ 56 * side) as Direction;
            if piece.kind() == PAWN && (d == 2 * UP) {
                DOUBLE_PAWN_PUSH
            } else if piece.kind() == PAWN && to == self.game.positions.top().en_passant {
                EN_PASSANT
            } else {
                QUIET_MOVE
            }
        } else {
            CAPTURE
        };

        let m = Move::new(from, to, mt);
        self.game.make_move(m);
        self.game.history.push(m);

        if !self.force {
            self.think();
        }
    }

    pub fn think(&mut self) {
        let m = self.game.root(self.max_depth);
        self.game.make_move(m);
        self.game.history.push(m);

        println!("move {}", m.to_can());
    }
}
