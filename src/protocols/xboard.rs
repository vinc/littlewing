use std::io;
use regex::Regex;

use common::*;
use attack::Attack;
use clock::Clock;
use fen::FEN;
use game::Game;
use moves_generator::MovesGenerator;
use search::Search;
use version;

pub struct XBoard {
    pub game: Game,
    max_depth: Depth,
    force: bool
}

impl XBoard {
    pub fn new() -> XBoard {
        XBoard {
            game: Game::from_fen(DEFAULT_FEN),
            max_depth: (MAX_PLY - 10) as Depth,
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
                "memory"   => self.cmd_memory(&*args),
                "cores"    => self.cmd_cores(&*args),
                "sd"       => self.cmd_depth(&*args),
                "level"    => self.cmd_level(&*args),
                "protover" => self.cmd_protover(&*args),
                _          => self.parse_move(&*args)
            }
        }
    }

    fn cmd_force(&mut self) {
        self.force = true;
    }

    fn cmd_new(&mut self) {
        self.max_depth = (MAX_PLY - 10) as Depth;
        self.game.clear();
        self.game.load_fen(DEFAULT_FEN);
    }

    fn cmd_go(&mut self) {
        self.force = false;
        self.think();
    }

    fn cmd_post(&mut self) {
        self.game.is_verbose = true;
    }

    fn cmd_nopost(&mut self) {
        self.game.is_verbose = false;
    }

    fn cmd_undo(&mut self) {
        if self.game.history.len() > 0 {
            let m = self.game.history.pop().unwrap();
            self.game.undo_move(m);
        }
    }

    fn cmd_remove(&mut self) {
        let m = self.game.history.pop().unwrap();
        self.game.undo_move(m);

        let m = self.game.history.pop().unwrap();
        self.game.undo_move(m);
    }

    fn cmd_time(&mut self, args: &[&str]) {
        // `time` is given in centiseconds
        let time = args[1].parse::<u64>().unwrap();
        self.game.clock.set_time(time * 10);
    }

    fn cmd_ping(&mut self, args: &[&str]) {
        println!("pong {}", args[1].parse::<usize>().unwrap());
    }

    fn cmd_setboard(&mut self, args: &[&str]) {
        if args.len() == 1 {
            panic!("no fen given");
        }

        let fen = args[1..].join(" ");

        self.game.clear();
        self.game.load_fen(&fen);
    }

    fn cmd_level(&mut self, args: &[&str]) {
        let mut moves = args[1].parse::<u16>().unwrap();

        if moves == 0 {
            // FIXME: 0 means "play the whole game in this time control period"
            // which is unsupported by our time management so we set it to
            // another value instead.
            moves = 60;
        }

        // `time` is given in `mm:ss` or `ss`.
        let time = match args[2].find(':') {
            Some(i) => args[2][0..i].parse::<u64>().unwrap() * 60 +
                       args[2][(i + 1)..].parse::<u64>().unwrap(),
            None    => args[2].parse::<u64>().unwrap()
        };

        self.game.clock = Clock::new(moves, time * 1000);
    }

    fn cmd_depth(&mut self, args: &[&str]) {
        self.max_depth = args[1].parse::<Depth>().unwrap() + 1;
    }

    fn cmd_memory(&mut self, args: &[&str]) {
        let memory = args[1].parse::<usize>().unwrap(); // In MB
        self.game.tt_resize(memory << 20);
    }

    fn cmd_cores(&mut self, args: &[&str]) {
        self.game.threads_count = args[1].parse::<usize>().unwrap();
    }

    #[allow(unused_variables)] // TODO: remove that
    fn cmd_protover(&mut self, args: &[&str]) {
        println!("feature myname=\"{}\"", version());
        println!("feature sigint=0 ping=1 setboard=1 memory=1 smp=1 done=1");
        // TODO: check that the features got accepted
    }

    fn parse_move(&mut self, args: &[&str]) {
        let re = Regex::new(r"^[a-h][0-9][a-h][0-9][nbrq]?$").unwrap();
        if !re.is_match(args[0]) {
            return;
        }

        let m = self.game.move_from_can(&args[0]);
        self.game.make_move(m);
        self.game.history.push(m);

        if !self.force {
            self.think();
        }
    }

    fn think(&mut self) {
        let n = self.max_depth;
        match self.game.search(1..n) {
            None => {
                if self.game.is_check(WHITE) {
                    println!("0-1 {{black mates}}");
                } else if self.game.is_check(BLACK) {
                    println!("1-0 {{white mates}}");
                } else {
                    println!("1/2-1/2 {{draw}}");
                }
            },
            Some(m) => {
                self.game.make_move(m);
                self.game.history.push(m);

                println!("move {}", m.to_can());
            }
        }
    }
}
